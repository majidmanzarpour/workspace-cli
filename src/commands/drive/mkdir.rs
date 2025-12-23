use crate::client::ApiClient;
use crate::error::Result;
use super::types::File;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateFolderRequest {
    name: String,
    mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parents: Option<Vec<String>>,
}

/// Create a new folder
pub async fn create_folder(
    client: &ApiClient,
    name: &str,
    parent_id: Option<&str>,
) -> Result<File> {
    let request = CreateFolderRequest {
        name: name.to_string(),
        mime_type: "application/vnd.google-apps.folder".to_string(),
        parents: parent_id.map(|id| vec![id.to_string()]),
    };

    client.post("/files", &request).await
}
