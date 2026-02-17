use reqwest::{Client, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::auth::TokenManager;
use crate::error::{WorkspaceError, ApiError};
use super::rate_limiter::{ApiRateLimiter, ConcurrencyPermit};
use super::retry::{RetryConfig, Retryable, with_retry, RetryError, is_retryable_status, parse_retry_after};

/// Base URLs for Google Workspace APIs
pub mod endpoints {
    pub const GMAIL: &str = "https://gmail.googleapis.com/gmail/v1";
    pub const DRIVE: &str = "https://www.googleapis.com/drive/v3";
    pub const CALENDAR: &str = "https://www.googleapis.com/calendar/v3";
    pub const DOCS: &str = "https://docs.googleapis.com/v1";
    pub const SHEETS: &str = "https://sheets.googleapis.com/v4";
    pub const SLIDES: &str = "https://slides.googleapis.com/v1";
    pub const TASKS: &str = "https://tasks.googleapis.com/tasks/v1";
    pub const CHAT: &str = "https://chat.googleapis.com/v1";
    pub const CONTACTS: &str = "https://people.googleapis.com/v1";
    pub const GROUPS: &str = "https://cloudidentity.googleapis.com/v1";
    pub const ADMIN: &str = "https://admin.googleapis.com/admin/directory/v1";
}

/// Google Workspace API client
pub struct ApiClient {
    http: Client,
    token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>,
    rate_limiter: Option<std::sync::Arc<ApiRateLimiter>>,
    retry_config: RetryConfig,
    base_url: String,
}

impl Clone for ApiClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            token_manager: self.token_manager.clone(),
            rate_limiter: self.rate_limiter.clone(),
            retry_config: self.retry_config.clone(),
            base_url: self.base_url.clone(),
        }
    }
}

