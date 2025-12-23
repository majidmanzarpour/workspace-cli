use crate::client::ApiClient;
use crate::error::Result;
use super::types::Document;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct CreateDocumentRequest {
    title: String,
}

/// Create a new Google Doc
pub async fn create_document(client: &ApiClient, title: &str) -> Result<Document> {
    let request = CreateDocumentRequest {
        title: title.to_string(),
    };

    client.post("/documents", &request).await
}
