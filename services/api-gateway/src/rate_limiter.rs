use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

const SHARDS: usize = 32;

struct LimiterEntry {
    last_reset: Instant,
    count: u32,
}

struct LimiterState {
    last_cleanup: Instant,
    entries: HashMap<IpAddr, LimiterEntry>,
}

#[inline]
fn ip_shard_index(ip: IpAddr) -> usize {
    match ip {
        IpAddr::V4(v4) => {
            let n = u32::from(v4);
            (n.wrapping_mul(0x9E3779B9) as usize) & (SHARDS - 1)
        }
        IpAddr::V6(v6) => {
            let segs = v6.segments();
            let mut h: u64 = 0;
            for seg in segs {
                h = (h.wrapping_mul(0x517cc1b727220a95)).wrapping_add(seg as u64);
            }
            (h as usize) & (SHARDS - 1)
        }
    }
}

pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    shards: [Mutex<LimiterState>; SHARDS],
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        let now = Instant::now();
        Self {
            max_requests,
            window,
            shards: std::array::from_fn(|_| {
                Mutex::new(LimiterState {
                    last_cleanup: now,
                    entries: HashMap::with_capacity(64),
                })
            }),
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        let idx = ip_shard_index(ip);
        let mut guard = self.shards[idx].lock();
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
