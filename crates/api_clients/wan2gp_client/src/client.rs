//! HTTP client for the Wan2GP ArtCraft Bridge API.

use crate::error::Wan2gpError;
use crate::types::*;
use log::{debug, info, warn};
use std::time::Duration;

/// Default bridge API base URL.
const DEFAULT_BASE_URL: &str = "http://localhost:7861";

/// Default timeout for regular API calls (status, models, etc.).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for generation polling — individual poll requests.
const POLL_TIMEOUT: Duration = Duration::from_secs(5);

/// Default interval between poll requests.
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Maximum time to wait for a generation to complete.
const DEFAULT_MAX_GENERATION_TIME: Duration = Duration::from_secs(600); // 10 minutes

/// Client for communicating with the Wan2GP Bridge API.
#[derive(Clone)]
pub struct Wan2gpClient {
  base_url: String,
  http: reqwest::Client,
  poll_interval: Duration,
  max_generation_time: Duration,
}

impl Wan2gpClient {
  /// Create a new client with the default base URL (`http://localhost:7861`).
  pub fn new() -> Self {
    Self::with_base_url(DEFAULT_BASE_URL)
  }

  /// Create a new client with a custom base URL.
  pub fn with_base_url(base_url: &str) -> Self {
    let http = reqwest::Client::builder()
      .timeout(DEFAULT_TIMEOUT)
      .build()
      .expect("Failed to build reqwest client");

    Self {
      base_url: base_url.trim_end_matches('/').to_string(),
      http,
      poll_interval: DEFAULT_POLL_INTERVAL,
      max_generation_time: DEFAULT_MAX_GENERATION_TIME,
    }
  }

  /// Set the polling interval for generation tasks.
  pub fn with_poll_interval(mut self, interval: Duration) -> Self {
    self.poll_interval = interval;
    self
  }

  /// Set the maximum time to wait for a generation to complete.
  pub fn with_max_generation_time(mut self, max_time: Duration) -> Self {
    self.max_generation_time = max_time;
    self
  }

  // ── API methods ───────────────────────────────────────────────────────

