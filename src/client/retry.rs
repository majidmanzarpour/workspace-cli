use std::time::Duration;
use rand::Rng;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier (typically 2.0 for exponential)
    pub multiplier: f64,
    /// Whether to add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    pub fn initial_backoff(mut self, d: Duration) -> Self {
        self.initial_backoff = d;
        self
    }

    pub fn max_backoff(mut self, d: Duration) -> Self {
        self.max_backoff = d;
        self
    }

    pub fn multiplier(mut self, m: f64) -> Self {
        self.multiplier = m;
        self
    }

    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Aggressive config for rate-limited APIs (Docs, Sheets, Slides)
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Conservative config for high-volume APIs (Gmail, Drive)
    pub fn conservative() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry state tracker
pub struct RetryState {
    config: RetryConfig,
    attempt: u32,
}

impl RetryState {
    pub fn new(config: RetryConfig) -> Self {
        Self { config, attempt: 0 }
    }

    /// Get current attempt number (0-indexed)
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Check if we should retry
    pub fn should_retry(&self) -> bool {
        self.attempt < self.config.max_retries
    }

    /// Get the next backoff duration and increment attempt counter
    pub fn next_backoff(&mut self) -> Option<Duration> {
        if !self.should_retry() {
            return None;
        }

        let backoff = self.calculate_backoff();
        self.attempt += 1;
        Some(backoff)
    }

    /// Calculate backoff for current attempt
    fn calculate_backoff(&self) -> Duration {
        let base = self.config.initial_backoff.as_secs_f64()
            * self.config.multiplier.powi(self.attempt as i32);

        let capped = base.min(self.config.max_backoff.as_secs_f64());

        let final_duration = if self.config.jitter {
            // Add random jitter: 0.5x to 1.5x the calculated duration
            let mut rng = rand::thread_rng();
            let jitter_factor = 0.5 + (rng.gen::<f64>() * 1.0); // 0.5 to 1.5
            capped * jitter_factor
        } else {
            capped
        };

        Duration::from_secs_f64(final_duration)
    }

    /// Reset the retry state
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

/// Determines if an error is retryable
pub trait Retryable {
    fn is_retryable(&self) -> bool;
    fn retry_after(&self) -> Option<Duration>;
}

/// HTTP status code based retry decision
pub fn is_retryable_status(status: u16) -> bool {
    matches!(status,
        408 | // Request Timeout
        429 | // Too Many Requests
        500 | // Internal Server Error
        502 | // Bad Gateway
        503 | // Service Unavailable
        504   // Gateway Timeout
    )
}

/// Parse Retry-After header value
pub fn parse_retry_after(value: &str) -> Option<Duration> {
    // Try parsing as seconds first
    if let Ok(secs) = value.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Try parsing as HTTP date (simplified - just extract reasonable delay)
    // In practice, Google APIs usually return seconds
    None
}

/// Execute an async operation with retry logic
pub async fn with_retry<F, Fut, T, E>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: Retryable + std::fmt::Debug,
{
    let mut state = RetryState::new(config);

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !e.is_retryable() {
                    return Err(RetryError::NonRetryable(e));
                }

                // Check if we can retry
                if !state.should_retry() {
                    return Err(RetryError::MaxRetriesExceeded {
                        attempts: state.attempt(),
                        last_error: e,
                    });
                }

                // Use Retry-After header if present, otherwise use calculated backoff
                let duration = if let Some(retry_after) = e.retry_after() {
                    state.attempt += 1; // Increment attempt counter even when using Retry-After
                    retry_after
                } else {
                    state.next_backoff().expect("should_retry() passed but next_backoff() returned None")
                };

                tracing::debug!(
                    attempt = state.attempt(),
                    backoff_ms = duration.as_millis() as u64,
                    "Retrying after backoff"
                );
                tokio::time::sleep(duration).await;
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RetryError<E> {
    #[error("Non-retryable error: {0:?}")]
    NonRetryable(E),

    #[error("Max retries ({attempts}) exceeded")]
    MaxRetriesExceeded {
        attempts: u32,
        last_error: E,
    },
}

impl<E> RetryError<E> {
    pub fn into_inner(self) -> E {
        match self {
            Self::NonRetryable(e) => e,
            Self::MaxRetriesExceeded { last_error, .. } => last_error,
        }
    }
}
