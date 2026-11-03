//! In-process login throttling.
//!
//! A small sliding window keyed by `ip|username` stops obvious brute forcing
//! without an external dependency. It is deliberately per process: a
//! multi-replica deployment in front of a shared database would need a shared
//! store, which is out of scope for M0 and documented in the ADR.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Maximum number of tracked keys before expired entries are pruned.
const PRUNE_THRESHOLD: usize = 4096;

/// Sliding-window login throttle.
#[derive(Debug)]
pub struct LoginThrottle {
    window: Duration,
    max_attempts: u32,
    entries: Mutex<HashMap<String, Vec<Instant>>>,
}

impl Default for LoginThrottle {
    fn default() -> Self {
        Self::new(Duration::from_secs(15 * 60), 10)
    }
}

impl LoginThrottle {
    /// Creates a throttle allowing `max_attempts` failures per `window` per key.
    #[must_use]
    pub fn new(window: Duration, max_attempts: u32) -> Self {
        Self {
            window,
            max_attempts,
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn key(ip: Option<IpAddr>, username: &str) -> String {
        format!(
            "{}|{}",
            ip.map_or_else(|| "-".to_owned(), |ip| ip.to_string()),
            username.trim().to_lowercase()
        )
    }

    /// Returns how many seconds the caller must wait, or `None` when allowed.
    #[must_use]
    pub fn check(&self, ip: Option<IpAddr>, username: &str) -> Option<u64> {
        let key = Self::key(ip, username);
        let now = Instant::now();
        let mut entries = match self.entries.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let attempts = entries.entry(key).or_default();
        attempts.retain(|at| now.duration_since(*at) < self.window);
        if attempts.len() < self.max_attempts as usize {
            return None;
        }
        let oldest = attempts.first().copied().unwrap_or(now);
        let elapsed = now.duration_since(oldest);
        let remaining = self.window.saturating_sub(elapsed);
        Some(remaining.as_secs().max(1))
    }

    /// Records a failed attempt.
    pub fn record_failure(&self, ip: Option<IpAddr>, username: &str) {
        let key = Self::key(ip, username);
        let now = Instant::now();
        let mut entries = match self.entries.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if entries.len() >= PRUNE_THRESHOLD {
            entries.retain(|_, attempts| {
                attempts.retain(|at| now.duration_since(*at) < self.window);
                !attempts.is_empty()
            });
        }
        let attempts = entries.entry(key).or_default();
        attempts.retain(|at| now.duration_since(*at) < self.window);
        attempts.push(now);
    }

    /// Clears the counter after a successful login.
    pub fn record_success(&self, ip: Option<IpAddr>, username: &str) {
        let key = Self::key(ip, username);
        let mut entries = match self.entries.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        entries.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_the_limit_and_clears_on_success() {
        let throttle = LoginThrottle::new(Duration::from_secs(60), 2);
        let ip: IpAddr = "10.0.0.1".parse().expect("ip");
        assert!(throttle.check(Some(ip), "admin").is_none());
        throttle.record_failure(Some(ip), "admin");
        throttle.record_failure(Some(ip), "admin");
        let wait = throttle.check(Some(ip), "admin").expect("limited");
        assert!(wait > 0);

        throttle.record_success(Some(ip), "admin");
        assert!(throttle.check(Some(ip), "admin").is_none());
    }

    #[test]
    fn different_usernames_have_separate_budgets() {
        let throttle = LoginThrottle::new(Duration::from_secs(60), 1);
        let ip: IpAddr = "10.0.0.1".parse().expect("ip");
        throttle.record_failure(Some(ip), "admin");
        assert!(throttle.check(Some(ip), "admin").is_some());
        assert!(throttle.check(Some(ip), "other").is_none());
    }

    #[test]
    fn missing_ip_still_throttles() {
        let throttle = LoginThrottle::new(Duration::from_secs(60), 1);
        throttle.record_failure(None, "admin");
        assert!(throttle.check(None, "ADMIN ").is_some());
    }
}
