use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use serde_json::Value;
use tokio::{
    sync::{Mutex as AsyncMutex, broadcast, mpsc, oneshot},
    task::JoinHandle,
    time,
};

use crate::stream::v1::config::StreamClientConfig;

use super::super::{
    error::{ConnectionResult, StreamConnectionError},
    models::{
        rpc::{
            StreamJsonRpcMessage, StreamJsonRpcReqMethod, StreamJsonRpcRequest, StreamJsonRpcResult,
        },
        topic::{StreamTopic, topics_match, topics_param},
        update::StreamUpdate,
    },
    state::{StreamConnectionStatus, StreamConnectionStatusManager},
};
use super::{StreamCredentials, TopicStatus};

mod connection;

use connection::{LnmStreamResponse, StreamApiConnection, StreamConnectionIo};

type PendingMap = HashMap<
    String,
    (
        StreamJsonRpcRequest,
        oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
    ),
>;

pub(super) type DisconnectTransmiter = mpsc::Sender<()>;
type DisconnectReceiver = mpsc::Receiver<()>;

pub(super) type RequestTransmiter = mpsc::Sender<(
    StreamJsonRpcRequest,
    oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
)>;
type RequestReceiver = mpsc::Receiver<(
    StreamJsonRpcRequest,
    oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
)>;

pub(super) type ResponseTransmiter = broadcast::Sender<StreamUpdate>;
pub(super) type ResponseReceiver = broadcast::Receiver<StreamUpdate>;

#[async_trait]
trait StreamConnector: Send + Sync {
    async fn connect(&self, endpoint: &str) -> ConnectionResult<Box<dyn StreamConnectionIo>>;
}

struct LnmStreamConnector;

#[async_trait]
impl StreamConnector for LnmStreamConnector {
    async fn connect(&self, endpoint: &str) -> ConnectionResult<Box<dyn StreamConnectionIo>> {
        Ok(Box::new(StreamApiConnection::new(endpoint).await?))
    }
}

pub(super) struct StreamEventLoop {
    config: StreamClientConfig,
    ws: Option<Box<dyn StreamConnectionIo>>,
    connector: Arc<dyn StreamConnector>,
    disconnect_rx: DisconnectReceiver,
    request_rx: RequestReceiver,
    response_tx: ResponseTransmiter,
    connection_status_manager: Arc<StreamConnectionStatusManager>,
    credentials: Arc<AsyncMutex<Option<StreamCredentials>>>,
    subscriptions: Arc<AsyncMutex<HashMap<StreamTopic, TopicStatus>>>,
}

impl StreamEventLoop {
    async fn new(
        config: StreamClientConfig,
        disconnect_rx: DisconnectReceiver,
        request_rx: RequestReceiver,
        response_tx: ResponseTransmiter,
        connection_status_manager: Arc<StreamConnectionStatusManager>,
        credentials: Arc<AsyncMutex<Option<StreamCredentials>>>,
        subscriptions: Arc<AsyncMutex<HashMap<StreamTopic, TopicStatus>>>,
    ) -> ConnectionResult<Self> {
        let connector: Arc<dyn StreamConnector> = Arc::new(LnmStreamConnector);
        let ws = connector.connect(config.endpoint()).await?;

        Ok(Self {
            config,
            ws: Some(ws),
            connector,
            disconnect_rx,
            request_rx,
            response_tx,
            connection_status_manager,
            credentials,
            subscriptions,
        })
    }

