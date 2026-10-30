use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

struct LimiterEntry {
    last_reset: Instant,
    count: u32,
}

struct LimiterState {
    last_cleanup: Instant,
    entries: HashMap<IpAddr, LimiterEntry>,
}

pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    state: Mutex<LimiterState>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            state: Mutex::new(LimiterState {
                last_cleanup: Instant::now(),
                entries: HashMap::new(),
            }),
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        let mut guard = self.state.lock();
        let now = Instant::now();

        if now.duration_since(guard.last_cleanup) > Duration::from_secs(60) {
            guard
                .entries
                .retain(|_, entry| now.duration_since(entry.last_reset) < Duration::from_secs(60));
            guard.last_cleanup = now;
        }

        let entry = guard.entries.entry(ip).or_insert(LimiterEntry {
            last_reset: now,
            count: 0,
        });

        if now.duration_since(entry.last_reset) >= self.window {
            entry.last_reset = now;
            entry.count = 1;
            true
        } else if entry.count < self.max_requests {
            entry.count += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limiter_allows_up_to_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        for _ in 0..10 {
            assert!(limiter.check(ip));
        }

        assert!(!limiter.check(ip));
    }

    #[test]
    fn test_rate_limiter_distinct_ips() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1));
        let ip1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));

        assert!(limiter.check(ip1));
        assert!(limiter.check(ip1));
        assert!(!limiter.check(ip1));

        assert!(limiter.check(ip2));
        assert!(limiter.check(ip2));
        assert!(!limiter.check(ip2));
    }
}
