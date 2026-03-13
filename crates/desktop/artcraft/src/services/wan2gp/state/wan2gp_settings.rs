use serde_derive::{Deserialize, Serialize};
use std::sync::RwLock;

/// Runtime settings for the Wan2GP bridge integration.
///
/// These are stored in-memory and persisted to the app's settings file.
/// The frontend can read/write these via Tauri commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wan2gpSettingsData {
  /// URL of the Wan2GP bridge API (default: http://localhost:7861)
  pub bridge_url: String,

  /// Currently selected model ID (e.g., "i2v_2_2")
  pub selected_model: Option<String>,

  /// Resolution string (e.g., "832x480")
  pub resolution: Option<String>,

  /// Random seed (-1 for random)
  pub seed: Option<i64>,

  /// Number of inference steps
  pub num_inference_steps: Option<u32>,

  /// CFG / guidance scale
  pub guidance_scale: Option<f64>,

  /// Video length in frames
  pub video_length: Option<u32>,

  /// Speed profile parameters (from the profiles endpoint)
  pub profile_params: Option<serde_json::Value>,
}

impl Default for Wan2gpSettingsData {
  fn default() -> Self {
    Self {
      bridge_url: "http://localhost:7861".to_string(),
      selected_model: None,
      resolution: Some("832x480".to_string()),
      seed: Some(-1),
      num_inference_steps: None,
      guidance_scale: None,
      video_length: None,
      profile_params: None,
    }
  }
}

/// Thread-safe wrapper around Wan2GP settings.
/// Managed as Tauri state.
pub struct Wan2gpSettings {
  data: RwLock<Wan2gpSettingsData>,
}

impl Wan2gpSettings {
  pub fn new() -> Self {
    Self {
      data: RwLock::new(Wan2gpSettingsData::default()),
    }
  }

  pub fn from_data(data: Wan2gpSettingsData) -> Self {
    Self {
      data: RwLock::new(data),
    }
  }

  // Accessor properties that read through the lock

  pub fn bridge_url(&self) -> String {
    self.data.read().unwrap().bridge_url.clone()
  }

  pub fn selected_model(&self) -> Option<String> {
    self.data.read().unwrap().selected_model.clone()
  }

  pub fn resolution(&self) -> Option<String> {
    self.data.read().unwrap().resolution.clone()
  }

  pub fn seed(&self) -> Option<i64> {
    self.data.read().unwrap().seed
  }

  pub fn num_inference_steps(&self) -> Option<u32> {
    self.data.read().unwrap().num_inference_steps
  }

  pub fn guidance_scale(&self) -> Option<f64> {
    self.data.read().unwrap().guidance_scale
  }

  pub fn video_length(&self) -> Option<u32> {
    self.data.read().unwrap().video_length
  }

  pub fn profile_params(&self) -> Option<serde_json::Value> {
    self.data.read().unwrap().profile_params.clone()
  }

  /// Update all settings at once.
  pub fn update(&self, new_data: Wan2gpSettingsData) {
    let mut data = self.data.write().unwrap();
    *data = new_data;
  }

  /// Get a snapshot of the current settings.
  pub fn snapshot(&self) -> Wan2gpSettingsData {
    self.data.read().unwrap().clone()
  }
}

impl Default for Wan2gpSettings {
  fn default() -> Self {
    Self::new()
  }
}
