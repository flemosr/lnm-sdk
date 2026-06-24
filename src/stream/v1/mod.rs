use std::sync::Arc;

use tokio::sync::Mutex;

pub mod error;

mod config;
mod lnm;
mod models;
mod repositories;
mod state;

pub use config::StreamClientConfig;
use error::Result;
use lnm::LnmStreamRepo;
pub use models::{
    AuthenticateResult, HelloResult, StreamJsonRpcError, StreamOhlcTimeframe, StreamRateLimit,
    StreamResponseMetadata, StreamSubscription, StreamTopic, StreamUpdate, TimeResult,
    WhoamiResult,
};
pub use repositories::StreamRepository;
pub use state::StreamConnectionStatus;

/// Thread-safe handle to a [`StreamRepository`].
pub type StreamConnection = Arc<dyn StreamRepository>;

/// Client for interacting with the LN Markets Stream v1 API.
///
/// The default endpoint is `wss://stream.lnmarkets.com/v1`.
pub struct StreamClient {
    config: StreamClientConfig,
    conn: Mutex<Option<StreamConnection>>,
}

impl StreamClient {
    /// Creates a new Stream v1 client in a disconnected state.
    pub fn new(config: impl Into<StreamClientConfig>) -> Arc<Self> {
        Arc::new(Self {
            config: config.into(),
            conn: Mutex::new(None),
        })
    }

    /// Connects to the Stream API or returns an existing active connection.
    pub async fn connect(&self) -> Result<StreamConnection> {
        let mut conn_guard = self.conn.lock().await;

        if let Some(conn) = conn_guard.as_ref()
            && conn.is_connected().await
        {
            return Ok(conn.clone());
        }

        let new_conn = Arc::new(LnmStreamRepo::new(self.config.clone()).await?);

        *conn_guard = Some(new_conn.clone());

        Ok(new_conn)
    }

    /// Clears the cached connection handle.
    pub async fn reset(&self) {
        let mut conn_guard = self.conn.lock().await;

        *conn_guard = None;
    }
}
