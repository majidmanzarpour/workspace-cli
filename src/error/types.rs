use serde::Serialize;
use thiserror::Error;

/// Structured error response for agent consumption
#[derive(Debug, Serialize)]
pub struct CliError {
    pub status: &'static str,  // Always "error"
    pub error_code: ErrorCode,
    pub domain: String,  // "gmail", "drive", "calendar", etc.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actionable_fix: Option<String>,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    AuthenticationFailed,
    TokenExpired,
    RateLimitExceeded,
    QuotaExceeded,
    NotFound,
    PermissionDenied,
    InvalidRequest,
    NetworkError,
    ServerError,
    ConfigurationError,
    Unknown,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Error)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub domain: String,
    pub retry_after: Option<u64>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.domain, self.message)
    }
}

impl CliError {
    pub fn new(code: ErrorCode, domain: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: "error",
            error_code: code,
            domain: domain.into(),
            message: message.into(),
            retry_after_seconds: None,
            actionable_fix: None,
        }
    }

    pub fn with_retry(mut self, seconds: u64) -> Self {
        self.retry_after_seconds = Some(seconds);
        self
    }

    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.actionable_fix = Some(fix.into());
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"status":"error","error_code":"unknown","message":"Failed to serialize error"}"#.to_string()
        })
    }
}

// Conversion from WorkspaceError to CliError
impl From<&WorkspaceError> for CliError {
    fn from(err: &WorkspaceError) -> Self {
        match err {
            WorkspaceError::Auth(msg) => {
                CliError::new(ErrorCode::AuthenticationFailed, "auth", msg.clone())
                    .with_fix("Run 'workspace-cli auth login' to re-authenticate")
            }
            WorkspaceError::Api(api_err) => {
                let code = match api_err.code {
                    401 => ErrorCode::TokenExpired,
                    403 => ErrorCode::PermissionDenied,
                    404 => ErrorCode::NotFound,
                    429 => ErrorCode::RateLimitExceeded,
                    _ if api_err.code >= 500 => ErrorCode::ServerError,
                    _ => ErrorCode::InvalidRequest,
                };
                let mut cli_err = CliError::new(code, api_err.domain.clone(), &api_err.message);
                if let Some(retry) = api_err.retry_after {
                    cli_err = cli_err.with_retry(retry);
                }
                cli_err
            }
            WorkspaceError::Network(e) => {
                CliError::new(ErrorCode::NetworkError, "network", e.to_string())
                    .with_fix("Check your internet connection and try again")
            }
            WorkspaceError::Config(msg) => {
                CliError::new(ErrorCode::ConfigurationError, "config", msg.clone())
            }
            WorkspaceError::Io(e) => {
                CliError::new(ErrorCode::Unknown, "io", e.to_string())
            }
            WorkspaceError::Serialization(e) => {
                CliError::new(ErrorCode::Unknown, "serialization", e.to_string())
            }
            WorkspaceError::NotFound(msg) => {
                CliError::new(ErrorCode::NotFound, "resource", msg.clone())
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, WorkspaceError>;