    async fn run(mut self) {
        let mut ws = self
            .ws
            .take()
            .expect("`StreamEventLoop` must start with an active connection");
        let mut pending: PendingMap = HashMap::new();

        loop {
            match self.run_connected(ws.as_mut(), &mut pending).await {
                Ok(()) => {
                    self.fail_pending(&mut pending);
                    self.fail_queued_requests();
                    self.update_connection_status(StreamConnectionStatus::Disconnected);
                    break;
                }
                Err(err) => {
                    self.fail_pending(&mut pending);

                    if !err.is_reconnectable() {
                        self.update_connection_status(StreamConnectionStatus::Failed(Arc::new(
                            err,
                        )));
                        self.fail_queued_requests();
                        break;
                    }

                    self.update_connection_status(StreamConnectionStatus::Reconnecting);
                    self.fail_queued_requests();

                    match self.reconnect().await {
                        Ok(Some(new_ws)) => {
                            ws = new_ws;
                            self.update_connection_status(StreamConnectionStatus::Connected);
                        }
                        Ok(None) => {
                            self.fail_queued_requests();
                            self.update_connection_status(StreamConnectionStatus::Disconnected);
                            break;
                        }
                        Err(err) => {
                            self.fail_queued_requests();
                            self.update_connection_status(StreamConnectionStatus::Failed(
                                Arc::new(err),
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn run_connected(
        &mut self,
        ws: &mut dyn StreamConnectionIo,
        pending: &mut PendingMap,
    ) -> ConnectionResult<()> {
        let heartbeat_interval = self.config.heartbeat_interval();
        let new_heartbeat_timer = || Box::pin(time::sleep(heartbeat_interval));
        let mut heartbeat_timer = new_heartbeat_timer();
        let mut waiting_for_pong = false;
        let mut close_initiated = false;

        loop {
            tokio::select! {
                Some(_) = self.disconnect_rx.recv() => {
                    close_initiated = true;
                    heartbeat_timer = new_heartbeat_timer();

                    ws.send_close().await?;
                }
                Some((json_rpc_req, oneshot_tx)) = self.request_rx.recv() => {
                    ws.send_json_rpc(&json_rpc_req).await?;
                    pending.insert(json_rpc_req.id().clone(), (json_rpc_req, oneshot_tx));
                }
                read_response_result = ws.read_response() => {
                    waiting_for_pong = false;
                    heartbeat_timer = new_heartbeat_timer();

                    match read_response_result? {
                        LnmStreamResponse::JsonRpc(json_rpc_message) => {
                            self.handle_json_rpc_message(json_rpc_message, pending);
                        }
                        LnmStreamResponse::Ping(payload) => {
                            ws.send_pong(payload).await?;
                        }
                        LnmStreamResponse::Close => {
                            if close_initiated {
                                return Ok(());
                            }

                            let _ = ws.send_close().await;

                            return Err(StreamConnectionError::ServerRequestedClose);
                        }
                        LnmStreamResponse::Pong => {}
                    };
                }
                _ = &mut heartbeat_timer => {
                    if close_initiated {
                        return Err(StreamConnectionError::NoServerCloseConfirmation);
                    }

                    if waiting_for_pong {
                        return Err(StreamConnectionError::NoServerPong);
                    }

                    ws.send_ping().await?;

                    waiting_for_pong = true;
                    heartbeat_timer = new_heartbeat_timer();
                }
            };
        }
    }

    fn handle_json_rpc_message(
        &self,
        json_rpc_message: StreamJsonRpcMessage,
        pending: &mut PendingMap,
    ) {
        let response_id = match &json_rpc_message {
            StreamJsonRpcMessage::Response { id, .. } => Some(id.clone()),
            StreamJsonRpcMessage::Subscription(_) => None,
        };

        if let Some(id) = response_id {
            if let Some((req, oneshot_tx)) = pending.remove(&id) {
                let result = json_rpc_message.into_rpc_result(&req).and_then(|result| {
                    result.ok_or_else(|| StreamConnectionError::UnexpectedJsonRpcResult {
                        method: req.method().to_string(),
                        result: Value::Null,
                    })
                });

                let _ = oneshot_tx.send(result);
            }
        } else if let StreamJsonRpcMessage::Subscription(update) = json_rpc_message {
            let _ = self.response_tx.send(update);
        }
    }

    async fn reconnect(&mut self) -> ConnectionResult<Option<Box<dyn StreamConnectionIo>>> {
        let max_attempts = self.config.reconnect_max_attempts();
        if max_attempts == Some(0) {
            return Err(StreamConnectionError::ReconnectAttemptsExhausted);
        }

        let mut attempt = 0;
        let mut backoff = self.config.reconnect_initial_backoff();
        let max_backoff = self.config.reconnect_max_backoff();
        let reconnect_attempts_exhausted =
            |attempt: usize| max_attempts.is_some_and(|max_attempts| attempt >= max_attempts);
        let next_reconnect_backoff = |current: Duration| {
            current
                .checked_mul(2)
                .unwrap_or(max_backoff)
                .min(max_backoff)
        };

        loop {
            attempt += 1;

            let connect_result = tokio::select! {
                Some(_) = self.disconnect_rx.recv() => return Ok(None),
                result = self.connector.connect(self.config.endpoint()) => result,
            };

            let mut ws = match connect_result {
                Ok(ws) => ws,
                Err(err) => {
                    if reconnect_attempts_exhausted(attempt) {
                        return Err(err);
                    }

                    if self.wait_for_reconnect_backoff(backoff).await {
                        return Ok(None);
                    }

                    backoff = next_reconnect_backoff(backoff);
                    continue;
                }
            };

            match self.restore_connection(ws.as_mut()).await {
                Ok(()) => return Ok(Some(ws)),
                Err(err) => {
                    if !err.is_reconnectable() || reconnect_attempts_exhausted(attempt) {
                        return Err(err);
                    }

                    if self.wait_for_reconnect_backoff(backoff).await {
                        return Ok(None);
                    }

                    backoff = next_reconnect_backoff(backoff);
                }
            }
        }
    }

    async fn restore_connection(
        &mut self,
        ws: &mut dyn StreamConnectionIo,
    ) -> ConnectionResult<()> {
        let credentials = { self.credentials.lock().await.clone() };
        if let Some(credentials) = credentials {
            let request = StreamJsonRpcRequest::new(
                StreamJsonRpcReqMethod::Authenticate,
                Some(credentials.authenticate_params()?),
            );

            match self.send_control_request(ws, request).await? {
                StreamJsonRpcResult::Authenticate(result) if result.authenticated() => {}
                StreamJsonRpcResult::Authenticate(_) => {
                    return Err(StreamConnectionError::ReauthenticationRejected);
                }
                _ => {
                    return Err(StreamConnectionError::UnexpectedJsonRpcResult {
                        method: StreamJsonRpcReqMethod::Authenticate.to_string(),
                        result: Value::Null,
                    });
                }
            }
        }

        let topics = self.desired_topics_for_reconnect().await;
        if topics.is_empty() {
            return Ok(());
        }

        let request = StreamJsonRpcRequest::new(
            StreamJsonRpcReqMethod::Subscribe,
            Some(topics_param(topics.clone())),
        );

        match self.send_control_request(ws, request).await? {
            StreamJsonRpcResult::Subscribe { subscribed, .. } => {
                if topics_match(&topics, &subscribed) {
                    Ok(())
                } else {
                    Err(StreamConnectionError::SubscriptionRestoreMismatch {
                        requested: topics,
                        subscribed,
                    })
                }
            }
            _ => Err(StreamConnectionError::UnexpectedJsonRpcResult {
                method: StreamJsonRpcReqMethod::Subscribe.to_string(),
                result: Value::Null,
            }),
        }
    }

    async fn send_control_request(
        &mut self,
        ws: &mut dyn StreamConnectionIo,
        request: StreamJsonRpcRequest,
    ) -> ConnectionResult<StreamJsonRpcResult> {
        ws.send_json_rpc(&request).await?;

        loop {
            match ws.read_response().await? {
                LnmStreamResponse::JsonRpc(json_rpc_message) => {
                    let response_id = match &json_rpc_message {
                        StreamJsonRpcMessage::Response { id, .. } => Some(id.clone()),
                        StreamJsonRpcMessage::Subscription(_) => None,
                    };

                    if response_id.as_ref() == Some(request.id()) {
                        return json_rpc_message
                            .into_rpc_result(&request)
                            .and_then(|result| {
                                result.ok_or_else(|| {
                                    StreamConnectionError::UnexpectedJsonRpcResult {
                                        method: request.method().to_string(),
                                        result: Value::Null,
                                    }
                                })
                            });
                    }

                    if let StreamJsonRpcMessage::Subscription(update) = json_rpc_message {
                        let _ = self.response_tx.send(update);
                    }
                }
                LnmStreamResponse::Ping(payload) => {
                    ws.send_pong(payload).await?;
                }
                LnmStreamResponse::Close => {
                    let _ = ws.send_close().await;

                    return Err(StreamConnectionError::ServerRequestedClose);
                }
                LnmStreamResponse::Pong => {}
            }
        }
    }

    async fn desired_topics_for_reconnect(&mut self) -> Vec<StreamTopic> {
        let subscriptions = self.subscriptions.lock().await;

        subscriptions
            .iter()
            .filter_map(|(topic, status)| match status {
                TopicStatus::Subscribed | TopicStatus::UnsubscriptionPending => Some(topic.clone()),
                TopicStatus::SubscriptionPending => None,
            })
            .collect()
    }

    async fn wait_for_reconnect_backoff(&mut self, backoff: Duration) -> bool {
        tokio::select! {
            Some(_) = self.disconnect_rx.recv() => true,
            _ = time::sleep(backoff) => false,
        }
    }

    fn update_connection_status(&self, new_status: StreamConnectionStatus) {
        self.connection_status_manager.update(new_status.clone());
        let _ = self.response_tx.send(new_status.into());
    }

    fn fail_pending(&self, pending: &mut PendingMap) {
        for (_, (_, oneshot_tx)) in pending.drain() {
            let _ = oneshot_tx.send(Err(StreamConnectionError::ConnectionInterrupted));
        }
    }

    fn fail_queued_requests(&mut self) {
        while let Ok((_, oneshot_tx)) = self.request_rx.try_recv() {
            let _ = oneshot_tx.send(Err(StreamConnectionError::ConnectionInterrupted));
        }
    }

    pub async fn try_spawn(
        config: StreamClientConfig,
        disconnect_rx: DisconnectReceiver,
        request_rx: RequestReceiver,
        response_tx: ResponseTransmiter,
        credentials: Arc<AsyncMutex<Option<StreamCredentials>>>,
        subscriptions: Arc<AsyncMutex<HashMap<StreamTopic, TopicStatus>>>,
    ) -> ConnectionResult<(JoinHandle<()>, Arc<StreamConnectionStatusManager>)> {
        let connection_status_manager = StreamConnectionStatusManager::new();

        let event_loop = Self::new(
            config,
            disconnect_rx,
            request_rx,
            response_tx,
            connection_status_manager.clone(),
            credentials,
            subscriptions,
        )
        .await?;

        let event_loop_handle = tokio::spawn(event_loop.run());

        Ok((event_loop_handle, connection_status_manager))
    }
}

#[cfg(test)]
mod tests;
