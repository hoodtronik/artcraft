use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use log::{error, info};
use serde_derive::Serialize;
use tauri::State;
use wan2gp_client::client::Wan2gpClient;

#[derive(Serialize)]
pub struct Wan2gpCancelResponse {
  pub success: bool,
  pub message: Option<String>,
}

#[tauri::command]
pub async fn wan2gp_cancel_task_command(
  task_id: String,
  wan2gp_settings: State<'_, Wan2gpSettings>,
) -> Result<Wan2gpCancelResponse, String> {
  let bridge_url = wan2gp_settings.bridge_url();
  let client = Wan2gpClient::with_base_url(&bridge_url);

  info!("[Wan2GP] Cancel requested for task: {}", task_id);

  match client.cancel_task(&task_id).await {
    Ok(resp) => {
      info!("[Wan2GP] Task {} cancel response: status={}", task_id, resp.status);
      Ok(Wan2gpCancelResponse {
        success: true,
        message: Some(format!("Task {} status: {}", resp.id, resp.status)),
      })
    }
    Err(err) => {
      error!("[Wan2GP] Failed to cancel task {}: {}", task_id, err);
      Ok(Wan2gpCancelResponse {
        success: false,
        message: Some(format!("Failed to cancel: {}", err)),
      })
    }
  }
}
