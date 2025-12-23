use crate::client::ApiClient;
use crate::error::Result;
use super::types::{BatchUpdateRequest, BatchUpdateResponse, Request, InsertTextRequest, Location, Document, ReplaceAllTextRequest, SubstringMatchCriteria};
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
                replace_all_text: None,
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
                replace_all_text: None,
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

/// Replace all occurrences of text in a document
pub async fn replace_text(
    client: &ApiClient,
    document_id: &str,
    find: &str,
    replace: &str,
    match_case: bool,
) -> Result<BatchUpdateResponse> {
    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: None,
                replace_all_text: Some(ReplaceAllTextRequest {
                    contains_text: SubstringMatchCriteria {
                        text: find.to_string(),
                        match_case,
                    },
                    replace_text: replace.to_string(),
                }),
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}
