use crate::client::ApiClient;
use crate::error::Result;
use super::types::Message;
use super::labels::modify_labels;

/// Mark a message as read (removes UNREAD label)
pub async fn mark_read(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec![], vec!["UNREAD".to_string()]).await
}

/// Mark a message as unread (adds UNREAD label)
pub async fn mark_unread(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec!["UNREAD".to_string()], vec![]).await
}

/// Star a message (adds STARRED label)
pub async fn star_message(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec!["STARRED".to_string()], vec![]).await
}

/// Unstar a message (removes STARRED label)
pub async fn unstar_message(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec![], vec!["STARRED".to_string()]).await
}

/// Mark a message as important (adds IMPORTANT label)
pub async fn mark_important(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec!["IMPORTANT".to_string()], vec![]).await
}

/// Mark a message as not important (removes IMPORTANT label)
pub async fn mark_not_important(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec![], vec!["IMPORTANT".to_string()]).await
}

/// Archive a message (removes INBOX label)
pub async fn archive_message(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec![], vec!["INBOX".to_string()]).await
}

/// Move a message to inbox (adds INBOX label)
pub async fn move_to_inbox(client: &ApiClient, message_id: &str) -> Result<Message> {
    modify_labels(client, message_id, vec!["INBOX".to_string()], vec![]).await
}
