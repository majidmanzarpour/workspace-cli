use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: Option<EventDateTime>,
    pub end: Option<EventDateTime>,
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<Attendee>,
    pub organizer: Option<Organizer>,
    pub html_link: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub recurrence: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDateTime {
    pub date: Option<String>,      // For all-day events
    pub date_time: Option<String>, // For timed events (RFC3339)
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendee {
    pub email: String,
    #[serde(default)]
    pub optional: bool,
    pub response_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organizer {
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(rename = "self")]
    pub is_self: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventList {
    #[serde(default)]
    pub items: Vec<Event>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
    pub summary: Option<String>,
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarList {
    #[serde(default)]
    pub items: Vec<CalendarListEntry>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListEntry {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub primary: Option<bool>,
    pub access_role: Option<String>,
}
