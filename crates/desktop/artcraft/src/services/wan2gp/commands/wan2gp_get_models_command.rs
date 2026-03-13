use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use serde_derive::Serialize;
use tauri::State;
use wan2gp_client::client::Wan2gpClient;

#[derive(Serialize)]
pub struct Wan2gpModelInfo {
  pub id: String,
  pub name: String,
  pub architecture: String,
  pub model_type: String,
  pub is_i2v: bool,
  pub description: String,
}

#[derive(Serialize)]
pub struct Wan2gpModelsResponse {
  pub video: Vec<Wan2gpModelInfo>,
  pub image: Vec<Wan2gpModelInfo>,
  pub error: Option<String>,
}

fn convert_model(m: &wan2gp_client::types::ModelInfo) -> Wan2gpModelInfo {
  Wan2gpModelInfo {
    id: m.id.clone(),
    name: m.name.clone(),
    architecture: m.architecture.clone(),
    model_type: m.model_type.clone(),
    is_i2v: m.is_i2v,
    description: m.description.clone(),
  }
}

#[tauri::command]
pub async fn wan2gp_get_models_command(
  wan2gp_settings: State<'_, Wan2gpSettings>,
) -> Result<Wan2gpModelsResponse, String> {
  let bridge_url = wan2gp_settings.bridge_url();
  let client = Wan2gpClient::with_base_url(&bridge_url);

  match client.list_models().await {
    Ok(models) => Ok(Wan2gpModelsResponse {
      video: models.video.iter().map(convert_model).collect(),
      image: models.image.iter().map(convert_model).collect(),
      error: None,
    }),
    Err(err) => Ok(Wan2gpModelsResponse {
      video: vec![],
      image: vec![],
      error: Some(err.to_string()),
    }),
  }
}
