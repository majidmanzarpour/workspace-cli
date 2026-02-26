use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Message, MessageListResponse, CreateMessageRequest, Thread};

pub struct ListMessagesParams {
    pub space_name: String,
    pub page_size: u32,
    pub page_token: Option<String>,
    pub order_by: Option<String>,
    pub filter: Option<String>,
}

impl ListMessagesParams {
    pub fn new(space_name: impl Into<String>) -> Self {
        Self {
            space_name: space_name.into(),
            page_size: 25,
            page_token: None,
            order_by: None,
            filter: None,
        }
    }
}

pub async fn list_messages(client: &ApiClient, params: ListMessagesParams) -> Result<MessageListResponse> {
    let space = if params.space_name.starts_with("spaces/") {
        params.space_name.clone()
    } else {
        format!("spaces/{}", params.space_name)
    };
    let mut query: Vec<(&str, String)> = vec![
        ("pageSize", params.page_size.to_string()),
    ];
    if let Some(ref token) = params.page_token {
        query.push(("pageToken", token.clone()));
    }
    if let Some(ref order) = params.order_by {
        query.push(("orderBy", order.clone()));
    }
    if let Some(ref f) = params.filter {
        query.push(("filter", f.clone()));
    }
    let path = format!("/{}/messages", space);
    client.get_with_query(&path, &query).await
}

pub async fn get_message(client: &ApiClient, message_name: &str) -> Result<Message> {
    let path = if message_name.starts_with("spaces/") {
        format!("/{}", message_name)
    } else {
        format!("/spaces/{}", message_name)
    };
    client.get(&path).await
}

pub async fn send_message(client: &ApiClient, space_name: &str, text: &str, thread_name: Option<&str>) -> Result<Message> {
    let space = if space_name.starts_with("spaces/") {
        space_name.to_string()
    } else {
        format!("spaces/{}", space_name)
    };
    let body = CreateMessageRequest {
        text: text.to_string(),
        thread: thread_name.map(|t| Thread { name: Some(t.to_string()) }),
    };
    let path = format!("/{}/messages", space);
    client.post(&path, &body).await
}
