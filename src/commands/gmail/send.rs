use crate::client::ApiClient;
use crate::error::Result;
use crate::utils::base64::encode_base64url_string;
use super::types::Message;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};

pub struct ComposeParams {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub from: Option<String>,
    pub cc: Option<String>,
    /// Message-ID of parent message (for replies)
    pub in_reply_to: Option<String>,
    /// Space-separated Message-IDs of thread history (for replies)
    pub references: Option<String>,
    /// Gmail thread ID (for replies)
    pub thread_id: Option<String>,
}

pub async fn send_message(client: &ApiClient, params: ComposeParams) -> Result<Message> {
    let raw_email = build_raw_email(&params);
    let encoded = encode_base64url_string(&raw_email);

    // Include threadId for replies to maintain threading
    let request = if let Some(ref thread_id) = params.thread_id {
        serde_json::json!({
            "raw": encoded,
            "threadId": thread_id
        })
    } else {
        serde_json::json!({ "raw": encoded })
    };

    client.post("/users/me/messages/send", &request).await
}

pub async fn create_draft(client: &ApiClient, params: ComposeParams) -> Result<serde_json::Value> {
    let raw_email = build_raw_email(&params);
    let encoded = encode_base64url_string(&raw_email);

    // Include threadId for reply drafts to maintain threading
    let message = if let Some(ref thread_id) = params.thread_id {
        serde_json::json!({
            "raw": encoded,
            "threadId": thread_id
        })
    } else {
        serde_json::json!({ "raw": encoded })
    };

    let request = serde_json::json!({ "message": message });
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

    // Add In-Reply-To header for replies (RFC 5322)
    if let Some(ref in_reply_to) = params.in_reply_to {
        email.push_str(&format!("In-Reply-To: {}\r\n", sanitize_header(in_reply_to)));
    }

    // Add References header for threading (RFC 5322)
    if let Some(ref references) = params.references {
        email.push_str(&format!("References: {}\r\n", sanitize_header(references)));
    }

    // Sanitize and add Subject header (RFC 2047 encode if non-ASCII)
    let subject = sanitize_header(&params.subject);
    if subject.is_ascii() {
        email.push_str(&format!("Subject: {}\r\n", subject));
    } else {
        // RFC 2047 encoded-word: =?charset?encoding?encoded_text?=
        let encoded = BASE64_STANDARD.encode(subject.as_bytes());
        email.push_str(&format!("Subject: =?UTF-8?B?{}?=\r\n", encoded));
    }

    // Send multipart/alternative: a text/plain part for clients that want it, and
    // a text/html part that Gmail actually renders.
    //
    // Why not text/plain alone: Gmail displays a text/plain body in its
    // fixed-width reading and compose view, which hard-wraps the text around 80
    // columns. The stored message is fine (paragraphs stay on one long line), but
    // the moment you open the draft in Gmail's composer those visual breaks get
    // baked into the message you send, so a normal paragraph goes out looking
    // like a ragged column. Offering an HTML alternative makes Gmail treat it as
    // an ordinary rich-text mail and reflow it to the window.
    let boundary = format!("----=_workspace_cli_{}", uuid::Uuid::new_v4().simple());
    email.push_str("MIME-Version: 1.0\r\n");
    email.push_str(&format!(
        "Content-Type: multipart/alternative; boundary=\"{}\"\r\n",
        boundary
    ));
    email.push_str("\r\n");

    // Plain-text alternative, body verbatim.
    email.push_str(&format!("--{}\r\n", boundary));
    email.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    email.push_str("\r\n");
    email.push_str(&normalize_newlines(&params.body));
    email.push_str("\r\n");

    // HTML alternative.
    email.push_str(&format!("--{}\r\n", boundary));
    email.push_str("Content-Type: text/html; charset=utf-8\r\n");
    email.push_str("\r\n");
    email.push_str(&body_to_html(&params.body));
    email.push_str("\r\n");

    email.push_str(&format!("--{}--\r\n", boundary));

    email
}

/// CRLF line endings, as RFC 5322 requires for transport.
fn normalize_newlines(body: &str) -> String {
    body.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\r\n")
}

/// Minimal, deliberately boring plain-text to HTML conversion.
///
/// A blank line starts a new paragraph; a single newline inside a paragraph
/// becomes a <br>. Everything is escaped, so this never emits markup the caller
/// did not ask for. No inline styles: the aim is an email that looks like one
/// typed in Gmail, not a designed template.
fn body_to_html(body: &str) -> String {
    let escape = |s: &str| -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };

    let normalized = body.replace("\r\n", "\n").replace('\r', "\n");
    let paragraphs: Vec<String> = normalized
        .split("\n\n")
        .map(|para| para.trim_matches('\n'))
        .filter(|para| !para.trim().is_empty())
        .map(|para| {
            let lines: Vec<String> = para.split('\n').map(|l| escape(l)).collect();
            format!("<div>{}</div>", lines.join("<br>"))
        })
        .collect();

    format!(
        "<html><body>{}</body></html>",
        paragraphs.join("<div><br></div>")
    )
}

/// Metadata extracted from an original message for constructing a reply
pub struct ReplyMetadata {
    /// Recipient for the reply (Reply-To or From of original)
    pub to: String,
    /// Cc recipients for reply-all (original To + Cc)
    pub cc: Option<String>,
    /// Subject with "Re: " prefix if not already present
    pub subject: String,
    /// Message-ID of the original message
    pub in_reply_to: String,
    /// Thread history (original References + Message-ID)
    pub references: String,
    /// Gmail thread ID
    pub thread_id: String,
}

/// Extract metadata from an original message needed to construct a reply
pub fn extract_reply_metadata(message: &Message) -> Option<ReplyMetadata> {
    let payload = message.payload.as_ref()?;
    let headers = &payload.headers;

    // Helper to get header value by name (case-insensitive)
    let get = |name: &str| -> Option<String> {
        headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.clone())
    };

    // Get reply-to address (prefer Reply-To, fall back to From)
    let to = get("Reply-To").or_else(|| get("From"))?;

    // Get subject, add "Re: " prefix if not already present
    let original_subject = get("Subject").unwrap_or_default();
    let subject = if original_subject.to_lowercase().starts_with("re:") {
        original_subject
    } else {
        format!("Re: {}", original_subject)
    };

    // Get Message-ID for In-Reply-To header
    let message_id = get("Message-ID")?;

    // Build References: original References (if any) + Message-ID
    let references = match get("References") {
        Some(refs) => format!("{} {}", refs, message_id),
        None => message_id.clone(),
    };

    // For reply-all: combine original To and Cc recipients
    let original_to = get("To");
    let original_cc = get("Cc");
    let cc = match (original_to, original_cc) {
        (Some(t), Some(c)) => Some(format!("{}, {}", t, c)),
        (Some(t), None) => Some(t),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    Some(ReplyMetadata {
        to,
        cc,
        subject,
        in_reply_to: message_id,
        references,
        thread_id: message.thread_id.clone(),
    })
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
