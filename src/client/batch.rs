use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use std::time::Duration;
use uuid::Uuid;

/// Batch request endpoints
pub mod batch_endpoints {
    pub const GMAIL: &str = "https://gmail.googleapis.com/batch/gmail/v1";
    pub const DRIVE: &str = "https://www.googleapis.com/batch/drive/v3";
    pub const CALENDAR: &str = "https://www.googleapis.com/batch/calendar/v3";
    pub const CHAT: &str = "https://chat.googleapis.com/batch";
}

/// A single request in a batch
#[derive(Debug, Clone)]
pub struct BatchRequest {
    pub id: String,
    pub method: Method,
    pub path: String,
    pub body: Option<serde_json::Value>,
}

impl BatchRequest {
    pub fn get(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            method: Method::GET,
            path: path.into(),
            body: None,
        }
    }

    pub fn post(id: impl Into<String>, path: impl Into<String>, body: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            method: Method::POST,
            path: path.into(),
            body: Some(body),
        }
    }

    pub fn delete(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            method: Method::DELETE,
            path: path.into(),
            body: None,
        }
    }

    pub fn patch(id: impl Into<String>, path: impl Into<String>, body: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            method: Method::PATCH,
            path: path.into(),
            body: Some(body),
        }
    }
}

/// Response from a single request in a batch
#[derive(Debug, Clone)]
pub struct BatchResponse {
    pub id: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: serde_json::Value,
}

impl BatchResponse {
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Parse body into a specific type
    pub fn parse<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.body.clone())
    }
}

/// Batch request client
pub struct BatchClient {
    http: Client,
    endpoint: String,
    max_requests: usize,
}

impl BatchClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            endpoint: endpoint.into(),
            max_requests: 100, // Google's limit
        }
    }

    /// Create a Gmail batch client
    pub fn gmail() -> Self {
        Self::new(batch_endpoints::GMAIL)
    }

    /// Create a Drive batch client
    pub fn drive() -> Self {
        Self::new(batch_endpoints::DRIVE)
    }

    /// Create a Calendar batch client
    pub fn calendar() -> Self {
        Self::new(batch_endpoints::CALENDAR)
    }

    /// Create a Chat batch client
    pub fn chat() -> Self {
        Self::new(batch_endpoints::CHAT)
    }

    /// Execute a batch of requests
    pub async fn execute(
        &self,
        requests: Vec<BatchRequest>,
        access_token: &str,
    ) -> Result<Vec<BatchResponse>, BatchError> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        if requests.len() > self.max_requests {
            return Err(BatchError::TooManyRequests {
                count: requests.len(),
                max: self.max_requests,
            });
        }

        let boundary = format!("batch_{}", Uuid::new_v4().to_string().replace("-", ""));
        let body = self.build_multipart_body(&requests, &boundary);

        let response = self.http
            .post(&self.endpoint)
            .header("Content-Type", format!("multipart/mixed; boundary={}", boundary))
            .header("Authorization", format!("Bearer {}", access_token))
            .body(body)
            .send()
            .await
            .map_err(BatchError::Network)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(BatchError::HttpError { status, message: text });
        }

        // Get the response boundary from Content-Type header
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let response_boundary = extract_boundary(content_type)
            .ok_or_else(|| BatchError::InvalidResponse("Missing boundary in response".into()))?;

        let response_body = response.text().await.map_err(BatchError::Network)?;
        self.parse_multipart_response(&response_body, &response_boundary)
    }

    /// Build multipart/mixed request body
    fn build_multipart_body(&self, requests: &[BatchRequest], boundary: &str) -> String {
        let mut body = String::new();

        for req in requests {
            // Part boundary
            body.push_str(&format!("--{}\r\n", boundary));

            // Part headers
            body.push_str("Content-Type: application/http\r\n");
            body.push_str(&format!("Content-ID: <{}>\r\n", req.id));
            body.push_str("\r\n");

            // HTTP request line
            body.push_str(&format!("{} {} HTTP/1.1\r\n", req.method, req.path));

            // Request headers and body
            if let Some(ref json_body) = req.body {
                let json_str = serde_json::to_string(json_body).unwrap_or_default();
                body.push_str("Content-Type: application/json\r\n");
                body.push_str(&format!("Content-Length: {}\r\n", json_str.len()));
                body.push_str("\r\n");
                body.push_str(&json_str);
            } else {
                body.push_str("\r\n");
            }

            body.push_str("\r\n");
        }

        // End boundary
        body.push_str(&format!("--{}--\r\n", boundary));
        body
    }

    /// Parse multipart/mixed response
    fn parse_multipart_response(
        &self,
        body: &str,
        boundary: &str,
    ) -> Result<Vec<BatchResponse>, BatchError> {
        let mut responses = Vec::new();
        let parts: Vec<&str> = body.split(&format!("--{}", boundary)).collect();

        for part in parts {
            let part = part.trim();
            if part.is_empty() || part == "--" {
                continue;
            }

            if let Some(response) = self.parse_part(part)? {
                responses.push(response);
            }
        }

        Ok(responses)
    }

    /// Parse a single part of the multipart response
    fn parse_part(&self, part: &str) -> Result<Option<BatchResponse>, BatchError> {
        // Split headers and body
        let mut sections = part.splitn(2, "\r\n\r\n");
        let headers_section = sections.next().unwrap_or("");

        // Find Content-ID
        let id = headers_section.lines()
            .find(|l| l.to_lowercase().starts_with("content-id:"))
            .and_then(|l| {
                let id = l.split(':').nth(1)?.trim();
                // Remove < > brackets and Google's "response-" prefix if present
                let cleaned = id.trim_matches(|c| c == '<' || c == '>' || c == ' ');
                let cleaned = cleaned.strip_prefix("response-").unwrap_or(cleaned);
                Some(cleaned.to_string())
            })
            .unwrap_or_default();

        // Find the HTTP response within this part
        let rest = sections.next().unwrap_or("");

        // Look for HTTP status line
        let mut lines = rest.lines();
        let status_line = lines.next().unwrap_or("");

        if !status_line.starts_with("HTTP/") {
            return Ok(None);
        }

        // Parse status code
        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Parse headers until empty line
        let mut headers = Vec::new();
        let mut body_start = false;
        let mut body_lines = Vec::new();

        for line in lines {
            if body_start {
                body_lines.push(line);
            } else if line.is_empty() {
                body_start = true;
            } else if let Some((key, value)) = line.split_once(':') {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        // Parse body as JSON
        let body_str = body_lines.join("\n");
        let body: serde_json::Value = serde_json::from_str(&body_str).unwrap_or_default();

        Ok(Some(BatchResponse {
            id,
            status,
            headers,
            body,
        }))
    }
}

/// Extract boundary from Content-Type header
fn extract_boundary(content_type: &str) -> Option<String> {
    content_type
        .split(';')
        .find(|p| p.trim().starts_with("boundary="))
        .and_then(|p| {
            let boundary = p.split('=').nth(1)?.trim();
            // Remove quotes if present (both single and double)
            let unquoted = boundary.trim_matches(|c| c == '"' || c == '\'');
            Some(unquoted.to_string())
        })
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Too many requests: {count} (max: {max})")]
    TooManyRequests { count: usize, max: usize },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP error {status}: {message}")]
    HttpError { status: u16, message: String },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
