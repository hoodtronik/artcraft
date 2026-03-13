use std::fmt;

/// Errors from the Wan2GP bridge API client.
#[derive(Debug)]
pub enum Wan2gpError {
  /// The bridge API server is not reachable.
  ConnectionFailed(String),

  /// The bridge returned an HTTP error status.
  HttpError { status: u16, message: String },

  /// Failed to parse the response body.
  ParseError(String),

  /// The generation task failed on the Wan2GP side.
  GenerationFailed(String),

  /// The generation task timed out.
  Timeout,

  /// The generation task was cancelled.
  Cancelled,

  /// Generic error wrapper.
  Other(anyhow::Error),
}

impl fmt::Display for Wan2gpError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ConnectionFailed(msg) => write!(f, "Wan2GP bridge connection failed: {}", msg),
      Self::HttpError { status, message } => write!(f, "Wan2GP HTTP error {}: {}", status, message),
      Self::ParseError(msg) => write!(f, "Wan2GP response parse error: {}", msg),
      Self::GenerationFailed(msg) => write!(f, "Wan2GP generation failed: {}", msg),
      Self::Timeout => write!(f, "Wan2GP generation timed out"),
      Self::Cancelled => write!(f, "Wan2GP generation was cancelled"),
      Self::Other(err) => write!(f, "Wan2GP error: {}", err),
    }
  }
}

impl std::error::Error for Wan2gpError {}

impl From<reqwest::Error> for Wan2gpError {
  fn from(err: reqwest::Error) -> Self {
    if err.is_connect() {
      Self::ConnectionFailed(err.to_string())
    } else if err.is_timeout() {
      Self::Timeout
    } else {
      Self::Other(err.into())
    }
  }
}

impl From<anyhow::Error> for Wan2gpError {
  fn from(err: anyhow::Error) -> Self {
    Self::Other(err)
  }
}

impl From<serde_json::Error> for Wan2gpError {
  fn from(err: serde_json::Error) -> Self {
    Self::ParseError(err.to_string())
  }
}
