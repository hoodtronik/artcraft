//! Data types matching the Wan2GP Bridge API JSON contract.

use serde_derive::{Deserialize, Serialize};

// ── Status ──────────────────────────────────────────────────────────────────

/// Response from `GET /artcraft/api/status`
#[derive(Debug, Deserialize)]
pub struct BridgeStatus {
  pub online: bool,
  pub version: String,
  pub engine: String,
  pub gpu: String,
  pub gpu_vram_gb: f64,
  pub api_base: String,
  pub active_tasks: u32,
}

// ── Models ──────────────────────────────────────────────────────────────────

/// Response from `GET /artcraft/api/models`
#[derive(Debug, Deserialize)]
pub struct ModelList {
  pub video: Vec<ModelInfo>,
  pub image: Vec<ModelInfo>,
}

/// A single model discovered by Wan2GP.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelInfo {
  pub id: String,
  pub name: String,
  pub architecture: String,
  pub source: String,

  /// "video" or "image"
  #[serde(rename = "type")]
  pub model_type: String,

  /// Whether this is an Image-to-Video model (only for video models).
  #[serde(default)]
  #[serde(rename = "isI2V")]
  pub is_i2v: bool,

  #[serde(default)]
  pub description: String,
}

/// A speed profile for a model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpeedProfile {
  pub name: String,
  pub params: serde_json::Value,
}

// ── Generate ────────────────────────────────────────────────────────────────

/// Request body for `POST /artcraft/api/generate`
#[derive(Debug, Serialize)]
pub struct GenerateRequest {
  pub model: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub prompt: Option<String>,

  /// "video" or "image"
  #[serde(skip_serializing_if = "Option::is_none")]
  pub output_type: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolution: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub seed: Option<i64>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub num_inference_steps: Option<u32>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub guidance_scale: Option<f64>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub video_length: Option<u32>,

  /// Base64-encoded start image (for I2V models).
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "image_start_b64")]
  pub image_start: Option<String>,

  /// Base64-encoded end image.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "image_end_b64")]
  pub image_end: Option<String>,

  /// Base64-encoded reference images.
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(rename = "image_refs_b64")]
  pub image_refs: Option<Vec<String>>,

  /// Speed profile parameters to apply.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub profile_params: Option<serde_json::Value>,

  /// Additional override parameters.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extra_params: Option<serde_json::Value>,
}

/// Response from `POST /artcraft/api/generate`
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
  pub task_id: String,
  pub status: String,
  pub poll_url: String,
}

// ── Task status ─────────────────────────────────────────────────────────────

/// Response from `GET /artcraft/api/tasks/<id>`
#[derive(Debug, Deserialize)]
pub struct TaskStatusResponse {
  pub id: String,
  pub status: String,
  pub progress: f64,
  pub progress_message: Option<String>,
  pub error: Option<String>,
  pub result_type: Option<String>,
  pub result_path: Option<String>,
  pub download_url: Option<String>,
  pub created_at: Option<f64>,
  pub started_at: Option<f64>,
  pub completed_at: Option<f64>,
}

impl TaskStatusResponse {
  pub fn is_complete(&self) -> bool {
    self.status == "complete"
  }

  pub fn is_failed(&self) -> bool {
    self.status == "failed"
  }

  pub fn is_cancelled(&self) -> bool {
    self.status == "cancelled"
  }

  pub fn is_terminal(&self) -> bool {
    self.is_complete() || self.is_failed() || self.is_cancelled()
  }
}

/// Response from `POST /artcraft/api/tasks/<id>/cancel`
#[derive(Debug, Deserialize)]
pub struct CancelResponse {
  pub id: String,
  pub status: String,
}

/// Generic error response from the bridge API.
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
  pub error: String,
}
