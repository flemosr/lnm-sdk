use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{Arc, Mutex as SyncMutex},
};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde_json::json;
use sha2::Sha256;
use tokio::{
    sync::{Mutex as AsyncMutex, broadcast, mpsc, oneshot},
    task::JoinHandle,
    time,
};

use super::{
    config::StreamClientConfig,
    error::{ConnectionResult, Result, StreamApiError, StreamConnectionError},
    models::{
        rpc::{
            AuthenticateResult, HelloResult, StreamJsonRpcReqMethod, StreamJsonRpcRequest,
            StreamJsonRpcResult, WhoamiResult,
        },
        topic::{StreamTopic, topics_match, topics_param},
        update::StreamUpdate,
    },
    repositories::StreamRepository,
    state::{StreamConnectionStatus, StreamConnectionStatusManager},
};

mod event_loop;

use event_loop::{
    DisconnectTransmiter, RequestTransmiter, ResponseReceiver, ResponseTransmiter, StreamEventLoop,
};

/// Subscription lifecycle status tracked for a Stream topic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopicStatus {
    /// A subscribe request has been sent but not confirmed.
    SubscriptionPending,
    /// The topic is currently subscribed.
    Subscribed,
    /// An unsubscribe request has been sent but not confirmed.
    UnsubscriptionPending,
}

impl fmt::Display for TopicStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubscriptionPending => write!(f, "SubscriptionPending"),
            Self::Subscribed => write!(f, "Subscribed"),
            Self::UnsubscriptionPending => write!(f, "UnsubscriptionPending"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct StreamCredentials {
    key: String,
    secret: String,
    passphrase: String,
}

impl StreamCredentials {
    fn new(key: &str, secret: &str, passphrase: &str) -> Self {
        Self {
            key: key.to_string(),
            secret: secret.to_string(),
            passphrase: passphrase.to_string(),
        }
    }

    fn authenticate_params(&self) -> ConnectionResult<serde_json::Value> {
        let timestamp = Utc::now().timestamp_millis();
        let mut nonce_bytes = [0u8; 16];
        rand::rng().fill(&mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);
        let signature = self.authenticate_signature(timestamp, &nonce)?;

        Ok(json!({
            "key": self.key,
            "timestamp": timestamp,
            "nonce": nonce,
            "signature": signature,
            "passphrase": self.passphrase,
        }))
    }

    fn authenticate_signature(&self, timestamp: i64, nonce: &str) -> ConnectionResult<String> {
        let prehash = format!("{timestamp}{nonce}");
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes())
            .map_err(StreamConnectionError::InvalidSecretHmac)?;
        mac.update(prehash.as_bytes());
        let mac = mac.finalize().into_bytes();

        Ok(BASE64.encode(mac))
    }
}

pub(super) struct LnmStreamRepo {
    config: StreamClientConfig,
    event_loop_handle: SyncMutex<Option<JoinHandle<()>>>,
    disconnect_tx: DisconnectTransmiter,
    request_tx: RequestTransmiter,
    response_tx: ResponseTransmiter,
    connection_status_manager: Arc<StreamConnectionStatusManager>,
    credentials: Arc<AsyncMutex<Option<StreamCredentials>>>,
    subscriptions: Arc<AsyncMutex<HashMap<StreamTopic, TopicStatus>>>,
}

impl LnmStreamRepo {
    pub async fn new(config: StreamClientConfig) -> Result<Self> {
        let (disconnect_tx, disconnect_rx) = mpsc::channel::<()>(1);
        let (request_tx, request_rx) = mpsc::channel::<(
            StreamJsonRpcRequest,
            oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
        )>(1_000);
        let (response_tx, _) = broadcast::channel::<StreamUpdate>(10_000);
        let credentials = Arc::new(AsyncMutex::new(None));
        let subscriptions = Arc::new(AsyncMutex::new(HashMap::new()));

        let (event_loop_handle, connection_status_manager) = StreamEventLoop::try_spawn(
            config.clone(),
            disconnect_rx,
            request_rx,
            response_tx.clone(),
            credentials.clone(),
            subscriptions.clone(),
        )
        .await
        .map_err(StreamApiError::FailedToSpawnEventLoop)?;

        Ok(Self {
            config,
            event_loop_handle: SyncMutex::new(Some(event_loop_handle)),
            disconnect_tx,
            request_tx,
            response_tx,
            connection_status_manager,
            credentials,
            subscriptions,
        })
    }

    async fn evaluate_connection_status(&self) -> Result<()> {
        let connection_status = self.connection_status_manager.snapshot();

        if connection_status.is_connected() {
            return Ok(());
        }

        Err(StreamApiError::BadConnectionStatus(connection_status))
    }

    async fn send_request(&self, request: StreamJsonRpcRequest) -> Result<StreamJsonRpcResult> {
        self.evaluate_connection_status().await?;

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.request_tx
            .send((request, oneshot_tx))
            .await
            .map_err(|_| StreamApiError::RequestQueueClosed)?;

        oneshot_rx
            .await
            .map_err(StreamApiError::ReceiveResponse)?
            .map_err(StreamApiError::RequestFailed)
    }

