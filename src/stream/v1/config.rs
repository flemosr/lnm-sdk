use std::time::Duration;

/// Configuration for the Stream v1 WebSocket client.
#[derive(Clone, Debug)]
pub struct StreamClientConfig {
    endpoint: String,
    heartbeat_interval: Duration,
    disconnect_timeout: Duration,
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
}

impl Default for StreamClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "wss://stream.lnmarkets.com/v1".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            disconnect_timeout: Duration::from_secs(6),
        }
    }
}
