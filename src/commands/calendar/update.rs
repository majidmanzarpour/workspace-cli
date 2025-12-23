use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Event, EventDateTime};

pub struct UpdateEventParams {
    pub calendar_id: String,
    pub event_id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub time_zone: Option<String>,
}

pub async fn update_event(client: &ApiClient, params: UpdateEventParams) -> Result<Event> {
    // First, get the existing event
    let path = format!(
        "/calendars/{}/events/{}",
        urlencoding::encode(&params.calendar_id),
        urlencoding::encode(&params.event_id)
    );

    let mut event: Event = client.get(&path).await?;

    // Update fields
    if let Some(summary) = params.summary {
        event.summary = Some(summary);
    }
    if let Some(description) = params.description {
        event.description = Some(description);
    }
    if let Some(location) = params.location {
        event.location = Some(location);
    }
    if let Some(start) = params.start {
        let is_all_day = !start.contains('T');
        event.start = Some(EventDateTime {
            date: if is_all_day { Some(start.clone()) } else { None },
            date_time: if is_all_day { None } else { Some(start) },
            time_zone: if is_all_day { None } else { params.time_zone.clone() },
        });
    }
    if let Some(end) = params.end {
        let is_all_day = !end.contains('T');
        event.end = Some(EventDateTime {
            date: if is_all_day { Some(end.clone()) } else { None },
            date_time: if is_all_day { None } else { Some(end) },
            time_zone: if is_all_day { None } else { params.time_zone.clone() },
        });
    }

    client.put(&path, &event).await
}
