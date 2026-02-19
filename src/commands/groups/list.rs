use crate::client::ApiClient;
use crate::error::Result;
use super::types::DirectoryGroupsResponse;

pub struct ListGroupsParams {
    pub email: Option<String>,
    pub domain: Option<String>,
    pub page_size: u32,
    pub page_token: Option<String>,
}

pub async fn list_groups(client: &ApiClient, params: ListGroupsParams) -> Result<DirectoryGroupsResponse> {
    let mut query: Vec<(&str, String)> = vec![
        ("maxResults", params.page_size.to_string()),
    ];
    if let Some(ref email) = params.email {
        query.push(("userKey", email.clone()));
    }
    if let Some(ref domain) = params.domain {
        query.push(("domain", domain.clone()));
    }
    if let Some(ref token) = params.page_token {
        query.push(("pageToken", token.clone()));
    }
    client.get_with_query("/groups", &query).await
}
