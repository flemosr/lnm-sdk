use std::{fmt, io, result, string::FromUtf8Error};

use fastwebsockets::{OpCode, WebSocketError};
use hmac::digest::InvalidLength;
use hyper::http::{self, uri::InvalidUri};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinError,
};
use tokio_rustls::rustls::pki_types::InvalidDnsNameError;

pub use crate::shared::models::error::OhlcRangeParseError;

use super::{
    lnm::TopicStatus,
    models::{StreamTopic, StreamUpdate},
    state::StreamConnectionStatus,
};

/// JSON-RPC error returned by the Stream API.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StreamJsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl StreamJsonRpcError {
    pub fn code(&self) -> i64 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }

    /// Returns `error.data.code` when present.
    pub fn application_code(&self) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(Value::as_object)
            .and_then(|data| data.get("code"))
            .and_then(Value::as_str)
    }
}

impl fmt::Display for StreamJsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.application_code() {
            Some(application_code) => write!(
                f,
                "JSON-RPC error {} ({application_code}): {}",
                self.code, self.message
            ),
            None => write!(f, "JSON-RPC error {}: {}", self.code, self.message),
        }
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StreamConnectionError {
    #[error("InvalidDnsName error, {0}")]
    InvalidDnsName(InvalidDnsNameError),

    #[error("InvalidEndpointUri error, {0}")]
    InvalidEndpointUri(InvalidUri),

    #[error("InvalidEndpoint error, {0}")]
    InvalidEndpoint(String),

    #[error("CreateTcpStream error, {0}")]
    CreateTcpStream(io::Error),

    #[error("ConnectTcpStream error, {0}")]
    ConnectTcpStream(io::Error),

    #[error("HttpUpgradeRequest error, {0}")]
    HttpUpgradeRequest(http::Error),

    #[error("Handshake error, {0}")]
    Handshake(WebSocketError),

    #[error("WriteFrame error, {0}")]
    WriteFrame(WebSocketError),

    #[error("EncodeJson error, {0}")]
    EncodeJson(serde_json::Error),

    #[error("ReadFrame error, {0}")]
    ReadFrame(WebSocketError),

    #[error("DecodeText error, {0}")]
    DecodeText(FromUtf8Error),

    #[error("DecodeJson error, {0}")]
    DecodeJson(serde_json::Error),

    #[error("UnhandledOpCode error, {0:?}")]
    UnhandledOpCode(OpCode),

    #[error("ServerRequestedClose error")]
    ServerRequestedClose,

    #[error("NoServerCloseConfirmation error")]
    NoServerCloseConfirmation,

    #[error("NoServerPong error")]
    NoServerPong,

    #[error("ConnectionInterrupted error")]
    ConnectionInterrupted,

    #[error("ReconnectAttemptsExhausted error")]
    ReconnectAttemptsExhausted,

    #[error("ReauthenticationRejected error")]
    ReauthenticationRejected,

    #[error(
        "SubscriptionRestoreMismatch error, requested {requested:?}, subscribed {subscribed:?}"
    )]
    SubscriptionRestoreMismatch {
        requested: Vec<StreamTopic>,
        subscribed: Vec<StreamTopic>,
    },

    #[error("UnexpectedJsonRpcEnvelope error, {0}")]
    UnexpectedJsonRpcEnvelope(String),

    #[error("UnexpectedJsonRpcResult error for method {method}: {result}")]
    UnexpectedJsonRpcResult {
        method: String,
        result: serde_json::Value,
    },

    #[error("JsonRpcError error, {0}")]
    JsonRpcError(StreamJsonRpcError),

    #[error("UnknownTopic error, {0}")]
    UnknownTopic(String),

    #[error("UnknownOhlcTimeframe error, {0}")]
    UnknownOhlcTimeframe(String),

    #[error("InvalidTimestampMillis error, {0}")]
    InvalidTimestampMillis(i64),

    #[error("InvalidSecretHmac error, {0}")]
    InvalidSecretHmac(InvalidLength),
}

impl StreamConnectionError {
    pub(crate) fn is_reconnectable(&self) -> bool {
        matches!(
            self,
            Self::WriteFrame(_)
                | Self::ReadFrame(_)
                | Self::ServerRequestedClose
                | Self::NoServerPong
        )
    }
}

pub(super) type ConnectionResult<T> = result::Result<T, StreamConnectionError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StreamApiError {
    #[error("Failed to spawn event loop: {0}")]
    FailedToSpawnEventLoop(StreamConnectionError),

    #[error("BadConnectionStatus error, {0}")]
    BadConnectionStatus(StreamConnectionStatus),

    #[error("SendConnectionUpdate error, {0}")]
    SendConnectionUpdate(broadcast::error::SendError<StreamUpdate>),

    #[error("Request queue is closed")]
    RequestQueueClosed,

    #[error("ReceiveResponse error, {0}")]
    ReceiveResponse(oneshot::error::RecvError),

    #[error("RequestFailed error, {0}")]
    RequestFailed(StreamConnectionError),

    #[error("InvalidRpcResult error for {0}")]
    InvalidRpcResult(&'static str),

    #[error("SubscribeWithUnsubscriptionPending error, {0}")]
    SubscribeWithUnsubscriptionPending(StreamTopic),

    #[error("InvalidSubscriptionsTopicNotFound error, {0}")]
    InvalidSubscriptionsTopicNotFound(StreamTopic),

    #[error("InvalidSubscriptionsTopicStatus error")]
    InvalidSubscriptionsTopicStatus {
        topic: StreamTopic,
        status: TopicStatus,
    },

    #[error("UnsubscribeWithSubscriptionPending error, {0}")]
    UnsubscribeWithSubscriptionPending(StreamTopic),

    #[error("SendDisconnectRequest error, {0}")]
    SendDisconnectRequest(mpsc::error::SendError<()>),

    #[error("[TaskJoin] {0}")]
    TaskJoin(JoinError),

    #[error("Stream WebSocket is not connected, status: {0}")]
    WebSocketNotConnected(StreamConnectionStatus),

    #[error("Stream WebSocket disconnect timeout")]
    DisconnectTimeout,
}

pub(super) type Result<T> = result::Result<T, StreamApiError>;
