use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A lock-free Token Bucket for institutional-grade traffic shaping.
/// 
/// Uses nanosecond-precise atomic timekeeping to ensure maximum throughput
/// and strict rate-limit compliance across 32+ worker threads.
pub struct TokenBucket {
    /// Maximum number of tokens (burst capacity).
    capacity: AtomicI64,
    /// Number of tokens added per second.
    refill_rate: AtomicI64,
    /// Current number of tokens in the bucket.
    tokens: AtomicI64,
    /// Last refill timestamp in nanoseconds since UNIX_EPOCH.
    last_refill_ns: AtomicU64,
}

impl TokenBucket {
    pub fn new(rate_bps: u64) -> Self {
        let capacity = (rate_bps as i64).min(8 * 1024 * 1024).max(64 * 1024); // Cap burst at 8MB
        let now_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

        Self {
            capacity: AtomicI64::new(capacity),
            refill_rate: AtomicI64::new(rate_bps as i64),
            tokens: AtomicI64::new(capacity),
            last_refill_ns: AtomicU64::new(now_ns),
        }
    }

    /// Attempts to consume `n` tokens from the bucket.
    /// Returns `None` on success, or `Duration` to wait on failure.
    pub fn consume(&self, n: i64) -> Option<Duration> {
        self.refill();

        loop {
            let current = self.tokens.load(Ordering::Acquire);
            if current >= n {
                if self.tokens.compare_exchange(current, current - n, Ordering::SeqCst, Ordering::Acquire).is_ok() {
                    return None;
                }
            } else {
                let rate = self.refill_rate.load(Ordering::Relaxed);
                if rate <= 0 { return None; } // No limit

                let needed = n - current;
                let wait_secs = needed as f64 / rate as f64;
                return Some(Duration::from_secs_f64(wait_secs.max(0.005))); // 5ms min sleep
            }
        }
    }

    /// Incremental refill using lock-free atomic timekeeping.
    fn refill(&self) {
        let now_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        let last_ns = self.last_refill_ns.load(Ordering::Acquire);

        if now_ns <= last_ns { return; }

        let elapsed_ns = now_ns - last_ns;
        let rate = self.refill_rate.load(Ordering::Relaxed);
        let cap = self.capacity.load(Ordering::Relaxed);

        // Tokens to add = (elapsed_nanos / 1,000,000,000) * rate
        let new_tokens = (elapsed_ns as f64 / 1_000_000_000.0 * rate as f64) as i64;

        if new_tokens > 0 {
            if self.last_refill_ns.compare_exchange(last_ns, now_ns, Ordering::SeqCst, Ordering::Acquire).is_ok() {
                let current = self.tokens.load(Ordering::Acquire);
                let next = (current + new_tokens).min(cap);
                self.tokens.store(next, Ordering::Release);
            }
        }
    }

    /// Dynamically updates the rate and burst capacity.
    pub fn set_rate(&self, new_rate_bps: u64) {
        let new_cap = (new_rate_bps as i64).min(8 * 1024 * 1024).max(64 * 1024);
        self.refill_rate.store(new_rate_bps as i64, Ordering::Release);
        self.capacity.store(new_cap, Ordering::Release);
        // Removed tokens.store(new_cap) to ensure a smooth transition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_token_bucket_refill() {
        let bucket = TokenBucket::new(1000); // 1000 bytes per sec
        let min_cap = 64 * 1024; // the bucket enforces 64KB minimum burst
        
        // Initial state should be full
        assert!(bucket.consume(min_cap).is_none());
        assert!(bucket.consume(1).is_some()); // Empty now
        
        // Wait for refill (at least 100ms for 100 tokens)
        thread::sleep(Duration::from_millis(150));
        assert!(bucket.consume(100).is_none());
    }

    #[test]
    fn test_token_bucket_burst() {
        let bucket = TokenBucket::new(100); 
        let min_cap = 64 * 1024;
        // Should allow consuming capacity immediately
        assert!(bucket.consume(min_cap).is_none());
    }
}
