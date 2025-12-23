pub mod types;
pub mod get;

// Re-export commonly used types
pub use types::{
    Presentation,
    Page,
    PageElement,
    Shape,
    Table,
    TextContent,
    TextElement,
};

// Re-export key functions
pub use get::{
    get_presentation,
    get_page,
    extract_all_text,
    extract_page_text,
    get_slide_text,
    get_summary,
};
