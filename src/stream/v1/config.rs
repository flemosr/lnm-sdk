use std::time::Duration;

/// Configuration for the Stream v1 WebSocket client.
#[derive(Clone, Debug)]
pub struct StreamClientConfig {
    endpoint: String,
    heartbeat_interval: Duration,
    disconnect_timeout: Duration,
    reconnect_initial_backoff: Duration,
    reconnect_max_backoff: Duration,
    reconnect_max_attempts: Option<usize>,
}

impl StreamClientConfig {
    /// Creates a new Stream v1 client configuration with the default endpoint.
    pub fn new(heartbeat_interval: Duration, disconnect_timeout: Duration) -> Self {
        Self {
            heartbeat_interval,
            disconnect_timeout,
            ..Default::default()
        }
    }

    /// Returns the Stream API endpoint.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Returns the heartbeat interval used for WebSocket ping frames.
    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    /// Returns the disconnect timeout duration.
    pub fn disconnect_timeout(&self) -> Duration {
        self.disconnect_timeout
    }

    /// Returns the initial reconnect backoff.
    pub fn reconnect_initial_backoff(&self) -> Duration {
        self.reconnect_initial_backoff
    }

    /// Returns the maximum reconnect backoff.
    pub fn reconnect_max_backoff(&self) -> Duration {
        self.reconnect_max_backoff
    }

    /// Returns the maximum number of reconnect attempts.
    ///
    /// `None` means the client will keep trying until explicitly disconnected.
    pub fn reconnect_max_attempts(&self) -> Option<usize> {
        self.reconnect_max_attempts
    }

    /// Sets the Stream API endpoint.
    ///
    /// Default: `wss://stream.lnmarkets.com/v1`
    pub fn with_endpoint(mut self, endpoint: impl ToString) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    /// Sets the heartbeat interval.
    ///
    /// Default: `30` seconds
    pub fn with_heartbeat_interval(mut self, heartbeat_interval: Duration) -> Self {
        self.heartbeat_interval = heartbeat_interval;
        self
    }

    /// Sets the disconnect timeout duration.
    ///
    /// Default: `6` seconds
    pub fn with_disconnect_timeout(mut self, disconnect_timeout: Duration) -> Self {
        self.disconnect_timeout = disconnect_timeout;
        self
    }

    /// Sets the initial reconnect backoff.
    ///
    /// Default: `1` second
    pub fn with_reconnect_initial_backoff(mut self, reconnect_initial_backoff: Duration) -> Self {
        self.reconnect_initial_backoff = reconnect_initial_backoff;
        self
    }

    /// Sets the maximum reconnect backoff.
    ///
    /// Default: `30` seconds
    pub fn with_reconnect_max_backoff(mut self, reconnect_max_backoff: Duration) -> Self {
        self.reconnect_max_backoff = reconnect_max_backoff;
        self
    }

    /// Sets the maximum number of reconnect attempts.
    ///
    /// Default: `None`, retry indefinitely until explicitly disconnected.
    pub fn with_reconnect_max_attempts(mut self, reconnect_max_attempts: Option<usize>) -> Self {
        self.reconnect_max_attempts = reconnect_max_attempts;
        self
    }
}

impl Default for StreamClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "wss://stream.lnmarkets.com/v1".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            disconnect_timeout: Duration::from_secs(6),
            reconnect_initial_backoff: Duration::from_secs(1),
            reconnect_max_backoff: Duration::from_secs(30),
            reconnect_max_attempts: None,
        }
    }
}