    fn try_consume_handle(&self) -> Option<JoinHandle<()>> {
        self.event_loop_handle
            .lock()
            .expect("`LnmStreamRepo::event_loop_handle` mutex can't be poisoned")
            .take()
    }
}

impl crate::sealed::Sealed for LnmStreamRepo {}

#[async_trait]
impl StreamRepository for LnmStreamRepo {
    async fn is_connected(&self) -> bool {
        self.connection_status_manager.is_connected()
    }

    async fn connection_status(&self) -> StreamConnectionStatus {
        self.connection_status_manager.snapshot()
    }

    async fn hello(&self, client_name: &str, client_version: &str) -> Result<HelloResult> {
        let request = StreamJsonRpcRequest::new(
            StreamJsonRpcReqMethod::Hello,
            Some(json!({
                "clientName": client_name,
                "clientVersion": client_version,
            })),
        );

        match self.send_request(request).await? {
            StreamJsonRpcResult::Hello(result) => Ok(result),
            _ => Err(StreamApiError::InvalidRpcResult("hello")),
        }
    }

    async fn ping(&self) -> Result<()> {
        let request = StreamJsonRpcRequest::new(StreamJsonRpcReqMethod::Ping, None);

        match self.send_request(request).await? {
            StreamJsonRpcResult::Pong(_) => Ok(()),
            _ => Err(StreamApiError::InvalidRpcResult("ping")),
        }
    }

    async fn time(&self) -> Result<DateTime<Utc>> {
        let request = StreamJsonRpcRequest::new(StreamJsonRpcReqMethod::Time, None);

        match self.send_request(request).await? {
            StreamJsonRpcResult::Time(result) => Ok(result.time()),
            _ => Err(StreamApiError::InvalidRpcResult("time")),
        }
    }

    async fn authenticate(
        &self,
        key: &str,
        secret: &str,
        passphrase: &str,
    ) -> Result<AuthenticateResult> {
        let credentials = StreamCredentials::new(key, secret, passphrase);
        let params = credentials
            .authenticate_params()
            .map_err(StreamApiError::RequestFailed)?;
        let request = StreamJsonRpcRequest::new(StreamJsonRpcReqMethod::Authenticate, Some(params));

        match self.send_request(request).await? {
            StreamJsonRpcResult::Authenticate(result) => {
                let mut credentials_lock = self.credentials.lock().await;
                if result.authenticated() {
                    *credentials_lock = Some(credentials);
                } else {
                    *credentials_lock = None;
                }

                Ok(result)
            }
            _ => Err(StreamApiError::InvalidRpcResult("authenticate")),
        }
    }

    async fn whoami(&self) -> Result<WhoamiResult> {
        let request = StreamJsonRpcRequest::new(StreamJsonRpcReqMethod::Whoami, None);

        match self.send_request(request).await? {
            StreamJsonRpcResult::Whoami(result) => Ok(result),
            _ => Err(StreamApiError::InvalidRpcResult("whoami")),
        }
    }

    async fn subscribe(&self, topics: Vec<StreamTopic>) -> Result<()> {
        self.evaluate_connection_status().await?;

        let topics: HashSet<StreamTopic> = topics.into_iter().collect();
        if topics.is_empty() {
            return Ok(());
        }

        let mut subscriptions_lock = self.subscriptions.lock().await;
        let mut topics_to_subscribe = Vec::new();

        for topic in topics {
            match subscriptions_lock.get(&topic) {
                Some(TopicStatus::Subscribed | TopicStatus::SubscriptionPending) => continue,
                Some(TopicStatus::UnsubscriptionPending) => {
                    return Err(StreamApiError::SubscribeWithUnsubscriptionPending(topic));
                }
                None => {
                    topics_to_subscribe.push(topic.clone());
                    subscriptions_lock.insert(topic, TopicStatus::SubscriptionPending);
                }
            }
        }

        drop(subscriptions_lock);

        if topics_to_subscribe.is_empty() {
            return Ok(());
        }

        let request = StreamJsonRpcRequest::new(
            StreamJsonRpcReqMethod::Subscribe,
            Some(topics_param(topics_to_subscribe.clone())),
        );

        let result = self.send_request(request).await;
        let mut subscriptions_lock = self.subscriptions.lock().await;

        let subscribed = match result {
            Ok(StreamJsonRpcResult::Subscribe { subscribed, .. }) => subscribed,
            Ok(_) => return Err(StreamApiError::InvalidRpcResult("subscribe")),
            Err(err) => {
                for topic in topics_to_subscribe {
                    subscriptions_lock.remove(&topic);
                }
                return Err(err);
            }
        };

        let success = topics_match(&topics_to_subscribe, &subscribed);
        for topic in topics_to_subscribe {
            let topic_status = subscriptions_lock
                .get(&topic)
                .ok_or_else(|| StreamApiError::InvalidSubscriptionsTopicNotFound(topic.clone()))?;

            if *topic_status != TopicStatus::SubscriptionPending {
                return Err(StreamApiError::InvalidSubscriptionsTopicStatus {
                    topic: topic.clone(),
                    status: *topic_status,
                });
            }

            if success {
                subscriptions_lock.insert(topic, TopicStatus::Subscribed);
            } else {
                subscriptions_lock.remove(&topic);
            }
        }

        Ok(())
    }

