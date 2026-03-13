use crate::core::commands::enqueue::generate_error::GenerateError;
use crate::core::commands::enqueue::image_to_video::enqueue_image_to_video_command::EnqueueImageToVideoRequest;
use crate::core::commands::enqueue::task_enqueue_success::TaskEnqueueSuccess;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::utils::download_media_file_to_temp_dir::download_media_file_to_temp_dir;
use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use anyhow::anyhow;
use base64::Engine;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info};
use wan2gp_client::client::Wan2gpClient;
use wan2gp_client::types::GenerateRequest;

/// Handle video generation via the local Wan2GP bridge.
///
/// Unlike cloud providers, this requires no credentials — just a running
/// Wan2GP instance with the ArtCraft Bridge plugin enabled.
pub async fn handle_wan2gp_video(
  request: &EnqueueImageToVideoRequest,
  app_data_root: &AppDataRoot,
  app_env_configs: &AppEnvConfigs,
  wan2gp_settings: &Wan2gpSettings,
) -> Result<TaskEnqueueSuccess, GenerateError> {

  let bridge_url = wan2gp_settings.bridge_url();
  let client = Wan2gpClient::with_base_url(&bridge_url);

  // Check that the bridge is online
  let status = client.status().await.map_err(|e| {
    error!("Wan2GP bridge is not reachable: {}", e);
    GenerateError::AnyhowError(anyhow!(
      "Cannot connect to Wan2GP. Make sure Wan2GP is running with the ArtCraft Bridge plugin enabled. Error: {}",
      e
    ))
  })?;

  info!("Wan2GP bridge online: engine={}, gpu={}", status.engine, status.gpu);

  // Determine the model: prefer the request's model ID (from the Local mode selector),
  // fall back to the global selected_model in settings.
  let wan2gp_model = request.wan2gp_model_id.clone()
    .or_else(|| wan2gp_settings.selected_model())
    .ok_or_else(|| {
      GenerateError::AnyhowError(anyhow!("No Wan2GP model selected. Pick a model from the Local mode selector or configure one in Wan2GP settings."))
    })?;

  let b64_engine = base64::engine::general_purpose::STANDARD;

  // Handle start image (for I2V models)
  let image_start_b64 = if let Some(image_token) = request.image_media_token.as_ref() {
    let local_file = download_media_file_to_temp_dir(
      app_env_configs,
      app_data_root,
      image_token,
    ).await?;
    let bytes = tokio::fs::read(local_file.path()).await.map_err(GenerateError::IoError)?;
    Some(b64_engine.encode(&bytes))
  } else {
    None
  };

  // Handle end image
  let image_end_b64 = if let Some(end_token) = request.end_frame_image_media_token.as_ref() {
    let local_file = download_media_file_to_temp_dir(
      app_env_configs,
      app_data_root,
      end_token,
    ).await?;
    let bytes = tokio::fs::read(local_file.path()).await.map_err(GenerateError::IoError)?;
    Some(b64_engine.encode(&bytes))
  } else {
    None
  };

  // Handle reference images
  let image_refs_b64 = if let Some(ref_tokens) = request.reference_image_media_tokens.as_ref() {
    let mut refs = Vec::new();
    for token in ref_tokens {
      let local_file = download_media_file_to_temp_dir(
        app_env_configs,
        app_data_root,
        token,
      ).await?;
      let bytes = tokio::fs::read(local_file.path()).await.map_err(GenerateError::IoError)?;
      refs.push(b64_engine.encode(&bytes));
    }
    Some(refs)
  } else {
    None
  };

  // Build the generation request
  let gen_request = GenerateRequest {
    model: wan2gp_model.clone(),
    prompt: request.prompt.clone(),
    output_type: Some("video".to_string()),
    resolution: wan2gp_settings.resolution(),
    seed: wan2gp_settings.seed(),
    num_inference_steps: wan2gp_settings.num_inference_steps(),
    guidance_scale: wan2gp_settings.guidance_scale(),
    video_length: wan2gp_settings.video_length(),
    image_start: image_start_b64,
    image_end: image_end_b64,
    image_refs: image_refs_b64,
    profile_params: wan2gp_settings.profile_params(),
    extra_params: None,
  };

  info!("Submitting generation to Wan2GP bridge: model={}", wan2gp_model);

  // Submit the task
  let gen_response = client.generate(&gen_request).await.map_err(|e| {
    error!("Wan2GP generation submission failed: {}", e);
    GenerateError::AnyhowError(anyhow!("Wan2GP generation failed: {}", e))
  })?;

  info!("Wan2GP task created: task_id={}", gen_response.task_id);

  Ok(TaskEnqueueSuccess {
    provider: GenerationProvider::Wan2gp,
    model: None, // Dynamic — Wan2GP models aren't in GenerationModel enum
    provider_job_id: Some(gen_response.task_id),
    task_type: TaskType::VideoGeneration,
  })
}
