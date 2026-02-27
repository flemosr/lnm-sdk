use std::{num::NonZero, time::Duration};

/// Configuration for the v2 REST API client.
#[derive(Clone, Debug)]
pub struct RestClientConfig {
    timeout: Duration,
    rate_limiter_active: bool,
    rate_limit_auth_requests_per_minute: u32,
    rate_limit_unauth_requests_per_minute: u32,
}

impl RestClientConfig {
    /// Creates a new REST client configuration with the specified timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            rate_limiter_active: true,
            rate_limit_auth_requests_per_minute: 60,
            rate_limit_unauth_requests_per_minute: 30,
        }
    }

    /// Returns the request timeout duration.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns whether the rate limiter is active.
    pub fn rate_limiter_active(&self) -> bool {
        self.rate_limiter_active
    }

    /// Returns the authenticated requests-per-minute limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    pub fn rate_limit_auth_requests_per_minute(&self) -> u32 {
        self.rate_limit_auth_requests_per_minute
    }

    /// Returns the unauthenticated requests-per-minute limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    pub fn rate_limit_unauth_requests_per_minute(&self) -> u32 {
        self.rate_limit_unauth_requests_per_minute
    }

    /// Returns the interval between authenticated requests.
    pub(crate) fn rate_limit_auth_interval(&self) -> Duration {
        Duration::from_secs(60) / self.rate_limit_auth_requests_per_minute
    }

    /// Returns the interval between unauthenticated requests.
    pub(crate) fn rate_limit_unauth_interval(&self) -> Duration {
        Duration::from_secs(60) / self.rate_limit_unauth_requests_per_minute
    }

    /// Sets the request timeout duration.
    ///
    /// Default: `20` seconds
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enables or disables the rate limiter.
    ///
    /// Default: `true`
    pub fn with_rate_limiter_active(mut self, active: bool) -> Self {
        self.rate_limiter_active = active;
        self
    }

    /// Sets the authenticated requests-per-minute limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    ///
    /// Default: `60` (1 request per second)
    pub fn with_rate_limit_auth_requests_per_minute(
        mut self,
        rate: NonZero<u32>,
    ) -> Self {
        self.rate_limit_auth_requests_per_minute = rate.get();
        self
    }

    /// Sets the unauthenticated requests-per-minute limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    ///
    /// Default: `30`
    pub fn with_rate_limit_unauth_requests_per_minute(mut self, rate: NonZero<u32>) -> Self {
        self.rate_limit_unauth_requests_per_minute = rate.get();
        self
    }
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(20),
            rate_limiter_active: true,
            rate_limit_auth_requests_per_minute: 60,
            rate_limit_unauth_requests_per_minute: 30,
        }
    }
}