    async fn unsubscribe(&self, topics: Vec<StreamTopic>) -> Result<()> {
        self.evaluate_connection_status().await?;

        let topics: HashSet<StreamTopic> = topics.into_iter().collect();
        if topics.is_empty() {
            return Ok(());
        }

        let mut subscriptions_lock = self.subscriptions.lock().await;
        let mut topics_to_unsubscribe = Vec::new();

        for topic in topics {
            match subscriptions_lock.get(&topic) {
                Some(TopicStatus::Subscribed) => {
                    topics_to_unsubscribe.push(topic.clone());
                    subscriptions_lock.insert(topic, TopicStatus::UnsubscriptionPending);
                }
                Some(TopicStatus::SubscriptionPending) => {
                    return Err(StreamApiError::UnsubscribeWithSubscriptionPending(topic));
                }
                Some(TopicStatus::UnsubscriptionPending) | None => continue,
            }
        }

        drop(subscriptions_lock);

        if topics_to_unsubscribe.is_empty() {
            return Ok(());
        }

        let request = StreamJsonRpcRequest::new(
            StreamJsonRpcReqMethod::Unsubscribe,
            Some(topics_param(topics_to_unsubscribe.clone())),
        );

        let result = self.send_request(request).await;
        let mut subscriptions_lock = self.subscriptions.lock().await;

        let unsubscribed = match result {
            Ok(StreamJsonRpcResult::Unsubscribe { unsubscribed, .. }) => unsubscribed,
            Ok(_) => return Err(StreamApiError::InvalidRpcResult("unsubscribe")),
            Err(err) => {
                for topic in topics_to_unsubscribe {
                    subscriptions_lock.insert(topic, TopicStatus::Subscribed);
                }
                return Err(err);
            }
        };

        let success = topics_match(&topics_to_unsubscribe, &unsubscribed);
        for topic in topics_to_unsubscribe {
            let topic_status = subscriptions_lock
                .get(&topic)
                .ok_or_else(|| StreamApiError::InvalidSubscriptionsTopicNotFound(topic.clone()))?;

            if *topic_status != TopicStatus::UnsubscriptionPending {
                return Err(StreamApiError::InvalidSubscriptionsTopicStatus {
                    topic: topic.clone(),
                    status: *topic_status,
                });
            }

            if success {
                subscriptions_lock.remove(&topic);
            } else {
                subscriptions_lock.insert(topic, TopicStatus::Subscribed);
            }
        }

        Ok(())
    }

    async fn unsubscribe_all(&self) -> Result<Vec<StreamTopic>> {
        self.evaluate_connection_status().await?;

        let request = StreamJsonRpcRequest::new(StreamJsonRpcReqMethod::UnsubscribeAll, None);
        let unsubscribed = match self.send_request(request).await? {
            StreamJsonRpcResult::UnsubscribeAll { unsubscribed, .. } => unsubscribed,
            _ => return Err(StreamApiError::InvalidRpcResult("unsubscribeAll")),
        };

        let mut subscriptions_lock = self.subscriptions.lock().await;
        subscriptions_lock.clear();

        Ok(unsubscribed)
    }

    async fn subscriptions(&self) -> HashSet<StreamTopic> {
        let subscriptions = self.subscriptions.lock().await;
        subscriptions
            .iter()
            .filter_map(|(topic, status)| {
                if let TopicStatus::Subscribed = status {
                    Some(topic.clone())
                } else {
                    None
                }
            })
            .collect::<HashSet<StreamTopic>>()
    }

    async fn receiver(&self) -> Result<ResponseReceiver> {
        self.evaluate_connection_status().await?;

        Ok(self.response_tx.subscribe())
    }

    async fn disconnect(&self) -> Result<()> {
        let mut handle = match self.try_consume_handle() {
            Some(handle) if !handle.is_finished() => handle,
            _ => {
                let status = self.connection_status_manager.snapshot();
                return Err(StreamApiError::WebSocketNotConnected(status));
            }
        };

        self.connection_status_manager
            .update(StreamConnectionStatus::DisconnectInitiated);

        self.disconnect_tx.send(()).await.map_err(|e| {
            handle.abort();
            StreamApiError::SendDisconnectRequest(e)
        })?;

        tokio::select! {
            join_res = &mut handle => {
                join_res.map_err(StreamApiError::TaskJoin)
            }
            _ = time::sleep(self.config.disconnect_timeout()) => {
                handle.abort();
                Err(StreamApiError::DisconnectTimeout)
            }
        }
    }
}

impl Drop for LnmStreamRepo {
    fn drop(&mut self) {
        if let Ok(mut handle) = self.event_loop_handle.lock()
            && let Some(join_handle) = handle.take()
        {
            join_handle.abort();
        }
    }
}

#[cfg(test)]
mod tests;
