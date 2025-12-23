use crate::client::ApiClient;
use crate::error::Result;
use super::types::File;

/// Permanently delete a file (bypasses trash)
pub async fn delete_file(client: &ApiClient, file_id: &str) -> Result<()> {
    let path = format!("/files/{}", urlencoding::encode(file_id));
    client.delete(&path).await
}

/// Move a file to trash
pub async fn trash_file(client: &ApiClient, file_id: &str) -> Result<File> {
    let path = format!("/files/{}", urlencoding::encode(file_id));

    #[derive(serde::Serialize)]
    struct TrashRequest {
        trashed: bool,
    }

    client.patch(&path, &TrashRequest { trashed: true }).await
}

/// Restore a file from trash
pub async fn untrash_file(client: &ApiClient, file_id: &str) -> Result<File> {
    let path = format!("/files/{}", urlencoding::encode(file_id));

    #[derive(serde::Serialize)]
    struct UntrashRequest {
        trashed: bool,
    }

    client.patch(&path, &UntrashRequest { trashed: false }).await
}

/// Empty the trash (permanently delete all trashed files)
pub async fn empty_trash(client: &ApiClient) -> Result<()> {
    client.delete("/files/trash").await
}
