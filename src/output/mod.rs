mod formatter;
pub mod pagination;

pub use formatter::{Formatter, OutputFormat, output_json, output_jsonl, output_toon};
pub use pagination::{
    PageConfig, PagedResponse, PaginatedResult, PaginationState, Paginator,
    Timestamped, collect_all_pages, paginate_stream,
};
