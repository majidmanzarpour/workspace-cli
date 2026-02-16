use crate::client::ApiClient;
use crate::error::Result;
use super::types::TransitiveGroupsResponse;

pub struct ListGroupsParams {
    pub email: String,
    pub page_size: u32,
    pub page_token: Option<String>,
}

pub async fn list_groups(client: &ApiClient, params: ListGroupsParams) -> Result<TransitiveGroupsResponse> {
    let query_str = format!(
        "member_key_id == '{}' && 'cloudidentity.googleapis.com/groups.discussion_forum' in labels",
        params.email
    );
    let mut query: Vec<(&str, String)> = vec![
        ("query", query_str),
        ("pageSize", params.page_size.to_string()),
    ];
    if let Some(ref token) = params.page_token {
        query.push(("pageToken", token.clone()));
    }
    client.get_with_query("/groups/-/memberships:searchTransitiveGroups", &query).await
}
