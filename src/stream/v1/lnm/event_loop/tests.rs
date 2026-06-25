use std::{collections::VecDeque, future, sync::Mutex as SyncMutex};

use serde_json::json;
use tokio::time;

use super::*;
use crate::stream::v1::models::metadata::StreamResponseMetadata;

struct FakeConnector {
    connections: SyncMutex<VecDeque<FakeConnection>>,
}

impl FakeConnector {
    fn new(connections: Vec<FakeConnection>) -> Self {
        Self {
            connections: SyncMutex::new(connections.into()),
        }
    }
}

#[async_trait]
impl StreamConnector for FakeConnector {
    async fn connect(&self, _endpoint: &str) -> ConnectionResult<Box<dyn StreamConnectionIo>> {
        let connection = self
            .connections
            .lock()
            .expect("fake connection queue mutex must not be poisoned")
            .pop_front()
            .expect("fake connection must be available");

        Ok(Box::new(connection))
    }
}

enum FakeReadMode {
    FailImmediately(Option<StreamConnectionError>),
    FailAfterRequest(Option<StreamConnectionError>),
    Idle,
}

struct FakeConnection {
    read_mode: FakeReadMode,
    sent_requests: Arc<SyncMutex<Vec<StreamJsonRpcRequest>>>,
    last_request: Option<StreamJsonRpcRequest>,
    close_sent: bool,
}

impl FakeConnection {
    fn fail_immediately(
        error: StreamConnectionError,
        sent_requests: Arc<SyncMutex<Vec<StreamJsonRpcRequest>>>,
    ) -> Self {
        Self {
            read_mode: FakeReadMode::FailImmediately(Some(error)),
            sent_requests,
            last_request: None,
            close_sent: false,
        }
    }

    fn fail_after_request(
        error: StreamConnectionError,
        sent_requests: Arc<SyncMutex<Vec<StreamJsonRpcRequest>>>,
    ) -> Self {
        Self {
            read_mode: FakeReadMode::FailAfterRequest(Some(error)),
            sent_requests,
            last_request: None,
            close_sent: false,
        }
    }

    fn idle(sent_requests: Arc<SyncMutex<Vec<StreamJsonRpcRequest>>>) -> Self {
        Self {
            read_mode: FakeReadMode::Idle,
            sent_requests,
            last_request: None,
            close_sent: false,
        }
    }
}

#[async_trait]
impl StreamConnectionIo for FakeConnection {
    async fn send_json_rpc(&mut self, req: &StreamJsonRpcRequest) -> ConnectionResult<()> {
        self.sent_requests
            .lock()
            .expect("sent request mutex must not be poisoned")
            .push(req.clone());
        self.last_request = Some(req.clone());

        Ok(())
    }

    async fn send_close(&mut self) -> ConnectionResult<()> {
        self.close_sent = true;

        Ok(())
    }

    async fn send_pong(&mut self, _payload: Vec<u8>) -> ConnectionResult<()> {
        Ok(())
    }

    async fn send_ping(&mut self) -> ConnectionResult<()> {
        Ok(())
    }

    async fn read_response(&mut self) -> ConnectionResult<LnmStreamResponse> {
        if self.close_sent {
            return Ok(LnmStreamResponse::Close);
        }

        match &mut self.read_mode {
            FakeReadMode::FailImmediately(error) => {
                if let Some(error) = error.take() {
                    return Err(error);
                }
            }
            FakeReadMode::FailAfterRequest(error) => {
                if self.last_request.is_some()
                    && let Some(error) = error.take()
                {
                    self.last_request = None;
                    return Err(error);
                }
            }
            FakeReadMode::Idle => {}
        }

        if let Some(request) = self.last_request.take() {
            return Ok(LnmStreamResponse::JsonRpc(response_for_request(&request)));
        }

        future::pending().await
    }
}

fn response_for_request(request: &StreamJsonRpcRequest) -> StreamJsonRpcMessage {
    let result = match request.method() {
        StreamJsonRpcReqMethod::Authenticate => json!({
            "authenticated": true,
            "permissions": ["read", "trade"],
        }),
        StreamJsonRpcReqMethod::Subscribe => {
            let topics = request_topics(request);
            json!({ "subscribed": topics })
        }
        StreamJsonRpcReqMethod::Ping => json!("pong"),
        method => panic!("unexpected fake request method: {method}"),
    };

    StreamJsonRpcMessage::Response {
        id: request.id().clone(),
        result: Ok(result),
        metadata: StreamResponseMetadata::new(None, None, None, None),
    }
}

fn request_topics(request: &StreamJsonRpcRequest) -> Vec<StreamTopic> {
    let request_json: serde_json::Value = serde_json::from_slice(
        &request
            .try_to_bytes()
            .expect("request must serialize to json"),
    )
    .expect("request bytes must be json");

    serde_json::from_value(request_json["params"]["topics"].clone())
        .expect("request topics must decode")
}

async fn receive_status(response_rx: &mut ResponseReceiver) -> StreamConnectionStatus {
    loop {
        let update = time::timeout(Duration::from_secs(1), response_rx.recv())
            .await
            .expect("status update must arrive before timeout")
            .expect("response channel must remain open");

        if let StreamUpdate::ConnectionStatus(status) = update {
            return status;
        }
    }
}

