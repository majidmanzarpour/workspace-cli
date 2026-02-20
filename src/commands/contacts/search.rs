use crate::client::ApiClient;
use crate::error::Result;
use super::types::{SearchResponse, DirectoryPeopleResponse};

const READ_MASK: &str = "names,emailAddresses,phoneNumbers,organizations,urls";

pub async fn search_contacts(client: &ApiClient, query: &str, page_size: u32) -> Result<SearchResponse> {
    let query_params = vec![
        ("query", query.to_string()),
        ("pageSize", page_size.to_string()),
        ("readMask", READ_MASK.to_string()),
    ];
    client.get_with_query("/people:searchContacts", &query_params).await
}

pub struct DirectoryListParams {
    pub page_size: u32,
    pub page_token: Option<String>,
}

pub async fn list_directory(
    client: &ApiClient,
    params: DirectoryListParams,
) -> Result<DirectoryPeopleResponse> {
    let page_size = params.page_size;
    let page_token = params.page_token;
    let mut query_params: Vec<(&str, String)> = vec![
        ("pageSize", page_size.to_string()),
        ("readMask", "names,emailAddresses".to_string()),
        ("sources", "DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE".to_string()),
    ];
    if let Some(token) = page_token {
        query_params.push(("pageToken", token.to_string()));
    }
    client.get_with_query("/people:listDirectoryPeople", &query_params).await
}

pub async fn search_directory(
    client: &ApiClient,
    query: &str,
    page_size: u32,
    page_token: Option<&str>,
) -> Result<DirectoryPeopleResponse> {
    let mut query_params: Vec<(&str, String)> = vec![
        ("query", query.to_string()),
        ("pageSize", page_size.to_string()),
        ("readMask", "names,emailAddresses".to_string()),
        ("sources", "DIRECTORY_SOURCE_TYPE_DOMAIN_PROFILE".to_string()),
    ];
    if let Some(token) = page_token {
        query_params.push(("pageToken", token.to_string()));
    }
    client.get_with_query("/people:searchDirectoryPeople", &query_params).await
}
