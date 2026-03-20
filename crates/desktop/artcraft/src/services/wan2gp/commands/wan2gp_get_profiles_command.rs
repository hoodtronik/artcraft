use crate::services::wan2gp::state::wan2gp_settings::Wan2gpSettings;
use serde_derive::Serialize;
use tauri::State;
use wan2gp_client::client::Wan2gpClient;

#[derive(Serialize)]
pub struct Wan2gpProfileInfo {
  pub name: String,
  pub params: serde_json::Value,
}

#[derive(Serialize)]
pub struct Wan2gpProfilesResponse {
  pub profiles: Vec<Wan2gpProfileInfo>,
  pub error: Option<String>,
}

#[tauri::command]
pub async fn wan2gp_get_profiles_command(
  model_id: String,
  wan2gp_settings: State<'_, Wan2gpSettings>,
) -> Result<Wan2gpProfilesResponse, String> {
  let bridge_url = wan2gp_settings.bridge_url();
  let client = Wan2gpClient::with_base_url(&bridge_url);

  match client.get_profiles(&model_id).await {
    Ok(profiles) => Ok(Wan2gpProfilesResponse {
      profiles: profiles
        .into_iter()
        .map(|p| Wan2gpProfileInfo {
          name: p.name,
          params: p.params,
        })
        .collect(),
      error: None,
    }),
    Err(err) => Ok(Wan2gpProfilesResponse {
      profiles: vec![],
      error: Some(err.to_string()),
    }),
  }
}
