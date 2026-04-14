use std::{num::NonZero, time::Duration};

use crate::shared::rest::lnm::rate_limit::RateLimiterConfig;

/// Configuration for the v2 REST API client.
///
/// Rate limit defaults were set in line with the [API v2 docs](https://docs.lnmarkets.com/api/#limits).
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
            ..Default::default()
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
    pub fn with_rate_limit_auth_requests_per_minute(mut self, rpm: NonZero<u32>) -> Self {
        self.rate_limit_auth_requests_per_minute = rpm.get();
        self
    }

    /// Sets the unauthenticated requests-per-minute limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    ///
    /// Default: `30`
    pub fn with_rate_limit_unauth_requests_per_minute(mut self, rpm: NonZero<u32>) -> Self {
        self.rate_limit_unauth_requests_per_minute = rpm.get();
        self
    }
}

impl RateLimiterConfig for RestClientConfig {
    fn rate_limit_auth_interval(&self) -> Duration {
        Duration::from_secs(60) / self.rate_limit_auth_requests_per_minute
    }

    fn rate_limit_unauth_interval(&self) -> Duration {
        Duration::from_secs(60) / self.rate_limit_unauth_requests_per_minute
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
