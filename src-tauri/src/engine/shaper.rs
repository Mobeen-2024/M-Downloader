use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

/// A thread-safe Token Bucket for high-performance traffic shaping.
/// 
/// Allows for bursts up to the `capacity` while maintaining a long-term
/// average rate of `refill_rate` bytes per second.
pub struct TokenBucket {
    /// Maximum number of tokens the bucket can hold (burst capacity).
    capacity: i64,
    /// Number of tokens added per second.
    refill_rate: i64,
    /// Current number of tokens in the bucket.
    tokens: AtomicI64,
    /// The last time the bucket was refilled.
    last_refill: parking_lot::Mutex<Instant>,
}

impl TokenBucket {
    pub fn new(rate_bps: u64) -> Self {
        let capacity = (rate_bps as i64).max(64 * 1024); // Minimum 64KB burst
        Self {
            capacity,
            refill_rate: rate_bps as i64,
            tokens: AtomicI64::new(capacity),
            last_refill: parking_lot::Mutex::new(Instant::now()),
        }
    }

    /// Attempts to consume `n` tokens from the bucket.
    /// 
    /// If sufficient tokens are available, they are consumed and the function returns `None`.
    /// If tokens are insufficient, it returns the `Duration` the caller should sleep
    /// before retrying to stay within the rate limit.
    pub fn consume(&self, n: i64) -> Option<Duration> {
        self.refill();

        let current = self.tokens.load(Ordering::Relaxed);
        if current >= n {
            // Success: decrement tokens
            self.tokens.fetch_sub(n, Ordering::SeqCst);
            None
        } else {
            // Insufficient tokens: calculate wait time 
            // wait_time = (needed_tokens) / (tokens_per_second)
            let needed = n - current;
            let wait_secs = needed as f64 / self.refill_rate as f64;
            Some(Duration::from_secs_f64(wait_secs.max(0.01))) // Min 10ms sleep
        }
    }

    /// Updates the token count based on elapsed time since the last refill.
    fn refill(&self) {
        let mut last_refill = self.last_refill.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        
        if elapsed < 0.001 {
            return; // Too soon to refill
        }

        let new_tokens = (elapsed * self.refill_rate as f64) as i64;
        if new_tokens > 0 {
            let current = self.tokens.load(Ordering::Relaxed);
            let next = (current + new_tokens).min(self.capacity);
            self.tokens.store(next, Ordering::SeqCst);
            *last_refill = now;
        }
    }

    /// Dynamically update the refill rate (speed limit).
    pub fn set_rate(&self, new_rate_bps: u64) {
        let mut last_refill = self.last_refill.lock();
        // Reset tokens if rate changes significantly to avoid burst artifacts
        self.tokens.store(new_rate_bps as i64, Ordering::SeqCst);
        *last_refill = Instant::now();
    }
}
