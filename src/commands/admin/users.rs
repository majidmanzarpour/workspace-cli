use crate::client::ApiClient;
use crate::error::Result;
use super::types::{UsersListResponse, User};

pub struct ListUsersParams {
    pub domain: Option<String>,
    pub max_results: u32,
    pub page_token: Option<String>,
    pub query: Option<String>,
}

impl Default for ListUsersParams {
    fn default() -> Self {
        Self {
            domain: None,
            max_results: 100,
            page_token: None,
            query: None,
        }
    }
}

pub async fn list_users(client: &ApiClient, params: ListUsersParams) -> Result<UsersListResponse> {
    let mut query: Vec<(&str, String)> = vec![
        ("maxResults", params.max_results.to_string()),
    ];
    if let Some(ref domain) = params.domain {
        query.push(("domain", domain.clone()));
    }
    if let Some(ref token) = params.page_token {
        query.push(("pageToken", token.clone()));
    }
    if let Some(ref q) = params.query {
        query.push(("query", q.clone()));
    }
    client.get_with_query("/users", &query).await
}

pub async fn get_user(client: &ApiClient, user_key: &str) -> Result<User> {
    client.get(&format!("/users/{}", user_key)).await
}
