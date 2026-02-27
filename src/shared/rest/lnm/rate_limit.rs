use std::time::Duration;

use tokio::{sync::Mutex, time::Instant};

/// A simple fixed-interval rate limiter with separate auth/unauth buckets.
///
/// Each call to [`acquire`](Self::acquire) holds the internal mutex while sleeping, so concurrent
/// callers queue in FIFO order behind the previous request. With *N* concurrent requests the
/// *N*-th caller waits roughly *N x interval* before proceeding.
pub(crate) struct RateLimiter {
    last_auth_request: Mutex<Instant>,
    last_unauth_request: Mutex<Instant>,
    auth_interval: Duration,
    unauth_interval: Duration,
}

impl RateLimiter {
    pub fn new(auth_interval: Duration, unauth_interval: Duration) -> Self {
        Self {
            last_auth_request: Mutex::new(Instant::now() - auth_interval),
            last_unauth_request: Mutex::new(Instant::now() - unauth_interval),
            auth_interval,
            unauth_interval,
        }
    }

    pub async fn acquire(&self, authenticated: bool) {
        let (last, interval) = if authenticated {
            (&self.last_auth_request, self.auth_interval)
        } else {
            (&self.last_unauth_request, self.unauth_interval)
        };

        let mut last = last.lock().await;
        let elapsed = last.elapsed();

        if elapsed < interval {
            tokio::time::sleep(interval - elapsed).await;
        }

        *last = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn first_request_fires_immediately() {
        let rl = RateLimiter::new(Duration::from_millis(100), Duration::from_secs(1));
        let start = Instant::now();
        rl.acquire(true).await;

        assert!(start.elapsed() < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn auth_interval_pacing() {
        let rl = RateLimiter::new(Duration::from_millis(50), Duration::from_secs(1));
        rl.acquire(true).await;
        let start = Instant::now();
        rl.acquire(true).await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn unauth_interval_pacing() {
        let rl = RateLimiter::new(Duration::from_millis(50), Duration::from_millis(200));
        rl.acquire(false).await;
        let start = Instant::now();
        rl.acquire(false).await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(180));
        assert!(elapsed < Duration::from_millis(300));
    }

    #[tokio::test]
    async fn auth_does_not_block_unauth() {
        let rl = RateLimiter::new(Duration::from_millis(200), Duration::from_millis(50));

        // Fire an auth request
        rl.acquire(true).await;

        // Unauth should fire immediately (separate bucket)
        let start = Instant::now();
        rl.acquire(false).await;

        assert!(start.elapsed() < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn unauth_does_not_block_auth() {
        let rl = RateLimiter::new(Duration::from_millis(50), Duration::from_millis(200));

        // Fire an unauth request
        rl.acquire(false).await;

        // Auth should fire immediately (separate bucket)
        let start = Instant::now();
        rl.acquire(true).await;

        assert!(start.elapsed() < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn auth_faster_than_unauth() {
        let rl = RateLimiter::new(Duration::from_millis(20), Duration::from_millis(200));

        // Auth request then another auth request — short interval
        rl.acquire(true).await;
        let start = Instant::now();
        rl.acquire(true).await;
        let auth_elapsed = start.elapsed();

        // Unauth request then another unauth request — long interval
        rl.acquire(false).await;
        let start = Instant::now();
        rl.acquire(false).await;
        let unauth_elapsed = start.elapsed();

        assert!(auth_elapsed < unauth_elapsed);
    }

    #[tokio::test]
    async fn fifo_ordering() {
        let rl = Arc::new(RateLimiter::new(
            Duration::from_millis(20),
            Duration::from_secs(1),
        ));

        let order = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for i in 0..3u32 {
            let rl = rl.clone();
            let order = order.clone();
            handles.push(tokio::spawn(async move {
                rl.acquire(true).await;
                order.lock().await.push(i);
            }));
            // Small delay so each task reaches the mutex in spawn order.
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        for h in handles {
            h.await.unwrap();
        }

        let order = order.lock().await;
        assert_eq!(*order, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn multiple_requests_paced() {
        let rl = RateLimiter::new(Duration::from_millis(20), Duration::from_secs(1));
        let start = Instant::now();
        for _ in 0..5 {
            rl.acquire(true).await;
        }
        let elapsed = start.elapsed();

        // First fires immediately, then 4 intervals of 20ms = 80ms minimum
        assert!(elapsed >= Duration::from_millis(70));
        assert!(elapsed < Duration::from_millis(200));
    }
}
