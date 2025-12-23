use crate::client::ApiClient;
use crate::error::Result;
use super::types::File;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MoveRequest {
    // Empty - we use query params for move
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CopyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parents: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameRequest {
    name: String,
}

/// Move a file to a different folder
pub async fn move_file(
    client: &ApiClient,
    file_id: &str,
    new_parent_id: &str,
    remove_from_current: bool,
) -> Result<File> {
    // First, get the current parents if we need to remove them
    let mut path = format!("/files/{}?addParents={}",
        urlencoding::encode(file_id),
        urlencoding::encode(new_parent_id)
    );

    if remove_from_current {
        // Get current file to find existing parents
        let file: File = client.get(&format!("/files/{}?fields=parents", urlencoding::encode(file_id))).await?;
        if !file.parents.is_empty() {
            let remove_parents = file.parents.join(",");
            path = format!("{}&removeParents={}", path, urlencoding::encode(&remove_parents));
        }
    }

    client.patch(&path, &MoveRequest {}).await
}

/// Copy a file
pub async fn copy_file(
    client: &ApiClient,
    file_id: &str,
    new_name: Option<&str>,
    destination_parent: Option<&str>,
) -> Result<File> {
    let path = format!("/files/{}/copy", urlencoding::encode(file_id));

    let request = CopyRequest {
        name: new_name.map(|s| s.to_string()),
        parents: destination_parent.map(|p| vec![p.to_string()]),
    };

    client.post(&path, &request).await
}

/// Rename a file
pub async fn rename_file(
    client: &ApiClient,
    file_id: &str,
    new_name: &str,
) -> Result<File> {
    let path = format!("/files/{}", urlencoding::encode(file_id));

    let request = RenameRequest {
        name: new_name.to_string(),
    };

    client.patch(&path, &request).await
}
