use crate::client::ApiClient;
use crate::error::Result;
use crate::utils::base64::decode_base64url_string;
use super::types::{Message, MessagePart};

pub async fn get_message(client: &ApiClient, id: &str, format: &str) -> Result<Message> {
    let query = [("format", format)];
    client.get_with_query(&format!("/users/me/messages/{}", id), &query).await
}

/// Extract and decode the message body
pub fn extract_body(message: &Message) -> Option<String> {
    if let Some(ref payload) = message.payload {
        // Try to get body from payload directly
        if let Some(ref body) = payload.body {
            if let Some(ref data) = body.data {
                if !data.is_empty() {
                    if let Ok(decoded) = decode_base64url_string(data) {
                        return Some(decoded);
                    }
                }
            }
        }

        // Try to find text/plain or text/html in parts
        // Prefer text/plain over text/html for multipart/alternative
        if let Some(text) = find_text_part(&payload.parts, "text/plain") {
            return Some(text);
        }

        // Fallback to text/html if text/plain not found
        if let Some(html) = find_text_part(&payload.parts, "text/html") {
            return Some(html);
        }
    }
    None
}

fn find_text_part(parts: &[MessagePart], preferred_mime: &str) -> Option<String> {
    // Check if this is a multipart/alternative container
    for part in parts {
        let mime = part.mime_type.as_deref().unwrap_or("");

        // If we find multipart/alternative, search within it for the preferred type
        if mime == "multipart/alternative" {
            if let Some(text) = find_text_part(&part.parts, preferred_mime) {
                return Some(text);
            }
        }
    }

    // Look for the preferred MIME type in current level
    for part in parts {
        let mime = part.mime_type.as_deref().unwrap_or("");

        if mime == preferred_mime {
            if let Some(ref body) = part.body {
                if let Some(ref data) = body.data {
                    if !data.is_empty() {
                        if let Ok(decoded) = decode_base64url_string(data) {
                            return Some(decoded);
                        }
                    }
                }
            }
        }
    }

    // Recurse into nested parts that aren't multipart/alternative
    for part in parts {
        let mime = part.mime_type.as_deref().unwrap_or("");

        if mime.starts_with("multipart/") && mime != "multipart/alternative" {
            if let Some(text) = find_text_part(&part.parts, preferred_mime) {
                return Some(text);
            }
        }
    }

    None
}

/// Get header value by name
pub fn get_header(message: &Message, name: &str) -> Option<String> {
    message.payload.as_ref()?.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.clone())
}
