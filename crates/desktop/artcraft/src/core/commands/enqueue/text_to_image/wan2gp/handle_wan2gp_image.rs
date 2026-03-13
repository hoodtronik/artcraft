use crate::core::commands::enqueue::generate_error::GenerateError;
use crate::core::commands::enqueue::text_to_image::enqueue_text_to_image_command::EnqueueTextToImageRequest;
use crate::core::commands::enqueue::task_enqueue_success::TaskEnqueueSuccess;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use anyhow::anyhow;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info};
use wan2gp_client::client::Wan2gpClient;
use wan2gp_client::types::GenerateRequest;

/// Handle image generation via the local Wan2GP bridge.
pub async fn handle_wan2gp_image(
  request: &EnqueueTextToImageRequest,
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

  // Determine the model
  let wan2gp_model = request.wan2gp_model_id.clone()
    .or_else(|| wan2gp_settings.selected_model())
    .ok_or_else(|| {
      GenerateError::AnyhowError(anyhow!("No Wan2GP model selected. Pick a model from the Local mode selector or configure one in Wan2GP settings."))
    })?;

  // Map resolution from aspect ratio if provided
  let resolution = match &request.common_aspect_ratio {
    Some(ar) => {
      use crate::core::api_adapters::aspect_ratio::common_aspect_ratio::CommonAspectRatio;
      let res = match ar {
        CommonAspectRatio::WideSixteenByNine => "1280x720",
        CommonAspectRatio::WideFourByThree => "960x720",
        CommonAspectRatio::WideThreeByTwo => "960x640",
        CommonAspectRatio::WideFiveByFour => "896x720",
        CommonAspectRatio::Wide => "1280x720",
        CommonAspectRatio::TallNineBySixteen => "720x1280",
        CommonAspectRatio::TallThreeByFour => "720x960",
        CommonAspectRatio::TallTwoByThree => "640x960",
        CommonAspectRatio::TallFourByFive => "720x896",
        CommonAspectRatio::Tall => "720x1280",
        CommonAspectRatio::Square | CommonAspectRatio::SquareHd => "1024x1024",
        _ => "1024x1024", // Default for images
      };
      Some(res.to_string())
    }
    None => wan2gp_settings.resolution(),
  };

  // Build the generation request
  let gen_request = GenerateRequest {
    model: wan2gp_model.clone(),
    prompt: request.prompt.clone(),
    output_type: Some("image".to_string()),
    resolution,
    seed: wan2gp_settings.seed(),
    num_inference_steps: wan2gp_settings.num_inference_steps(),
    guidance_scale: wan2gp_settings.guidance_scale(),
    video_length: None, // Not applicable for images
    image_start: None,
    image_end: None,
    image_refs: None,
    profile_params: wan2gp_settings.profile_params(),
    extra_params: None,
  };

  info!("Submitting image generation to Wan2GP bridge: model={}", wan2gp_model);

  // Submit the task
  let gen_response = client.generate(&gen_request).await.map_err(|e| {
    error!("Wan2GP image generation submission failed: {}", e);
    GenerateError::AnyhowError(anyhow!("Wan2GP image generation failed: {}", e))
  })?;

  info!("Wan2GP image task created: task_id={}", gen_response.task_id);

  Ok(TaskEnqueueSuccess {
    provider: GenerationProvider::Wan2gp,
    model: None, // Dynamic — Wan2GP models aren't in GenerationModel enum
    provider_job_id: Some(gen_response.task_id),
    task_type: TaskType::ImageGeneration,
  })
}
