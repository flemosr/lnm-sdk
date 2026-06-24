use std::{collections::HashMap, sync::Arc};

use tokio::{
    sync::{broadcast, mpsc, oneshot},
    task::JoinHandle,
    time,
};

use crate::stream::v1::config::StreamClientConfig;

use super::super::{
    error::{ConnectionResult, StreamConnectionError},
    models::{StreamJsonRpcMessage, StreamJsonRpcRequest, StreamJsonRpcResult, StreamUpdate},
    state::{StreamConnectionStatus, StreamConnectionStatusManager},
};

mod connection;

use connection::{LnmStreamResponse, StreamApiConnection};

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

pub(super) struct StreamEventLoop {
    config: StreamClientConfig,
    ws: StreamApiConnection,
    disconnect_rx: DisconnectReceiver,
    request_rx: RequestReceiver,
    response_tx: ResponseTransmiter,
    connection_status_manager: Arc<StreamConnectionStatusManager>,
}

impl StreamEventLoop {
    async fn new(
        config: StreamClientConfig,
        disconnect_rx: DisconnectReceiver,
        request_rx: RequestReceiver,
        response_tx: ResponseTransmiter,
        connection_status_manager: Arc<StreamConnectionStatusManager>,
    ) -> ConnectionResult<Self> {
        let ws = StreamApiConnection::new(config.endpoint()).await?;

        Ok(Self {
            config,
            ws,
            disconnect_rx,
            request_rx,
            response_tx,
            connection_status_manager,
        })
    }

    async fn run(mut self) {
        let mut ws = self.ws;
        let mut pending: PendingMap = HashMap::new();

        let handler = || {
            let pending = &mut pending;
            let responses_tx = &self.response_tx;
            let heartbeat_interval = self.config.heartbeat_interval();

            async move {
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
                                    let response_id = match &json_rpc_message {
                                        StreamJsonRpcMessage::Response { id, .. } => Some(id.clone()),
                                        StreamJsonRpcMessage::Subscription(_) => None,
                                    };

                                    if let Some(id) = response_id {
                                        if let Some((req, oneshot_tx)) = pending.remove(&id) {
                                            let result = json_rpc_message
                                                .into_rpc_result(&req)
                                                .and_then(|result| result.ok_or_else(|| StreamConnectionError::UnexpectedJsonRpcResult {
                                                    method: req.method().to_string(),
                                                    result: serde_json::Value::Null,
                                                }));

                                            let _ = oneshot_tx.send(result);
                                        }
                                    } else if let StreamJsonRpcMessage::Subscription(update) = json_rpc_message {
                                        let _ = responses_tx.send(update);
                                    }
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
        };

        let new_connection_status = match handler().await {
            Ok(_) => StreamConnectionStatus::Disconnected,
            Err(e) => StreamConnectionStatus::Failed(Arc::new(e)),
        };

        self.connection_status_manager.update(new_connection_status);

        for (_, (_, oneshot_tx)) in pending {
            let _ = oneshot_tx.send(Err(StreamConnectionError::ServerRequestedClose));
        }

        let connection_status = self.connection_status_manager.snapshot();
        let _ = self.response_tx.send(connection_status.into());
    }

    pub async fn try_spawn(
        config: StreamClientConfig,
        disconnect_rx: DisconnectReceiver,
        request_rx: RequestReceiver,
        response_tx: ResponseTransmiter,
    ) -> ConnectionResult<(JoinHandle<()>, Arc<StreamConnectionStatusManager>)> {
        let connection_status_manager = StreamConnectionStatusManager::new();

        let event_loop = Self::new(
            config,
            disconnect_rx,
            request_rx,
            response_tx,
            connection_status_manager.clone(),
        )
        .await?;

        let event_loop_handle = tokio::spawn(event_loop.run());

        Ok((event_loop_handle, connection_status_manager))
    }
}
