use crate::client::ApiClient;
use crate::error::Result;
use super::types::{EventList, CalendarList};

pub struct ListEventsParams {
    pub calendar_id: String,
    pub time_min: Option<String>,
    pub time_max: Option<String>,
    pub max_results: u32,
    pub single_events: bool,  // Expand recurring events
    pub order_by: Option<String>,
    pub page_token: Option<String>,
    pub sync_token: Option<String>,
}

impl Default for ListEventsParams {
    fn default() -> Self {
        Self {
            calendar_id: "primary".to_string(),
            time_min: None,
            time_max: None,
            max_results: 20,
            single_events: true,  // Default to expanded events for easier agent processing
            order_by: Some("startTime".to_string()),
            page_token: None,
            sync_token: None,
        }
    }
}

pub async fn list_events(client: &ApiClient, params: ListEventsParams) -> Result<EventList> {
    let mut query_params: Vec<(&str, String)> = vec![
        ("maxResults", params.max_results.to_string()),
        ("singleEvents", params.single_events.to_string()),
    ];

    // Sync token is mutually exclusive with timeMin, timeMax, and pageToken
    // When using syncToken, only maxResults and other query-independent params should be included
    if let Some(ref sync) = params.sync_token {
        query_params.push(("syncToken", sync.clone()));
        // Do NOT add timeMin, timeMax, or pageToken when using syncToken
    } else {
        // Normal query mode - can use all filtering parameters
        if let Some(ref time_min) = params.time_min {
            query_params.push(("timeMin", time_min.clone()));
        }
        if let Some(ref time_max) = params.time_max {
            query_params.push(("timeMax", time_max.clone()));
        }
        if let Some(ref token) = params.page_token {
            query_params.push(("pageToken", token.clone()));
        }
        if let Some(ref order) = params.order_by {
            if params.single_events {
                query_params.push(("orderBy", order.clone()));
            }
        }
    }

    let path = format!("/calendars/{}/events", urlencoding::encode(&params.calendar_id));
    client.get_with_query(&path, &query_params).await
}

pub async fn list_calendars(client: &ApiClient) -> Result<CalendarList> {
    client.get("/users/me/calendarList").await
}
