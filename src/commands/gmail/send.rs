use crate::client::ApiClient;
use crate::error::Result;
use crate::utils::base64::encode_base64url_string;
use super::types::{Message, SendMessageRequest};

pub struct ComposeParams {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub from: Option<String>,
    pub cc: Option<String>,
}

pub async fn send_message(client: &ApiClient, params: ComposeParams) -> Result<Message> {
    let raw_email = build_raw_email(&params);
    let encoded = encode_base64url_string(&raw_email);

    let request = SendMessageRequest { raw: encoded };
    client.post("/users/me/messages/send", &request).await
}

pub async fn create_draft(client: &ApiClient, params: ComposeParams) -> Result<serde_json::Value> {
    let raw_email = build_raw_email(&params);
    let encoded = encode_base64url_string(&raw_email);

    let request = serde_json::json!({
        "message": {
            "raw": encoded
        }
    });
    client.post("/users/me/drafts", &request).await
}

fn build_raw_email(params: &ComposeParams) -> String {
    let mut email = String::new();

    // Add Date header (RFC 2822 requirement)
    let now = chrono::Utc::now();
    email.push_str(&format!("Date: {}\r\n", now.to_rfc2822()));

    // Add Message-ID header (RFC 2822 recommendation)
    let message_id = format!("<{}.{}@workspace-cli>",
        now.timestamp(),
        uuid::Uuid::new_v4());
    email.push_str(&format!("Message-ID: {}\r\n", message_id));

    // Sanitize and add From header
    if let Some(ref from) = params.from {
        email.push_str(&format!("From: {}\r\n", sanitize_header(from)));
    }

    // Sanitize and add To header
    email.push_str(&format!("To: {}\r\n", sanitize_header(&params.to)));

    // Sanitize and add Cc header if present
    if let Some(ref cc) = params.cc {
        email.push_str(&format!("Cc: {}\r\n", sanitize_header(cc)));
    }

    // Sanitize and add Subject header
    email.push_str(&format!("Subject: {}\r\n", sanitize_header(&params.subject)));

    email.push_str("MIME-Version: 1.0\r\n");
    email.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    email.push_str("\r\n");
    email.push_str(&params.body);

    email
}

/// Sanitize header values to prevent header injection attacks
/// Removes or replaces CR, LF, and CRLF sequences
fn sanitize_header(value: &str) -> String {
    value
        .replace('\r', "")
        .replace('\n', " ")
        .trim()
        .to_string()
}