fn test_event_loop(
    ws: FakeConnection,
    connector: Arc<dyn StreamConnector>,
    credentials: Arc<AsyncMutex<Option<StreamCredentials>>>,
    subscriptions: Arc<AsyncMutex<HashMap<StreamTopic, TopicStatus>>>,
) -> (
    StreamEventLoop,
    DisconnectTransmiter,
    RequestTransmiter,
    ResponseReceiver,
) {
    let (disconnect_tx, disconnect_rx) = mpsc::channel::<()>(1);
    let (request_tx, request_rx) = mpsc::channel::<(
        StreamJsonRpcRequest,
        oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
    )>(16);
    let (response_tx, response_rx) = broadcast::channel::<StreamUpdate>(16);

    let event_loop = StreamEventLoop {
        config: StreamClientConfig::default()
            .with_reconnect_initial_backoff(Duration::from_millis(1))
            .with_reconnect_max_backoff(Duration::from_millis(1))
            .with_reconnect_max_attempts(Some(1)),
        ws: Some(Box::new(ws)),
        connector,
        disconnect_rx,
        request_rx,
        response_tx,
        connection_status_manager: StreamConnectionStatusManager::new(),
        credentials,
        subscriptions,
    };

    (event_loop, disconnect_tx, request_tx, response_rx)
}

#[tokio::test]
async fn reconnect_reauthenticates_and_resubscribes_desired_topics() {
    let first_sent = Arc::new(SyncMutex::new(Vec::new()));
    let reconnect_sent = Arc::new(SyncMutex::new(Vec::new()));
    let first_connection =
        FakeConnection::fail_immediately(StreamConnectionError::NoServerPong, first_sent);
    let reconnect_connection = FakeConnection::idle(reconnect_sent.clone());
    let connector = Arc::new(FakeConnector::new(vec![reconnect_connection]));
    let credentials = Arc::new(AsyncMutex::new(Some(StreamCredentials::new(
        "key",
        "secret",
        "passphrase",
    ))));
    let subscriptions = Arc::new(AsyncMutex::new(HashMap::from([
        (
            StreamTopic::FuturesInverseBtcUsdLastPrice,
            TopicStatus::Subscribed,
        ),
        (
            StreamTopic::WalletDeposit,
            TopicStatus::UnsubscriptionPending,
        ),
        (
            StreamTopic::FuturesInverseBtcUsdBuckets,
            TopicStatus::SubscriptionPending,
        ),
    ])));
    let (event_loop, disconnect_tx, _, mut response_rx) =
        test_event_loop(first_connection, connector, credentials, subscriptions);
    let handle = tokio::spawn(event_loop.run());

    assert!(matches!(
        receive_status(&mut response_rx).await,
        StreamConnectionStatus::Reconnecting
    ));
    assert!(matches!(
        receive_status(&mut response_rx).await,
        StreamConnectionStatus::Connected
    ));

    disconnect_tx
        .send(())
        .await
        .expect("disconnect request must be sent");
    handle.await.expect("event loop task must complete");

    let sent = reconnect_sent
        .lock()
        .expect("sent request mutex must not be poisoned");
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].method(), &StreamJsonRpcReqMethod::Authenticate);
    assert_eq!(sent[1].method(), &StreamJsonRpcReqMethod::Subscribe);
    assert!(topics_match(
        &request_topics(&sent[1]),
        &[
            StreamTopic::FuturesInverseBtcUsdLastPrice,
            StreamTopic::WalletDeposit,
        ]
    ));
}

#[tokio::test]
async fn reconnect_fails_in_flight_requests_before_restoring_connection() {
    let first_sent = Arc::new(SyncMutex::new(Vec::new()));
    let reconnect_sent = Arc::new(SyncMutex::new(Vec::new()));
    let first_connection =
        FakeConnection::fail_after_request(StreamConnectionError::NoServerPong, first_sent);
    let reconnect_connection = FakeConnection::idle(reconnect_sent);
    let connector = Arc::new(FakeConnector::new(vec![reconnect_connection]));
    let credentials = Arc::new(AsyncMutex::new(None));
    let subscriptions = Arc::new(AsyncMutex::new(HashMap::new()));
    let (event_loop, disconnect_tx, request_tx, mut response_rx) =
        test_event_loop(first_connection, connector, credentials, subscriptions);
    let handle = tokio::spawn(event_loop.run());
    let (oneshot_tx, oneshot_rx) = oneshot::channel();

    request_tx
        .send((
            StreamJsonRpcRequest::new(StreamJsonRpcReqMethod::Ping, None),
            oneshot_tx,
        ))
        .await
        .expect("request must be sent");

    let error = time::timeout(Duration::from_secs(1), oneshot_rx)
        .await
        .expect("request must fail before timeout")
        .expect("request sender must not be dropped")
        .expect_err("request must fail on connection interruption");
    assert!(matches!(
        error,
        StreamConnectionError::ConnectionInterrupted
    ));
    assert!(matches!(
        receive_status(&mut response_rx).await,
        StreamConnectionStatus::Reconnecting
    ));
    assert!(matches!(
        receive_status(&mut response_rx).await,
        StreamConnectionStatus::Connected
    ));

    disconnect_tx
        .send(())
        .await
        .expect("disconnect request must be sent");
    handle.await.expect("event loop task must complete");
}
