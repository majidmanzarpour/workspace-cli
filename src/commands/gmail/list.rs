use crate::client::ApiClient;
use crate::error::Result;
use super::types::ListMessagesResponse;

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
