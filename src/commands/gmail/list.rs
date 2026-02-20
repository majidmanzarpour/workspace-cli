use crate::client::ApiClient;
use crate::client::batch::{BatchClient, BatchRequest};
use crate::error::Result;
use super::types::{ListMessagesResponse, MessageRef, MessageSummary, EnrichedListResponse};

pub struct ListParams {
    pub query: Option<String>,
    pub max_results: u32,
    pub label_ids: Option<Vec<String>>,
    pub page_token: Option<String>,
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            query: None,
            max_results: 20,
            label_ids: None,
            page_token: None,
        }
    }
}

pub async fn list_messages(client: &ApiClient, params: ListParams) -> Result<ListMessagesResponse> {
    let mut query_params = vec![
        ("maxResults", params.max_results.to_string()),
    ];

    if let Some(ref q) = params.query {
        query_params.push(("q", q.clone()));
    }
    if let Some(ref token) = params.page_token {
        query_params.push(("pageToken", token.clone()));
    }
    if let Some(ref labels) = params.label_ids {
        for label in labels {
            query_params.push(("labelIds", label.clone()));
        }
    }

    client.get_with_query("/users/me/messages", &query_params).await
}

/// Fetch metadata (Subject, From, Date) for messages using batch request
pub async fn enrich_messages(
    message_refs: Vec<MessageRef>,
    access_token: &str,
) -> Result<Vec<MessageSummary>> {
    if message_refs.is_empty() {
        return Ok(vec![]);
    }

    // Build batch requests for metadata
    let requests: Vec<BatchRequest> = message_refs.iter()
        .map(|msg| BatchRequest::get(
            &msg.id,
            format!("/gmail/v1/users/me/messages/{}?format=metadata&metadataHeaders=Subject&metadataHeaders=From&metadataHeaders=Date", msg.id)
        ))
        .collect();

    let client = BatchClient::gmail();
    let responses = client.execute(requests, access_token).await
        .map_err(|e| crate::error::WorkspaceError::Config(format!("Batch request failed: {}", e)))?;

    // Parse responses into MessageSummary
    let mut summaries = Vec::new();
    for resp in responses {
        let id = resp.body.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&resp.id)
            .to_string();
        let thread_id = resp.body.get("threadId")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();

        // Extract headers
        let headers = resp.body
            .get("payload")
            .and_then(|p| p.get("headers"))
            .and_then(|h| h.as_array());

        let get_header = |name: &str| -> Option<String> {
            headers?.iter()
                .find(|h| h.get("name").and_then(|n| n.as_str()) == Some(name))
                .and_then(|h| h.get("value"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        };

        let snippet = resp.body.get("snippet")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        summaries.push(MessageSummary {
            id,
            thread_id,
            subject: get_header("Subject"),
            from: get_header("From"),
            date: get_header("Date"),
            snippet,
        });
    }

    Ok(summaries)
}

/// List messages with enriched metadata (Subject, From, Date, snippet)
/// Uses batch request to fetch metadata in a single HTTP call
pub async fn list_messages_with_metadata(
    client: &ApiClient,
    params: ListParams,
    access_token: &str,
) -> Result<EnrichedListResponse> {
    // First, get message IDs
    let list_response = list_messages(client, params).await?;

    // Then fetch metadata via batch
    let summaries = enrich_messages(list_response.messages, access_token).await?;

    Ok(EnrichedListResponse {
        messages: summaries,
        next_page_token: list_response.next_page_token,
        result_size_estimate: list_response.result_size_estimate,
    })
}
