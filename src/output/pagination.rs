use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Common pagination response wrapper for Google APIs
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    /// The items in this page
    #[serde(default)]
    pub items: Vec<T>,

    /// Alternative field name used by some APIs (e.g., Gmail uses "messages")
    #[serde(default)]
    pub messages: Vec<T>,

    /// Alternative field name (e.g., Drive uses "files")
    #[serde(default)]
    pub files: Vec<T>,

    /// Alternative field name (e.g., Calendar uses "events")
    #[serde(default)]
    pub events: Vec<T>,

    /// Token for the next page
    pub next_page_token: Option<String>,

    /// Sync token for incremental sync (Calendar)
    pub next_sync_token: Option<String>,

    /// Result size estimate
    pub result_size_estimate: Option<u64>,
}

impl<T> PagedResponse<T> {
    /// Get all items from the response, checking all possible field names
    pub fn into_items(self) -> Vec<T> {
        if !self.items.is_empty() {
            self.items
        } else if !self.messages.is_empty() {
            self.messages
        } else if !self.files.is_empty() {
            self.files
        } else if !self.events.is_empty() {
            self.events
        } else {
            Vec::new()
        }
    }

    /// Check if there are more pages
    pub fn has_more(&self) -> bool {
        self.next_page_token.is_some()
    }
}

/// Pagination state for iterating through pages
#[derive(Debug, Clone)]
pub struct PaginationState {
    /// Current page token
    pub page_token: Option<String>,
    /// Total items fetched so far
    pub items_fetched: usize,
    /// Maximum items to fetch (None = unlimited)
    pub max_items: Option<usize>,
    /// Sync token for incremental sync
    pub sync_token: Option<String>,
    /// Whether we've reached the end
    pub exhausted: bool,
}

impl PaginationState {
    pub fn new() -> Self {
        Self {
            page_token: None,
            items_fetched: 0,
            max_items: None,
            sync_token: None,
            exhausted: false,
        }
    }

    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }

    pub fn with_sync_token(mut self, token: String) -> Self {
        self.sync_token = Some(token);
        self
    }

    /// Update state after receiving a page
    pub fn update<T>(&mut self, response: &PagedResponse<T>) {
        self.page_token = response.next_page_token.clone();
        if response.next_sync_token.is_some() {
            self.sync_token = response.next_sync_token.clone();
        }
        self.exhausted = self.page_token.is_none();
    }

    /// Check if we should fetch more pages
    pub fn should_continue(&self) -> bool {
        if self.exhausted {
            return false;
        }
        if let Some(max) = self.max_items {
            self.items_fetched < max
        } else {
            true
        }
    }

    /// Add to items fetched count
    pub fn add_items(&mut self, count: usize) {
        self.items_fetched += count;
    }
}

impl Default for PaginationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination result containing items and metadata
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_sync_token: Option<String>,
    pub total_fetched: usize,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>) -> Self {
        let total = items.len();
        Self {
            items,
            next_page_token: None,
            next_sync_token: None,
            total_fetched: total,
        }
    }

    pub fn with_page_token(mut self, token: Option<String>) -> Self {
        self.next_page_token = token;
        self
    }

    pub fn with_sync_token(mut self, token: Option<String>) -> Self {
        self.next_sync_token = token;
        self
    }
}

/// Trait for creating paginated streams
pub trait Paginator {
    type Item: Send;
    type Error: Send;

    /// Fetch a single page
    fn fetch_page(
        &self,
        page_token: Option<&str>,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<PagedResponse<Self::Item>, Self::Error>> + Send + '_>>;
}

/// Create a stream that yields items from all pages
pub fn paginate_stream<P>(
    paginator: P,
    max_items: Option<usize>,
) -> impl Stream<Item = Result<P::Item, P::Error>>
where
    P: Paginator + Send + Sync + 'static,
    P::Item: Send + 'static,
    P::Error: Send + 'static,
{
    async_stream::try_stream! {
        let mut state = PaginationState::new();
        if let Some(max) = max_items {
            state = state.with_max_items(max);
        }

        loop {
            let response = paginator.fetch_page(state.page_token.as_deref()).await?;
            let next_token = response.next_page_token.clone();
            let next_sync = response.next_sync_token.clone();
            let items = response.into_items();

            // Update state with the actual tokens from the response
            state.page_token = next_token;
            if next_sync.is_some() {
                state.sync_token = next_sync;
            }
            state.exhausted = state.page_token.is_none();

            for item in items {
                state.add_items(1);
                if let Some(max) = max_items {
                    if state.items_fetched > max {
                        return;
                    }
                }
                yield item;
            }

            if !state.should_continue() {
                break;
            }
        }
    }
}

/// Collect all pages into a single result
pub async fn collect_all_pages<P>(
    paginator: P,
    max_items: Option<usize>,
) -> Result<PaginatedResult<P::Item>, P::Error>
where
    P: Paginator + Send + Sync,
    P::Item: Send,
    P::Error: Send,
{
    let mut state = PaginationState::new();
    if let Some(max) = max_items {
        state = state.with_max_items(max);
    }

    let mut all_items = Vec::new();

    loop {
        let response = paginator.fetch_page(state.page_token.as_deref()).await?;
        state.update(&response);

        let items = response.into_items();
        let take_count = if let Some(max) = max_items {
            (max - all_items.len()).min(items.len())
        } else {
            items.len()
        };

        all_items.extend(items.into_iter().take(take_count));
        state.add_items(take_count);

        if !state.should_continue() {
            break;
        }
    }

    Ok(PaginatedResult {
        items: all_items,
        next_page_token: state.page_token,
        next_sync_token: state.sync_token,
        total_fetched: state.items_fetched,
    })
}
