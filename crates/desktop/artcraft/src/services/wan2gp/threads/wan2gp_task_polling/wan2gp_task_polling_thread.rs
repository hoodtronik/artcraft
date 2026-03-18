use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::generation_events::common::{GenerationAction, GenerationServiceProvider};
use crate::core::events::generation_events::generation_complete_event::GenerationCompleteEvent;
use crate::core::events::generation_events::generation_failed_event::GenerationFailedEvent;
use crate::core::events::functional_events::text_to_image_generation_complete_event::{
  GeneratedImage, TextToImageGenerationCompleteEvent,
};
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::data_dir::trait_data_subdir::DataSubdir;
use crate::core::state::task_database::TaskDatabase;
use crate::core::utils::task_database_pending_statuses::TASK_DATABASE_PENDING_STATUSES;
use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_media_file_class::TaskMediaFileClass;
use enums::tauri::tasks::task_status::TaskStatus;
use enums::tauri::tasks::task_type::TaskType;
use tokens::tokens::media_files::MediaFileToken;
use log::{error, info, warn};
use sqlite_tasks::queries::list_tasks_by_provider_and_status::{
  list_tasks_by_provider_and_status, ListTasksByProviderAndStatusArgs,
};
use sqlite_tasks::queries::update_successful_task_status_with_metadata::{
  update_successful_task_status_with_metadata, UpdateSuccessfulTaskArgs,
};
use sqlite_tasks::queries::update_task_status::{update_task_status, UpdateTaskArgs};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use wan2gp_client::client::Wan2gpClient;

/// Background thread that polls the Wan2GP bridge for task completion.
///
/// Unlike cloud providers, Wan2GP tasks are local — no credentials needed,
/// just check the bridge API for status and download results.
pub async fn wan2gp_task_polling_thread(
  app_handle: AppHandle,
  app_data_root: AppDataRoot,
  task_database: TaskDatabase,
  wan2gp_settings: Arc<Wan2gpSettings>,
) -> ! {
  info!("[Wan2GP Polling] Starting task polling thread...");

  loop {
    let res = polling_loop(
      &app_handle,
      &app_data_root,
      &task_database,
      &wan2gp_settings,
    )
    .await;
    if let Err(err) = res {
      error!("[Wan2GP Polling] Error: {:?}", err);
    }
    // Sleep before next cycle
    tokio::time::sleep(std::time::Duration::from_millis(5_000)).await;
  }
}

