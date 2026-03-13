use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use serde_derive::Serialize;
use tauri::State;
use wan2gp_client::client::Wan2gpClient;

#[derive(Serialize)]
pub struct Wan2gpStatusResponse {
  pub online: bool,
  pub engine: Option<String>,
  pub gpu: Option<String>,
  pub gpu_vram_gb: Option<f64>,
  pub active_tasks: Option<u32>,
  pub error: Option<String>,
}

#[tauri::command]
pub async fn wan2gp_get_status_command(
  wan2gp_settings: State<'_, Wan2gpSettings>,
) -> Result<Wan2gpStatusResponse, String> {
  let bridge_url = wan2gp_settings.bridge_url();
  let client = Wan2gpClient::with_base_url(&bridge_url);

  match client.status().await {
    Ok(status) => Ok(Wan2gpStatusResponse {
      online: status.online,
      engine: Some(status.engine),
      gpu: Some(status.gpu),
      gpu_vram_gb: Some(status.gpu_vram_gb),
      active_tasks: Some(status.active_tasks),
      error: None,
    }),
    Err(err) => Ok(Wan2gpStatusResponse {
      online: false,
      engine: None,
      gpu: None,
      gpu_vram_gb: None,
      active_tasks: None,
      error: Some(err.to_string()),
    }),
  }
}
