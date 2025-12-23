pub mod types;
pub mod list;
pub mod create;
pub mod update;
pub mod delete;

// Re-export commonly used types and functions
pub use types::{
    Event,
    EventDateTime,
    Attendee,
    Organizer,
    EventList,
    CalendarList,
    CalendarListEntry,
};

pub use list::{
    list_events,
    list_calendars,
    ListEventsParams,
};

pub use create::{
    create_event,
    CreateEventParams,
};

pub use update::{
    update_event,
    UpdateEventParams,
};

pub use delete::delete_event;
