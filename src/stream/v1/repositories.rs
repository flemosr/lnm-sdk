use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast::Receiver;

use super::{
    error::Result,
    models::{AuthenticateResult, HelloResult, StreamTopic, StreamUpdate, WhoamiResult},
    state::StreamConnectionStatus,
};

/// Methods for interacting with LN Markets' Stream v1 API.
///
/// This trait is sealed and not meant to be implemented outside of `lnm-sdk`.
#[async_trait]
pub trait StreamRepository: crate::sealed::Sealed + Send + Sync {
    /// Returns whether the WebSocket connection is currently established.
    async fn is_connected(&self) -> bool;

    /// Returns the current connection status of the WebSocket.
    async fn connection_status(&self) -> StreamConnectionStatus;

    /// Sends a `hello` request with client identification metadata.
    async fn hello(&self, client_name: &str, client_version: &str) -> Result<HelloResult>;

    /// Sends a JSON-RPC `ping` request and expects a `pong` response.
    async fn ping(&self) -> Result<()>;

    /// Returns the server time.
    async fn time(&self) -> Result<DateTime<Utc>>;

    /// Authenticates the current Stream session with REST v3 API credentials.
    async fn authenticate(
        &self,
        key: &str,
        secret: &str,
        passphrase: &str,
    ) -> Result<AuthenticateResult>;

    /// Returns the current authenticated session.
    async fn whoami(&self) -> Result<WhoamiResult>;

    /// Subscribes to the specified Stream topics.
    async fn subscribe(&self, topics: Vec<StreamTopic>) -> Result<()>;

    /// Unsubscribes from the specified Stream topics.
    async fn unsubscribe(&self, topics: Vec<StreamTopic>) -> Result<()>;

    /// Unsubscribes from all currently subscribed Stream topics.
    async fn unsubscribe_all(&self) -> Result<Vec<StreamTopic>>;

    /// Returns the set of currently subscribed Stream topics.
    async fn subscriptions(&self) -> HashSet<StreamTopic>;

    /// Creates a new receiver for Stream updates.
    async fn receiver(&self) -> Result<Receiver<StreamUpdate>>;

    /// Disconnects the Stream WebSocket.
    async fn disconnect(&self) -> Result<()>;
}
