use crate::services::wan2gp::state::wan2gp_settings::{Wan2gpSettings, Wan2gpSettingsData};
use log::info;
use serde_derive::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct UpdateWan2gpSettingsRequest {
  pub bridge_url: Option<String>,
  pub selected_model: Option<String>,
  pub resolution: Option<String>,
  pub seed: Option<i64>,
  pub num_inference_steps: Option<u32>,
  pub guidance_scale: Option<f64>,
  pub video_length: Option<u32>,
  pub profile_params: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn wan2gp_update_settings_command(
  request: UpdateWan2gpSettingsRequest,
  wan2gp_settings: State<'_, Wan2gpSettings>,
) -> Result<(), String> {
  let mut current = wan2gp_settings.snapshot();

  if let Some(url) = request.bridge_url {
    current.bridge_url = url;
  }
  if request.selected_model.is_some() {
    current.selected_model = request.selected_model;
  }
  if request.resolution.is_some() {
    current.resolution = request.resolution;
  }
  if let Some(seed) = request.seed {
    current.seed = Some(seed);
  }
  if let Some(steps) = request.num_inference_steps {
    current.num_inference_steps = Some(steps);
  }
  if let Some(scale) = request.guidance_scale {
    current.guidance_scale = Some(scale);
  }
  if let Some(length) = request.video_length {
    current.video_length = Some(length);
  }
  if request.profile_params.is_some() {
    current.profile_params = request.profile_params;
  }

  info!("Wan2GP settings updated: bridge_url={}, model={:?}", current.bridge_url, current.selected_model);
  wan2gp_settings.update(current);
  Ok(())
}
