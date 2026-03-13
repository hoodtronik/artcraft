use crate::services::wan2gp::state::wan2gp_settings::{Wan2gpSettings, Wan2gpSettingsData};
use serde_derive::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct Wan2gpSettingsResponse {
  pub bridge_url: String,
  pub selected_model: Option<String>,
  pub resolution: Option<String>,
  pub seed: Option<i64>,
  pub num_inference_steps: Option<u32>,
  pub guidance_scale: Option<f64>,
  pub video_length: Option<u32>,
}

#[tauri::command]
pub async fn wan2gp_get_settings_command(
  wan2gp_settings: State<'_, Wan2gpSettings>,
) -> Result<Wan2gpSettingsResponse, String> {
  let snapshot = wan2gp_settings.snapshot();
  Ok(Wan2gpSettingsResponse {
    bridge_url: snapshot.bridge_url,
    selected_model: snapshot.selected_model,
    resolution: snapshot.resolution,
    seed: snapshot.seed,
    num_inference_steps: snapshot.num_inference_steps,
    guidance_scale: snapshot.guidance_scale,
    video_length: snapshot.video_length,
  })
}
