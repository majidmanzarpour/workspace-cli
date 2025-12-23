pub mod types;
pub mod list;
pub mod get;
pub mod send;

// Re-export main types and functions for convenience
pub use types::{
    Message,
    MessagePayload,
    MessagePart,
    MessageBody,
    Header,
    ListMessagesResponse,
    MessageRef,
    SendMessageRequest,
};

pub use list::{list_messages, ListParams};
pub use get::{get_message, extract_body, get_header};
pub use send::{send_message, create_draft, ComposeParams};
