use crate::client::ApiClient;
use crate::error::Result;
use super::types::{BatchUpdateRequest, BatchUpdateResponse, Request, InsertTextRequest, Location, Document};
use super::get::get_document;

/// Append text to the end of a document
pub async fn append_text(
    client: &ApiClient,
    document_id: &str,
    text: &str,
) -> Result<BatchUpdateResponse> {
    // First get the document to find the end index
    let doc = get_document(client, document_id).await?;
    let end_index = get_end_index(&doc);

    // Insert at end (index - 1 because we insert before the trailing newline)
    let insert_index = (end_index - 1).max(1);

    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: Some(InsertTextRequest {
                    text: format!("\n{}", text),
                    location: Location {
                        index: insert_index,
                        segment_id: None,
                    },
                }),
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

/// Insert text at a specific index
pub async fn insert_text(
    client: &ApiClient,
    document_id: &str,
    text: &str,
    index: i64,
) -> Result<BatchUpdateResponse> {
    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: Some(InsertTextRequest {
                    text: text.to_string(),
                    location: Location {
                        index,
                        segment_id: None,
                    },
                }),
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

fn get_end_index(doc: &Document) -> i64 {
    doc.body
        .as_ref()
        .and_then(|b| b.content.last())
        .and_then(|e| e.end_index)
        .unwrap_or(1)
}
