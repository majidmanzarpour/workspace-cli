use crate::client::ApiClient;
use crate::error::Result;
use super::types::Message;

/// Move a message to trash
pub async fn trash_message(client: &ApiClient, message_id: &str) -> Result<Message> {
    let path = format!("/users/me/messages/{}/trash", urlencoding::encode(message_id));
    client.post(&path, &serde_json::Value::Null).await
}

/// Remove a message from trash
pub async fn untrash_message(client: &ApiClient, message_id: &str) -> Result<Message> {
    let path = format!("/users/me/messages/{}/untrash", urlencoding::encode(message_id));
    client.post(&path, &serde_json::Value::Null).await
}
