use std::{num::NonZero, time::Duration};

use crate::shared::rest::lnm::rate_limit::RateLimiterConfig;

/// Configuration for the v3 REST API client.
///
/// Rate limit defaults were set in line with the [API v3 docs](https://api.lnmarkets.com/v3/#description/rate-limit).
#[derive(Clone, Debug)]
pub struct RestClientConfig {
    timeout: Duration,
    rate_limiter_active: bool,
    rate_limit_auth_requests_per_second: u32,
    rate_limit_unauth_requests_per_second: u32,
}

impl RestClientConfig {
    /// Creates a new v3 REST client configuration with the specified timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            rate_limiter_active: true,
            rate_limit_auth_requests_per_second: 5,
            rate_limit_unauth_requests_per_second: 1,
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

    /// Returns the authenticated requests-per-second limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    pub fn rate_limit_auth_requests_per_second(&self) -> u32 {
        self.rate_limit_auth_requests_per_second
    }

    /// Returns the unauthenticated requests-per-second limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    pub fn rate_limit_unauth_requests_per_second(&self) -> u32 {
        self.rate_limit_unauth_requests_per_second
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

    /// Sets the authenticated requests-per-second limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    ///
    /// Default: `5`
    pub fn with_rate_limit_auth_requests_per_second(mut self, rate: NonZero<u32>) -> Self {
        self.rate_limit_auth_requests_per_second = rate.get();
        self
    }

    /// Sets the unauthenticated requests-per-second limit.
    ///
    /// Only enforced when [`rate_limiter_active`](Self::rate_limiter_active) is `true`.
    ///
    /// Default: `1`
    pub fn with_rate_limit_unauth_requests_per_second(mut self, rate: NonZero<u32>) -> Self {
        self.rate_limit_unauth_requests_per_second = rate.get();
        self
    }
}

impl RateLimiterConfig for RestClientConfig {
    fn rate_limit_auth_interval(&self) -> Duration {
        Duration::from_secs(1) / self.rate_limit_auth_requests_per_second
    }

    fn rate_limit_unauth_interval(&self) -> Duration {
        Duration::from_secs(1) / self.rate_limit_unauth_requests_per_second
    }
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(20),
            rate_limiter_active: true,
            rate_limit_auth_requests_per_second: 5,
            rate_limit_unauth_requests_per_second: 1,
        }
    }
}
