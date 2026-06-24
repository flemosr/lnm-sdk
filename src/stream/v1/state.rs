use std::{
    fmt,
    sync::{Arc, Mutex, MutexGuard},
};

use super::error::StreamConnectionError;

/// Current Stream v1 WebSocket connection status.
#[derive(Debug, Clone)]
pub enum StreamConnectionStatus {
    Connected,
    Reconnecting,
    DisconnectInitiated,
    Disconnected,
    Failed(Arc<StreamConnectionError>),
}

impl StreamConnectionStatus {
    /// Returns `true` when the connection is currently established.
    pub fn is_connected(&self) -> bool {
        matches!(self, StreamConnectionStatus::Connected)
    }
}

impl fmt::Display for StreamConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamConnectionStatus::Connected => write!(f, "Connected"),
            StreamConnectionStatus::Reconnecting => write!(f, "Reconnecting"),
            StreamConnectionStatus::DisconnectInitiated => write!(f, "Disconnect Initiated"),
            StreamConnectionStatus::Disconnected => write!(f, "Disconnected"),
            StreamConnectionStatus::Failed(err) => write!(f, "Failed: {}", err),
        }
    }
}

pub(super) struct StreamConnectionStatusManager(Mutex<StreamConnectionStatus>);

impl StreamConnectionStatusManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(Mutex::new(StreamConnectionStatus::Connected)))
    }

    fn lock_status(&self) -> MutexGuard<'_, StreamConnectionStatus> {
        self.0
            .lock()
            .expect("`StreamConnectionStatusManager` mutex can't be poisoned")
    }

    pub fn update(&self, new_status: StreamConnectionStatus) {
        let mut status_guard = self.lock_status();

        *status_guard = new_status
    }

    pub fn snapshot(&self) -> StreamConnectionStatus {
        self.lock_status().clone()
    }

    pub fn is_connected(&self) -> bool {
        self.lock_status().is_connected()
    }
}
