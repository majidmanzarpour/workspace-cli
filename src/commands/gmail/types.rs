use serde::{Deserialize, Serialize};
use crate::output::pagination::Timestamped;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    #[serde(default)]
    pub label_ids: Vec<String>,
    #[serde(default)]
    pub snippet: String,
    pub payload: Option<MessagePayload>,
    pub internal_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePayload {
    pub headers: Vec<Header>,
    pub mime_type: Option<String>,
    pub body: Option<MessageBody>,
    #[serde(default)]
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageBody {
    pub data: Option<String>,
    pub size: Option<i64>,
    pub attachment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePart {
    #[serde(default)]
    pub headers: Vec<Header>,
    pub mime_type: Option<String>,
    pub body: Option<MessageBody>,
    #[serde(default)]
    pub parts: Vec<MessagePart>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMessagesResponse {
    #[serde(default)]
    pub messages: Vec<MessageRef>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRef {
    pub id: String,
    pub thread_id: String,
}

/// Enriched message summary with headers (used by list with metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSummary {
    pub id: String,
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Enriched list response with message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedListResponse {
    #[serde(default)]
    pub messages: Vec<MessageSummary>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u64>,
}

/// Minimal message format optimized for AI agents (reduced token usage)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimalMessage {
    pub id: String,
    pub thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// Response for label modification operations (minimal token usage)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyResponse {
    pub success: bool,
    pub id: String,
    pub labels: Vec<String>,
}

impl ModifyResponse {
    pub fn from_message(message: &Message) -> Self {
        Self {
            success: true,
            id: message.id.clone(),
            labels: message.label_ids.clone(),
        }
    }
}

/// Response for send/reply operations (minimal token usage)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResponse {
    pub success: bool,
    pub id: String,
    pub thread_id: String,
}

impl SendResponse {
    pub fn from_message(message: &Message) -> Self {
        Self {
            success: true,
            id: message.id.clone(),
            thread_id: message.thread_id.clone(),
        }
    }
}

/// Response for draft operations (minimal token usage)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftResponse {
    pub success: bool,
    pub id: String,
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
}

impl Timestamped for MessageSummary {
    fn timestamp(&self) -> Option<&str> {
        self.date.as_deref()
    }
}

// For sending emails
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    pub raw: String,
}
