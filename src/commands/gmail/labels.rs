use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub r#type: String,
    pub message_list_visibility: Option<String>,
    pub label_list_visibility: Option<String>,
    pub messages_total: Option<i64>,
    pub messages_unread: Option<i64>,
    pub threads_total: Option<i64>,
    pub threads_unread: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLabelsResponse {
    #[serde(default)]
    pub labels: Vec<Label>,
}

/// List all labels in the user's mailbox
pub async fn list_labels(client: &ApiClient) -> Result<ListLabelsResponse> {
    client.get("/users/me/labels").await
}

/// Get a specific label by ID
pub async fn get_label(client: &ApiClient, label_id: &str) -> Result<Label> {
    let path = format!("/users/me/labels/{}", urlencoding::encode(label_id));
    client.get(&path).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyLabelsRequest {
    pub add_label_ids: Vec<String>,
    pub remove_label_ids: Vec<String>,
}

/// Modify labels on a message (add and/or remove labels)
pub async fn modify_labels(
    client: &ApiClient,
    message_id: &str,
    add_labels: Vec<String>,
    remove_labels: Vec<String>,
) -> Result<super::types::Message> {
    let path = format!("/users/me/messages/{}/modify", urlencoding::encode(message_id));
    let request = ModifyLabelsRequest {
        add_label_ids: add_labels,
        remove_label_ids: remove_labels,
    };
    client.post(&path, &request).await
}

/// Add labels to a message
pub async fn add_labels(
    client: &ApiClient,
    message_id: &str,
    label_ids: Vec<String>,
) -> Result<super::types::Message> {
    modify_labels(client, message_id, label_ids, vec![]).await
}

/// Remove labels from a message
pub async fn remove_labels(
    client: &ApiClient,
    message_id: &str,
    label_ids: Vec<String>,
) -> Result<super::types::Message> {
    modify_labels(client, message_id, vec![], label_ids).await
}
