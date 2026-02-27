use std::time::Duration;

/// Configuration for WebSocket clients.
#[derive(Clone, Debug)]
pub struct WebSocketClientConfig {
    disconnect_timeout: Duration,
}

impl WebSocketClientConfig {
    /// Creates a new WebSocket client configuration with the specified disconnect timeout.
    pub fn new(disconnect_timeout: Duration) -> Self {
        Self { disconnect_timeout }
    }

    /// Returns the disconnect timeout duration.
    pub fn disconnect_timeout(&self) -> Duration {
        self.disconnect_timeout
    }

    /// Sets the disconnect timeout duration.
    ///
    /// Default: `6` seconds
    pub fn with_disconnect_timeout(mut self, disconnect_timeout: Duration) -> Self {
        self.disconnect_timeout = disconnect_timeout;
        self
    }
}

impl Default for WebSocketClientConfig {
    fn default() -> Self {
        Self {
            disconnect_timeout: Duration::from_secs(6),
        }
    }
}
