use parking_lot::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Состояние предохранителя Circuit Breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerStatus {
    /// Штатная работа: апстрим доступен, запросы пропускаются.
    Closed,
    /// Апстрим недоступен: порог ошибок превышен, запросы блокируются на шлюзе (503).
    Open,
    /// Кулдаун истёк: шлюз пропускает один пробный запрос для проверки восстановления.
    HalfOpen,
}

/// Предохранитель Circuit Breaker для защиты внутренних микросервисов от каскадных сбоев.
pub struct CircuitBreaker {
    name: &'static str,
    failure_threshold: u32,
    cooldown: Duration,
    consecutive_failures: AtomicU32,
    last_failure: RwLock<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(name: &'static str, failure_threshold: u32, cooldown_secs: u64) -> Self {
        Self {
            name,
            failure_threshold,
            cooldown: Duration::from_secs(cooldown_secs),
            consecutive_failures: AtomicU32::new(0),
            last_failure: RwLock::new(None),
        }
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Проверка разрешения на запрос к апстриму
    pub fn can_execute(&self) -> (bool, BreakerStatus) {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);

        if failures < self.failure_threshold {
            return (true, BreakerStatus::Closed);
        }

        let last_time = *self.last_failure.read();
        if let Some(timestamp) = last_time
            && timestamp.elapsed() >= self.cooldown
        {
            tracing::info!(
                "[CIRCUIT BREAKER] [{}] Cooldown elapsed. Transitioning to HALF-OPEN (testing trial request)",
                self.name
            );
            return (true, BreakerStatus::HalfOpen);
        }

        (false, BreakerStatus::Open)
    }

    /// Фиксация успешного ответа апстрима
    pub fn record_success(&self) {
        let prev = self.consecutive_failures.swap(0, Ordering::Relaxed);
        if prev >= self.failure_threshold {
            tracing::info!(
                "[CIRCUIT BREAKER] [{}] Upstream has recovered! Circuit state -> CLOSED",
                self.name
            );
        }
    }

    /// Фиксация сбоя подключения
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.write() = Some(Instant::now());

        if failures == self.failure_threshold {
            tracing::error!(
                "[CIRCUIT BREAKER] [{}] Failure threshold ({}) reached! Circuit tripped to OPEN for {:?}",
                self.name,
                self.failure_threshold,
                self.cooldown
            );
        } else {
            tracing::warn!(
                "[CIRCUIT BREAKER] [{}] Failure recorded ({}/{})",
                self.name,
                failures,
                self.failure_threshold
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_transitions() {
        let cb = CircuitBreaker::new("TEST", 2, 1);

        // Initial state -> Closed
        let (allowed, status) = cb.can_execute();
        assert!(allowed);
        assert_eq!(status, BreakerStatus::Closed);

        // 1st failure -> still Closed
        cb.record_failure();
        let (allowed, status) = cb.can_execute();
        assert!(allowed);
        assert_eq!(status, BreakerStatus::Closed);

        // 2nd failure -> tripped to Open
        cb.record_failure();
        let (allowed, status) = cb.can_execute();
        assert!(!allowed);
        assert_eq!(status, BreakerStatus::Open);

        // Recovery via success -> Closed
        cb.record_success();
        let (allowed, status) = cb.can_execute();
        assert!(allowed);
        assert_eq!(status, BreakerStatus::Closed);
    }
}
