use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

/// Rate limiter configuration for a specific API
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum tokens in the bucket
    pub capacity: u32,
    /// Tokens added per second
    pub refill_rate: f64,
    /// Initial tokens (defaults to capacity)
    pub initial_tokens: Option<u32>,
}

impl RateLimitConfig {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            initial_tokens: None,
        }
    }

    /// Gmail: 250 quota units/sec
    pub fn gmail() -> Self {
        Self::new(250, 250.0)
    }

    /// Drive: 200 req/sec (12000/min)
    pub fn drive() -> Self {
        Self::new(200, 200.0)
    }

    /// Drive write operations: 3/sec
    pub fn drive_write() -> Self {
        Self::new(3, 3.0)
    }

    /// Calendar: 5 req/sec (500/100sec)
    pub fn calendar() -> Self {
        Self::new(5, 5.0)
    }

    /// Docs/Sheets/Slides: 1 req/sec (60/min)
    pub fn docs() -> Self {
        Self::new(1, 1.0)
    }

    /// Tasks: ~0.5 req/sec (50000/day ≈ 0.58/sec, be conservative)
    pub fn tasks() -> Self {
        Self::new(10, 0.5)
    }
}

/// Token bucket rate limiter
pub struct TokenBucket {
    config: RateLimitConfig,
    tokens: Mutex<f64>,
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    pub fn new(config: RateLimitConfig) -> Self {
        let initial = config.initial_tokens.unwrap_or(config.capacity) as f64;
        Self {
            config,
            tokens: Mutex::new(initial),
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// Acquire tokens, waiting if necessary
    pub async fn acquire(&self, cost: u32) -> Result<(), RateLimitError> {
        let cost = cost as f64;

        if cost > self.config.capacity as f64 {
            return Err(RateLimitError::CostExceedsCapacity {
                cost: cost as u32,
                capacity: self.config.capacity,
            });
        }

        loop {
            self.refill().await;

            let mut tokens = self.tokens.lock().await;

            if *tokens >= cost {
                *tokens -= cost;
                return Ok(());
            }

            // Calculate wait time
            let needed = cost - *tokens;
            let wait_secs = needed / self.config.refill_rate;
            drop(tokens); // Release lock while waiting

            tokio::time::sleep(Duration::from_secs_f64(wait_secs.min(1.0))).await;
        }
    }

    /// Try to acquire tokens without waiting
    pub async fn try_acquire(&self, cost: u32) -> bool {
        self.refill().await;

        let mut tokens = self.tokens.lock().await;
        let cost = cost as f64;

        if *tokens >= cost {
            *tokens -= cost;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    async fn refill(&self) {
        let now = Instant::now();

        // Acquire both locks in a consistent order to avoid deadlock
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        let elapsed = now.duration_since(*last_refill).as_secs_f64();

        if elapsed > 0.0 {
            let refill = elapsed * self.config.refill_rate;
            *tokens = (*tokens + refill).min(self.config.capacity as f64);
            *last_refill = now;
        }
    }

    /// Get current token count
    pub async fn available(&self) -> u32 {
        self.refill().await;
        let tokens = self.tokens.lock().await;
        *tokens as u32
    }
}

/// Semaphore-based concurrency limiter (for Drive writes)
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    max_permits: usize,
}

impl ConcurrencyLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_permits: max_concurrent,
        }
    }

    /// Drive write limiter: 3 concurrent operations
    pub fn drive_write() -> Self {
        Self::new(3)
    }

    /// Acquire a permit, waiting if necessary
    pub async fn acquire(&self) -> ConcurrencyPermit {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        ConcurrencyPermit { _permit: permit }
    }

    /// Try to acquire without waiting
    pub fn try_acquire(&self) -> Option<ConcurrencyPermit> {
        self.semaphore.clone().try_acquire_owned().ok().map(|permit| {
            ConcurrencyPermit { _permit: permit }
        })
    }

    /// Get available permits
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get max permits
    pub fn max_permits(&self) -> usize {
        self.max_permits
    }
}

/// RAII permit that releases on drop
pub struct ConcurrencyPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

/// Composite rate limiter for a specific API
pub struct ApiRateLimiter {
    token_bucket: TokenBucket,
    concurrency: Option<ConcurrencyLimiter>,
}

impl ApiRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            token_bucket: TokenBucket::new(config),
            concurrency: None,
        }
    }

    pub fn with_concurrency(mut self, limiter: ConcurrencyLimiter) -> Self {
        self.concurrency = Some(limiter);
        self
    }

    /// Acquire rate limit, returning optional concurrency permit
    pub async fn acquire(&self, cost: u32) -> Result<Option<ConcurrencyPermit>, RateLimitError> {
        self.token_bucket.acquire(cost).await?;

        if let Some(ref concurrency) = self.concurrency {
            Ok(Some(concurrency.acquire().await))
        } else {
            Ok(None)
        }
    }

    /// Gmail rate limiter
    pub fn gmail() -> Self {
        Self::new(RateLimitConfig::gmail())
    }

    /// Drive rate limiter with write concurrency limit
    pub fn drive() -> Self {
        Self::new(RateLimitConfig::drive())
            .with_concurrency(ConcurrencyLimiter::drive_write())
    }

    /// Calendar rate limiter
    pub fn calendar() -> Self {
        Self::new(RateLimitConfig::calendar())
    }

    /// Docs/Sheets/Slides rate limiter
    pub fn docs() -> Self {
        Self::new(RateLimitConfig::docs())
    }

    /// Tasks rate limiter
    pub fn tasks() -> Self {
        Self::new(RateLimitConfig::tasks())
    }
}

/// Quota costs for Gmail operations
pub mod gmail_costs {
    pub const LIST: u32 = 5;
    pub const GET: u32 = 5;
    pub const SEND: u32 = 100;
    pub const MODIFY: u32 = 5;
    pub const DELETE: u32 = 10;
    pub const BATCH_MODIFY: u32 = 50;
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Operation cost ({cost}) exceeds bucket capacity ({capacity})")]
    CostExceedsCapacity { cost: u32, capacity: u32 },
}