  /// Check if the bridge API is online.
  pub async fn status(&self) -> Result<BridgeStatus, Wan2gpError> {
    let url = format!("{}/artcraft/api/status", self.base_url);
    debug!("Wan2GP: GET {}", url);

    let resp = self.http.get(&url).send().await?;
    let status_code = resp.status().as_u16();

    if !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: status_code,
        message: body,
      });
    }

    let body: BridgeStatus = resp.json().await.map_err(|e| Wan2gpError::ParseError(e.to_string()))?;
    Ok(body)
  }

  /// Check if the bridge is reachable (non-failing version of `status`).
  pub async fn is_online(&self) -> bool {
    self.status().await.map(|s| s.online).unwrap_or(false)
  }

  /// List all available models (video + image).
  pub async fn list_models(&self) -> Result<ModelList, Wan2gpError> {
    let url = format!("{}/artcraft/api/models", self.base_url);
    debug!("Wan2GP: GET {}", url);

    let resp = self.http.get(&url).send().await?;

    if !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: resp.status().as_u16(),
        message: body,
      });
    }

    let models: ModelList = resp.json().await.map_err(|e| Wan2gpError::ParseError(e.to_string()))?;
    info!("Wan2GP: found {} video models, {} image models", models.video.len(), models.image.len());
    Ok(models)
  }

  /// Get speed profiles for a specific model.
  pub async fn get_profiles(&self, model_id: &str) -> Result<Vec<SpeedProfile>, Wan2gpError> {
    let url = format!("{}/artcraft/api/models/profiles/{}", self.base_url, model_id);
    debug!("Wan2GP: GET {}", url);

    let resp = self.http.get(&url).send().await?;

    if !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: resp.status().as_u16(),
        message: body,
      });
    }

    let profiles: Vec<SpeedProfile> = resp.json().await.map_err(|e| Wan2gpError::ParseError(e.to_string()))?;
    Ok(profiles)
  }

  /// Submit a generation task.
  /// Returns immediately with a task ID — use `poll_task` or `wait_for_completion` to get the result.
  pub async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse, Wan2gpError> {
    let url = format!("{}/artcraft/api/generate", self.base_url);
    info!("Wan2GP: POST {} (model={})", url, request.model);

    let resp = self.http
      .post(&url)
      .json(request)
      .send()
      .await?;

    let status_code = resp.status().as_u16();
    if status_code != 202 && !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: status_code,
        message: body,
      });
    }

    let gen_resp: GenerateResponse = resp.json().await.map_err(|e| Wan2gpError::ParseError(e.to_string()))?;
    info!("Wan2GP: task created: {}", gen_resp.task_id);
    Ok(gen_resp)
  }

  /// Poll the status of a generation task.
  pub async fn poll_task(&self, task_id: &str) -> Result<TaskStatusResponse, Wan2gpError> {
    let url = format!("{}/artcraft/api/tasks/{}", self.base_url, task_id);

    let resp = self.http
      .get(&url)
      .timeout(POLL_TIMEOUT)
      .send()
      .await?;

    if !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: resp.status().as_u16(),
        message: body,
      });
    }

    let task: TaskStatusResponse = resp.json().await.map_err(|e| Wan2gpError::ParseError(e.to_string()))?;
    Ok(task)
  }

  /// Submit a generate request, then poll until the task completes.
  /// Returns the final task status (which includes the result path and download URL).
  pub async fn generate_and_wait(&self, request: &GenerateRequest) -> Result<TaskStatusResponse, Wan2gpError> {
    let gen_resp = self.generate(request).await?;
    self.wait_for_completion(&gen_resp.task_id).await
  }

  /// Poll a task until it reaches a terminal state (complete, failed, or cancelled).
  pub async fn wait_for_completion(&self, task_id: &str) -> Result<TaskStatusResponse, Wan2gpError> {
    let start = tokio::time::Instant::now();

    loop {
      if start.elapsed() > self.max_generation_time {
        warn!("Wan2GP: task {} timed out after {:?}", task_id, self.max_generation_time);
        return Err(Wan2gpError::Timeout);
      }

      let status = self.poll_task(task_id).await?;

      if status.is_complete() {
        info!("Wan2GP: task {} complete", task_id);
        return Ok(status);
      }

      if status.is_failed() {
        let err_msg = status.error.unwrap_or_else(|| "Unknown error".to_string());
        warn!("Wan2GP: task {} failed: {}", task_id, err_msg);
        return Err(Wan2gpError::GenerationFailed(err_msg));
      }

      if status.is_cancelled() {
        return Err(Wan2gpError::Cancelled);
      }

      debug!("Wan2GP: task {} status={} progress={:.0}%",
        task_id, status.status, status.progress * 100.0);

      tokio::time::sleep(self.poll_interval).await;
    }
  }

  /// Download the result file of a completed task into memory.
  pub async fn download_result(&self, task_id: &str) -> Result<bytes::Bytes, Wan2gpError> {
    let url = format!("{}/artcraft/api/tasks/{}/download", self.base_url, task_id);
    info!("Wan2GP: downloading result for task {}", task_id);

    let resp = self.http
      .get(&url)
      .timeout(Duration::from_secs(120)) // Large file download timeout
      .send()
      .await?;

    if !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: resp.status().as_u16(),
        message: body,
      });
    }

    let bytes = resp.bytes().await.map_err(|e| Wan2gpError::Other(e.into()))?;
    info!("Wan2GP: downloaded {} bytes for task {}", bytes.len(), task_id);
    Ok(bytes)
  }

  /// Cancel a running or pending task.
  pub async fn cancel_task(&self, task_id: &str) -> Result<CancelResponse, Wan2gpError> {
    let url = format!("{}/artcraft/api/tasks/{}/cancel", self.base_url, task_id);
    info!("Wan2GP: cancelling task {}", task_id);

    let resp = self.http.post(&url).send().await?;

    if !resp.status().is_success() {
      let body = resp.text().await.unwrap_or_default();
      return Err(Wan2gpError::HttpError {
        status: resp.status().as_u16(),
        message: body,
      });
    }

    let cancel: CancelResponse = resp.json().await.map_err(|e| Wan2gpError::ParseError(e.to_string()))?;
    Ok(cancel)
  }
}

impl Default for Wan2gpClient {
  fn default() -> Self {
    Self::new()
  }
}
