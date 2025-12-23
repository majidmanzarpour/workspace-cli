use crate::client::ApiClient;
use crate::error::Result;

/// Permanently delete a message (bypasses trash)
pub async fn delete_message(client: &ApiClient, message_id: &str) -> Result<()> {
    let path = format!("/users/me/messages/{}", urlencoding::encode(message_id));
    client.delete(&path).await
}

/// Batch delete multiple messages
pub async fn batch_delete(client: &ApiClient, message_ids: &[String]) -> Result<()> {
    #[derive(serde::Serialize)]
    struct BatchDeleteRequest {
        ids: Vec<String>,
    }

    let request = BatchDeleteRequest {
        ids: message_ids.to_vec(),
    };

    let _: serde_json::Value = client.post("/users/me/messages/batchDelete", &request).await?;
    Ok(())
}