async fn polling_loop(
  app_handle: &AppHandle,
  app_data_root: &AppDataRoot,
  task_database: &TaskDatabase,
  wan2gp_settings: &Wan2gpSettings,
) -> Result<(), anyhow::Error> {
  // Find all pending Wan2GP tasks
  let local_tasks = list_tasks_by_provider_and_status(ListTasksByProviderAndStatusArgs {
    db: task_database.get_connection(),
    provider: GenerationProvider::Wan2gp,
    task_statuses: &TASK_DATABASE_PENDING_STATUSES,
  })
  .await?;

  let tasks = local_tasks.tasks;
  if tasks.is_empty() {
    return Ok(());
  }

  info!("[Wan2GP Polling] {} pending tasks", tasks.len());

  let bridge_url = wan2gp_settings.bridge_url();
  let client = Wan2gpClient::with_base_url(&bridge_url);

  for task in tasks.iter() {
    let task_id_str = match &task.provider_job_id {
      Some(id) => id.clone(),
      None => {
        warn!("[Wan2GP Polling] Task {} has no provider_job_id, skipping", task.id);
        continue;
      }
    };

    // Poll the bridge for this task's status
    let status = match client.poll_task(&task_id_str).await {
      Ok(s) => s,
      Err(e) => {
        warn!("[Wan2GP Polling] Failed to get status for task {}: {}", task_id_str, e);
        continue;
      }
    };

    match status.status.as_str() {
      "complete" => {
        info!("[Wan2GP Polling] Task {} complete!", task_id_str);

        // Determine if this is an image or video based on the task type in our database
        let is_image = task.task_type == TaskType::ImageGeneration
          || task.task_type == TaskType::ImageInpaintEdit
          || status.result_type.as_deref() == Some("image");

        // Download the result from the bridge
        let (result_bytes, result_path) = match client.download_result(&task_id_str).await {
          Ok(bytes) => {
            let ext = if is_image { "png" } else { "mp4" };
            let filename = format!("wan2gp_{}.{}", task_id_str, ext);
            let temp_dir = app_data_root.temp_dir().path();
            let dest = temp_dir.join(&filename);

            if let Err(e) = tokio::fs::write(&dest, &bytes).await {
              error!("[Wan2GP Polling] Failed to write result file: {}", e);
              (Some(bytes), None)
            } else {
              info!("[Wan2GP Polling] Downloaded result to: {:?} ({} bytes)", dest, bytes.len());
              (Some(bytes), Some(dest))
            }
          }
          Err(e) => {
            warn!("[Wan2GP Polling] Failed to download result for {}: {}", task_id_str, e);
            (None, None)
          }
        };

        // Build a displayable URL for the frontend
        let maybe_cdn_url = if is_image {
          // For images: use a data URL (base64-encoded) — works everywhere, no Tauri permissions needed
          result_bytes.as_ref().map(|bytes| {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
            format!("data:image/png;base64,{}", b64)
          })
        } else {
          // For videos: use a file:// URL (Tauri allows these for video elements)
          result_path.as_ref().map(|p| {
            let path_str = p.display().to_string().replace('\\', "/");
            format!("file:///{}", path_str)
          })
        };

        let media_class = if is_image {
          TaskMediaFileClass::Image
        } else {
          TaskMediaFileClass::Video
        };

        let generation_action = if is_image {
          GenerationAction::GenerateImage
        } else {
          GenerationAction::GenerateVideo
        };

        // Generate a synthetic media file token for local Wan2GP results.
        // The frontend's get_task_queue_command uses .zip() on token + cdn_url,
        // so BOTH must be Some for completed_item to be populated.
        let synthetic_token = MediaFileToken(format!("wan2gp_{}", task_id_str));

        // Update the task database
        let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs {
          db: task_database.get_connection(),
          task_id: &task.id,
          maybe_batch_token: None,
          maybe_primary_media_file_token: Some(&synthetic_token),
          maybe_primary_media_file_class: Some(media_class),
          maybe_primary_media_file_thumbnail_url_template: None,
          maybe_primary_media_file_cdn_url: maybe_cdn_url.as_deref(),
        })
        .await?;

        if updated {
          info!("[Wan2GP Polling] Task {} marked as complete in database", task_id_str);

          // Notify the frontend (general completion event for task queue)
          let event = GenerationCompleteEvent {
            action: Some(generation_action),
            service: GenerationServiceProvider::Wan2gp,
            model: None,
          };
          if let Err(err) = event.send(app_handle) {
            error!("[Wan2GP Polling] Failed to send completion event: {:?}", err);
          }

          // For image tasks: also emit the text_to_image_generation_complete_event
          // so the main UI page (TextToImage.tsx) can update its batch from "pending" to "complete"
          if is_image {
            if let Some(ref cdn_url_str) = maybe_cdn_url {
              match url::Url::parse(cdn_url_str) {
                Ok(parsed_url) => {
                  let img_event = TextToImageGenerationCompleteEvent {
                    generated_images: vec![GeneratedImage {
                      media_token: synthetic_token.clone(),
                      cdn_url: parsed_url,
                      maybe_thumbnail_template: None,
                    }],
                    maybe_frontend_subscriber_id: task.frontend_subscriber_id.clone(),
                    maybe_frontend_subscriber_payload: task.frontend_subscriber_payload.clone(),
                  };
                  if let Err(err) = img_event.send(app_handle) {
                    error!("[Wan2GP Polling] Failed to send text-to-image event: {:?}", err);
                  }
                }
                Err(e) => {
                  warn!("[Wan2GP Polling] Could not parse cdn_url for image event: {}", e);
                }
              }
            }
          }
        }
      }
      "failed" | "error" => {
        let err_msg = status.error.unwrap_or_else(|| "Unknown error".to_string());
        error!("[Wan2GP Polling] Task {} failed: {}", task_id_str, err_msg);

        let _ = update_task_status(UpdateTaskArgs {
          db: task_database.get_connection(),
          task_id: &task.id,
          status: TaskStatus::CompleteFailure,
        })
        .await;

        // Determine the action type from the task
        let is_image = task.task_type == TaskType::ImageGeneration
          || task.task_type == TaskType::ImageInpaintEdit;

        let generation_action = if is_image {
          GenerationAction::GenerateImage
        } else {
          GenerationAction::GenerateVideo
        };

        // Emit GenerationFailedEvent so the main UI can mark batches as failed
        let fail_event = GenerationFailedEvent {
          action: generation_action,
          service: GenerationServiceProvider::Wan2gp,
          model: None,
          reason: Some(err_msg),
        };
        if let Err(err) = fail_event.send(app_handle) {
          error!("[Wan2GP Polling] Failed to send failure event: {:?}", err);
        }

        // Also emit GenerationCompleteEvent so the notification dropdown refreshes
        let complete_event = GenerationCompleteEvent {
          action: Some(generation_action),
          service: GenerationServiceProvider::Wan2gp,
          model: None,
        };
        if let Err(err) = complete_event.send(app_handle) {
          error!("[Wan2GP Polling] Failed to send completion event for failure: {:?}", err);
        }
      }
      "cancelled" => {
        info!("[Wan2GP Polling] Task {} was cancelled", task_id_str);

        let _ = update_task_status(UpdateTaskArgs {
          db: task_database.get_connection(),
          task_id: &task.id,
          status: TaskStatus::CancelledByProvider,
        })
        .await;
      }
      "running" | "queued" | "pending" => {
        // Emit real progress to frontend so it can override time-based progress bar
        let progress_pct = (status.progress * 100.0).min(99.0);
        let progress_msg = status.progress_message.clone().unwrap_or_default();

        if status.progress > 0.0 {
          info!(
            "[Wan2GP Polling] Task {} progress: {:.0}% — {}",
            task_id_str,
            progress_pct,
            progress_msg
          );
        }

        // Emit lightweight progress event (no TauriEventName needed)
        let _ = app_handle.emit("wan2gp-progress-event", serde_json::json!({
          "provider_job_id": task_id_str,
          "progress": progress_pct,
          "message": progress_msg,
          "status": status.status,
        }));
      }
      other => {
        warn!("[Wan2GP Polling] Task {} has unknown status: {}", task_id_str, other);
      }
    }

    // Small delay between task checks
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
  }

  Ok(())
}
