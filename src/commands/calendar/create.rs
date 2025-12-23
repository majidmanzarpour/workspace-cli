use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Event, EventDateTime, Attendee};

pub struct CreateEventParams {
    pub calendar_id: String,
    pub summary: String,
    pub start: String,  // RFC3339 or YYYY-MM-DD
    pub end: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub attendees: Option<Vec<String>>,
    pub time_zone: Option<String>,
}

pub async fn create_event(client: &ApiClient, params: CreateEventParams) -> Result<Event> {
    let is_all_day = !params.start.contains('T');

    let start = if is_all_day {
        EventDateTime {
            date: Some(params.start.clone()),
            date_time: None,
            time_zone: None,  // All-day events should not have timezone
        }
    } else {
        EventDateTime {
            date: None,
            date_time: Some(params.start.clone()),
            time_zone: params.time_zone.clone(),
        }
    };

    let end = if is_all_day {
        EventDateTime {
            date: Some(params.end.clone()),
            date_time: None,
            time_zone: None,  // All-day events should not have timezone
        }
    } else {
        EventDateTime {
            date: None,
            date_time: Some(params.end.clone()),
            time_zone: params.time_zone.clone(),
        }
    };

    let attendees: Vec<Attendee> = params.attendees
        .unwrap_or_default()
        .into_iter()
        .map(|email| Attendee {
            email,
            optional: false,
            response_status: None,
        })
        .collect();

    let event = Event {
        id: None,
        summary: Some(params.summary),
        description: params.description,
        location: params.location,
        start: Some(start),
        end: Some(end),
        status: None,
        attendees,
        organizer: None,
        html_link: None,
        created: None,
        updated: None,
        recurrence: None,
    };

    let path = format!("/calendars/{}/events", urlencoding::encode(&params.calendar_id));
    client.post(&path, &event).await
}
