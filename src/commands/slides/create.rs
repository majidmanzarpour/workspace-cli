use crate::client::ApiClient;
use crate::error::Result;
use super::types::Presentation;
use super::batch_types::{SlidesBatchUpdateRequest, SlidesBatchUpdateResponse, create_slide_request};

/// Create a new presentation
pub async fn create_presentation(client: &ApiClient, title: &str) -> Result<Presentation> {
    let body = serde_json::json!({"title": title});
    client.post("/presentations", &body).await
}

/// Add a slide to an existing presentation
pub async fn add_slide(
    client: &ApiClient,
    presentation_id: &str,
    object_id: &str,
    index: Option<u32>,
    layout: &str,
) -> Result<SlidesBatchUpdateResponse> {
    let request = SlidesBatchUpdateRequest {
        requests: vec![create_slide_request(object_id, index, layout)],
    };
    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}
