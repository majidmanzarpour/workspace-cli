use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Response from Admin Reports API activities.list
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitiesResponse {
    #[serde(default)]
    pub items: Vec<ActivityItem>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub id: Option<ActivityId>,
    pub actor: Option<ActivityActor>,
    pub ip_address: Option<String>,
    #[serde(default)]
    pub events: Vec<ActivityEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityId {
    pub time: Option<String>,
    pub unique_qualifier: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityActor {
    pub email: Option<String>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEvent {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub parameters: Vec<EventParameter>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventParameter {
    pub name: Option<String>,
    pub value: Option<String>,
    pub multi_value: Option<Vec<String>>,
    pub bool_value: Option<bool>,
    pub int_value: Option<i64>,
}

/// Flattened view event for output
#[derive(Debug, Serialize)]
pub struct FlatViewEvent {
    pub time: String,
    pub actor_email: String,
    pub event_name: String,
    pub doc_id: String,
    pub doc_title: String,
    pub doc_type: String,
    pub owner: String,
}

pub struct DriveActivityParams {
    pub event_name: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub filters: Option<String>,
    pub max_results: u32,
}

/// Fetch all drive activity events, handling pagination automatically.
pub async fn list_drive_activity(
    client: &ApiClient,
    params: DriveActivityParams,
) -> Result<Vec<FlatViewEvent>> {
    let mut all_events = Vec::new();
    let mut page_token: Option<String> = None;
    let mut page_count = 0u32;

    loop {
        let mut query: Vec<(&str, String)> = vec![
            ("applicationName", "drive".to_string()),
            ("eventName", params.event_name.clone()),
            ("maxResults", params.max_results.to_string()),
        ];
        if let Some(ref start) = params.start_time {
            query.push(("startTime", start.clone()));
        }
        if let Some(ref end) = params.end_time {
            query.push(("endTime", end.clone()));
        }
        if let Some(ref f) = params.filters {
            query.push(("filters", f.clone()));
        }
        if let Some(ref token) = page_token {
            query.push(("pageToken", token.clone()));
        }

        let response: ActivitiesResponse = client
            .get_with_query("/activity/users/all/applications/drive", &query)
            .await?;

        // Flatten each item's events into FlatViewEvent
        for item in &response.items {
            let actor_email = item.actor.as_ref()
                .and_then(|a| a.email.clone())
                .unwrap_or_default();
            let time = item.id.as_ref()
                .and_then(|id| id.time.clone())
                .unwrap_or_default();

            for event in &item.events {
                let event_name = event.name.clone().unwrap_or_default();
                let mut doc_id = String::new();
                let mut doc_title = String::new();
                let mut doc_type = String::new();
                let mut owner = String::new();

                for p in &event.parameters {
                    match p.name.as_deref() {
                        Some("doc_id") => doc_id = p.value.clone().unwrap_or_default(),
                        Some("doc_title") => doc_title = p.value.clone().unwrap_or_default(),
                        Some("doc_type") => doc_type = p.value.clone().unwrap_or_default(),
                        Some("owner") => owner = p.value.clone().unwrap_or_default(),
                        _ => {}
                    }
                }

                all_events.push(FlatViewEvent {
                    time: time.clone(),
                    actor_email: actor_email.clone(),
                    event_name,
                    doc_id,
                    doc_title,
                    doc_type,
                    owner,
                });
            }
        }

        page_count += 1;
        eprint!("\r  Page {}: {} events so far...", page_count, all_events.len());

        match response.next_page_token {
            Some(token) if !token.is_empty() => page_token = Some(token),
            _ => break,
        }
    }
    eprintln!(); // newline after progress

    Ok(all_events)
}
