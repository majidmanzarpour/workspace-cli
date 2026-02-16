pub mod gmail;
pub mod drive;
pub mod calendar;
pub mod docs;
pub mod sheets;
pub mod slides;
pub mod tasks;
pub mod batch;
pub mod chat;
pub mod contacts;
pub mod groups;

// Re-export commonly used types
pub use gmail::types as gmail_types;
pub use drive::types as drive_types;
pub use calendar::types as calendar_types;
pub use docs::types as docs_types;
pub use sheets::types as sheets_types;
pub use slides::types as slides_types;
pub use tasks::types as tasks_types;
pub use chat::types as chat_types;
pub use contacts::types as contacts_types;
pub use groups::types as groups_types;
