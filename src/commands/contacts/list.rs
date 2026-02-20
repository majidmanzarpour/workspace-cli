use crate::client::ApiClient;
use crate::error::Result;
use super::types::{ConnectionsResponse, Person, READ_MASK};

pub struct ListContactsParams {
    pub page_size: u32,
    pub page_token: Option<String>,
}

pub async fn list_contacts(client: &ApiClient, params: ListContactsParams) -> Result<ConnectionsResponse> {
    let mut query_params: Vec<(&str, String)> = vec![
        ("pageSize", params.page_size.to_string()),
        ("personFields", READ_MASK.to_string()),
    ];
    if let Some(ref token) = params.page_token {
        query_params.push(("pageToken", token.clone()));
    }
    client.get_with_query("/people/me/connections", &query_params).await
}

pub async fn get_contact(client: &ApiClient, resource_name: &str) -> Result<Person> {
    let name = if resource_name.starts_with("people/") {
        resource_name.to_string()
    } else {
        format!("people/{}", resource_name)
    };
    let query_params = vec![
        ("personFields", READ_MASK.to_string()),
    ];
    let path = format!("/{}", name);
    client.get_with_query(&path, &query_params).await
}