impl ApiClient {
    /// Create a new API client
    pub fn new(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            token_manager,
            rate_limiter: None,
            retry_config: RetryConfig::default(),
            base_url: String::new(),
        }
    }

    /// Set the base URL for this client
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    /// Set rate limiter
    pub fn with_rate_limiter(mut self, limiter: ApiRateLimiter) -> Self {
        self.rate_limiter = Some(std::sync::Arc::new(limiter));
        self
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Create a Gmail client
    pub fn gmail(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::GMAIL)
            .with_rate_limiter(ApiRateLimiter::gmail())
            .with_retry_config(RetryConfig::conservative())
    }

    /// Create a Drive client
    pub fn drive(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::DRIVE)
            .with_rate_limiter(ApiRateLimiter::drive())
            .with_retry_config(RetryConfig::conservative())
    }

    /// Create a Calendar client
    pub fn calendar(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::CALENDAR)
            .with_rate_limiter(ApiRateLimiter::calendar())
            .with_retry_config(RetryConfig::default())
    }

    /// Create a Docs client
    pub fn docs(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::DOCS)
            .with_rate_limiter(ApiRateLimiter::docs())
            .with_retry_config(RetryConfig::aggressive())
    }

    /// Create a Sheets client
    pub fn sheets(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::SHEETS)
            .with_rate_limiter(ApiRateLimiter::docs())
            .with_retry_config(RetryConfig::aggressive())
    }

    /// Create a Slides client
    pub fn slides(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::SLIDES)
            .with_rate_limiter(ApiRateLimiter::docs())
            .with_retry_config(RetryConfig::aggressive())
    }

    /// Create a Tasks client
    pub fn tasks(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::TASKS)
            .with_rate_limiter(ApiRateLimiter::tasks())
            .with_retry_config(RetryConfig::default())
    }

    /// Create a Google Chat client
    pub fn chat(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::CHAT)
            .with_rate_limiter(ApiRateLimiter::tasks())
            .with_retry_config(RetryConfig::default())
    }

    /// Create a Google Contacts (People API) client
    pub fn contacts(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::CONTACTS)
            .with_rate_limiter(ApiRateLimiter::tasks())
            .with_retry_config(RetryConfig::default())
    }

    /// Create a Google Groups (Cloud Identity) client
    pub fn groups(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::GROUPS)
            .with_rate_limiter(ApiRateLimiter::tasks())
            .with_retry_config(RetryConfig::default())
    }

    pub fn admin(token_manager: std::sync::Arc<tokio::sync::RwLock<TokenManager>>) -> Self {
        Self::new(token_manager)
            .with_base_url(endpoints::ADMIN)
            .with_rate_limiter(ApiRateLimiter::tasks())
            .with_retry_config(RetryConfig::default())
    }

    /// Build full URL from path
    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        }
    }

    /// Get access token
    async fn get_token(&self) -> Result<String, WorkspaceError> {
        let tm = self.token_manager.read().await;
        tm.get_access_token()
            .await
            .map_err(|e| WorkspaceError::Auth(e.to_string()))
    }

    /// Execute a GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, WorkspaceError> {
        self.request_no_body(Method::GET, path, 1).await
    }

    /// Execute a GET request with query parameters
    pub async fn get_with_query<T, Q>(&self, path: &str, query: &Q) -> Result<T, WorkspaceError>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let query_string = serde_urlencoded::to_string(query)
            .map_err(|e| WorkspaceError::Config(e.to_string()))?;

        let full_url = if query_string.is_empty() {
            self.build_url(path)
        } else {
            let base_url = self.build_url(path);
            let separator = if base_url.contains('?') { "&" } else { "?" };
            format!("{}{}{}", base_url, separator, query_string)
        };

        self.request_no_body(Method::GET, &full_url, 1).await
    }

    /// Execute a POST request
    pub async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, WorkspaceError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request(Method::POST, path, Some(body), 1).await
    }

    /// Execute a PUT request
    pub async fn put<T, B>(&self, path: &str, body: &B) -> Result<T, WorkspaceError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request(Method::PUT, path, Some(body), 1).await
    }

    /// Execute a PATCH request
    pub async fn patch<T, B>(&self, path: &str, body: &B) -> Result<T, WorkspaceError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request(Method::PATCH, path, Some(body), 1).await
    }

    /// Execute a DELETE request
    pub async fn delete(&self, path: &str) -> Result<(), WorkspaceError> {
        let _: serde_json::Value = self.request_no_body(Method::DELETE, path, 1).await?;
        Ok(())
    }

    /// Execute a request without body (GET, DELETE)
    async fn request_no_body<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        cost: u32,
    ) -> Result<T, WorkspaceError> {
        // Acquire rate limit
        let _permit: Option<ConcurrencyPermit> = if let Some(ref limiter) = self.rate_limiter {
            limiter.acquire(cost).await.ok().flatten()
        } else {
            None
        };

        let url = self.build_url(path);

        // Create the request closure for retry
        let make_request = || async {
            // Get fresh token for each attempt (in case it expires during retries)
            let token = self.get_token().await?;

            let builder = self.http.request(method.clone(), &url)
                .bearer_auth(&token);

            let response = builder.send().await?;
            self.handle_response(response).await
        };

        // Execute with retry
        let result = with_retry(self.retry_config.clone(), make_request).await;

        match result {
            Ok(value) => Ok(value),
            Err(RetryError::NonRetryable(e)) => Err(e),
            Err(RetryError::MaxRetriesExceeded { last_error, .. }) => Err(last_error),
        }
    }

    /// Execute a request with body and rate limiting and retry
    async fn request<T, B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        cost: u32,
    ) -> Result<T, WorkspaceError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        // Acquire rate limit
        let _permit: Option<ConcurrencyPermit> = if let Some(ref limiter) = self.rate_limiter {
            limiter.acquire(cost).await.ok().flatten()
        } else {
            None
        };

        let url = self.build_url(path);

        // Create the request closure for retry
        let make_request = || async {
            // Get fresh token for each attempt (in case it expires during retries)
            let token = self.get_token().await?;

            let mut builder = self.http.request(method.clone(), &url)
                .bearer_auth(&token);

            if let Some(b) = body {
                builder = builder.json(b);
            }

            let response = builder.send().await?;
            self.handle_response(response).await
        };

        // Execute with retry
        let result = with_retry(self.retry_config.clone(), make_request).await;

        match result {
            Ok(value) => Ok(value),
            Err(RetryError::NonRetryable(e)) => Err(e),
            Err(RetryError::MaxRetriesExceeded { last_error, .. }) => Err(last_error),
        }
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, WorkspaceError> {
        let status = response.status();

        if status.is_success() {
            response.json().await.map_err(WorkspaceError::from)
        } else {
            let retry_after = response.headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(parse_retry_after)
                .map(|d| d.as_secs());

            let error_body: serde_json::Value = response.json().await.unwrap_or_default();
            let message = error_body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            Err(WorkspaceError::Api(ApiError {
                code: status.as_u16(),
                message,
                domain: "api".to_string(),
                retry_after,
            }))
        }
    }
}

/// Implement Retryable for WorkspaceError
impl Retryable for WorkspaceError {
    fn is_retryable(&self) -> bool {
        match self {
            WorkspaceError::Api(api_err) => is_retryable_status(api_err.code),
            WorkspaceError::Network(_) => true,
            _ => false,
        }
    }

    fn retry_after(&self) -> Option<Duration> {
        match self {
            WorkspaceError::Api(api_err) => api_err.retry_after.map(Duration::from_secs),
            _ => None,
        }
    }
}
