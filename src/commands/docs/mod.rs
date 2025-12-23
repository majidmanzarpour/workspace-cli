pub mod types;
pub mod get;
pub mod update;
pub mod create;

// Re-export commonly used types
pub use types::{
    Document,
    Body,
    StructuralElement,
    Paragraph,
    ParagraphElement,
    TextRun,
    TextStyle,
    BatchUpdateRequest,
    BatchUpdateResponse,
};

// Re-export key functions
pub use get::{
    get_document,
    document_to_markdown,
    document_to_text,
};

pub use update::{
    append_text,
    insert_text,
    replace_text,
};

pub use create::create_document;
