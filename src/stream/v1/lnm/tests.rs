use std::{collections::HashMap, sync::Mutex as SyncMutex, time::Duration};

use serde_json::{Value, json};
use tokio::sync::{Mutex as AsyncMutex, broadcast, mpsc::error::TryRecvError, oneshot};
use tokio::time;

use crate::stream::v1::models::{StreamJsonRpcMessage, StreamResponseMetadata};

use super::*;

type FakeRequestReceiver = mpsc::Receiver<(
    StreamJsonRpcRequest,
    oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
)>;

fn test_repo() -> (Arc<LnmStreamRepo>, FakeRequestReceiver) {
    let (disconnect_tx, _) = mpsc::channel::<()>(1);
    let (request_tx, request_rx) = mpsc::channel::<(
        StreamJsonRpcRequest,
        oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
    )>(16);
    let (response_tx, _) = broadcast::channel::<StreamUpdate>(16);

    let repo = LnmStreamRepo {
        config: StreamClientConfig::default(),
        event_loop_handle: SyncMutex::new(None),
        disconnect_tx,
        request_tx,
        response_tx,
        connection_status_manager: StreamConnectionStatusManager::new(),
        credentials: Arc::new(AsyncMutex::new(None)),
        subscriptions: Arc::new(AsyncMutex::new(HashMap::new())),
    };

    (Arc::new(repo), request_rx)
}

async fn receive_request(
    request_rx: &mut FakeRequestReceiver,
) -> (
    StreamJsonRpcRequest,
    oneshot::Sender<ConnectionResult<StreamJsonRpcResult>>,
) {
    time::timeout(Duration::from_secs(1), request_rx.recv())
        .await
        .expect("request must be sent before timeout")
        .expect("request channel must remain open")
}

fn assert_no_request(request_rx: &mut FakeRequestReceiver) {
    match request_rx.try_recv() {
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            panic!("request channel must remain open")
        }
        Ok((request, _)) => panic!("unexpected request: {}", request.method()),
    }
}

fn request_topics(request: &StreamJsonRpcRequest) -> Vec<StreamTopic> {
    let request_json: Value = serde_json::from_slice(
        &request
            .try_to_bytes()
            .expect("request must serialize to json"),
    )
    .expect("request bytes must be json");

    serde_json::from_value(request_json["params"]["topics"].clone())
        .expect("request topics must decode")
}

fn assert_topics(request: &StreamJsonRpcRequest, expected: &[StreamTopic]) {
    let topics = request_topics(request);

    assert!(topics_match(&topics, expected));
}

fn stream_metadata() -> StreamResponseMetadata {
    StreamResponseMetadata::new(None, None, None, None)
}

fn result_for_request(request: &StreamJsonRpcRequest, result: Value) -> StreamJsonRpcResult {
    StreamJsonRpcMessage::Response {
        id: request.id().clone(),
        result: Ok(result),
        metadata: stream_metadata(),
    }
    .into_rpc_result(request)
    .expect("response must decode")
    .expect("response must contain a result")
}

fn authenticate_result(request: &StreamJsonRpcRequest, authenticated: bool) -> StreamJsonRpcResult {
    result_for_request(
        request,
        json!({
            "authenticated": authenticated,
            "permissions": ["read", "trade"],
        }),
    )
}

async fn complete_subscribe(
    repo: Arc<LnmStreamRepo>,
    request_rx: &mut FakeRequestReceiver,
    topics: Vec<StreamTopic>,
) {
    let expected_topics = topics.clone();
    let subscribe_handle = tokio::spawn(async move { repo.subscribe(topics).await });
    let (request, response_tx) = receive_request(request_rx).await;

    assert_eq!(request.method(), &StreamJsonRpcReqMethod::Subscribe);
    assert_topics(&request, &expected_topics);

    response_tx
        .send(Ok(StreamJsonRpcResult::Subscribe {
            subscribed: expected_topics,
            metadata: stream_metadata(),
        }))
        .expect("subscribe response must be received");

    subscribe_handle
        .await
        .expect("subscribe task must complete")
        .expect("subscribe must succeed");
}

async fn complete_unsubscribe(
    repo: Arc<LnmStreamRepo>,
    request_rx: &mut FakeRequestReceiver,
    topics: Vec<StreamTopic>,
) {
    let expected_topics = topics.clone();
    let unsubscribe_handle = tokio::spawn(async move { repo.unsubscribe(topics).await });
    let (request, response_tx) = receive_request(request_rx).await;

    assert_eq!(request.method(), &StreamJsonRpcReqMethod::Unsubscribe);
    assert_topics(&request, &expected_topics);

    response_tx
        .send(Ok(StreamJsonRpcResult::Unsubscribe {
            unsubscribed: expected_topics,
            metadata: stream_metadata(),
        }))
        .expect("unsubscribe response must be received");

    unsubscribe_handle
        .await
        .expect("unsubscribe task must complete")
        .expect("unsubscribe must succeed");
}

#[test]
fn authenticate_signature_matches_documented_hmac_shape() {
    let credentials = StreamCredentials::new("key", "secret", "passphrase");
    let signature = credentials
        .authenticate_signature(1747035005657, "nonce123")
        .expect("signature must be generated");

    let mut mac = Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
    mac.update(b"1747035005657nonce123");
    let expected = BASE64.encode(mac.finalize().into_bytes());

    assert_eq!(signature, expected);
}

#[tokio::test]
async fn authenticate_stores_credentials_after_success() {
    let (repo, mut request_rx) = test_repo();
    let auth_repo = repo.clone();
    let auth_handle =
        tokio::spawn(async move { auth_repo.authenticate("key", "secret", "passphrase").await });
    let (request, response_tx) = receive_request(&mut request_rx).await;

    assert_eq!(request.method(), &StreamJsonRpcReqMethod::Authenticate);
    response_tx
        .send(Ok(authenticate_result(&request, true)))
        .expect("authenticate response must be received");

    let result = auth_handle
        .await
        .expect("authenticate task must complete")
        .expect("authenticate must succeed");
    assert!(result.authenticated());

    let credentials = repo.credentials.lock().await;
    let credentials = credentials
        .as_ref()
        .expect("credentials must be stored after successful authentication");
    assert_eq!(credentials.key, "key");
    assert_eq!(credentials.secret, "secret");
    assert_eq!(credentials.passphrase, "passphrase");
}

#[tokio::test]
async fn authenticate_clears_credentials_when_server_returns_unauthenticated() {
    let (repo, mut request_rx) = test_repo();
    *repo.credentials.lock().await = Some(StreamCredentials::new("old", "old", "old"));

    let auth_repo = repo.clone();
    let auth_handle =
        tokio::spawn(async move { auth_repo.authenticate("key", "secret", "passphrase").await });
    let (request, response_tx) = receive_request(&mut request_rx).await;

    assert_eq!(request.method(), &StreamJsonRpcReqMethod::Authenticate);
    response_tx
        .send(Ok(authenticate_result(&request, false)))
        .expect("authenticate response must be received");

    let result = auth_handle
        .await
        .expect("authenticate task must complete")
        .expect("authenticate must succeed");
    assert!(!result.authenticated());
    assert!(repo.credentials.lock().await.is_none());
}

#[tokio::test]
async fn subscribe_deduplicates_input_topics() {
    let (repo, mut request_rx) = test_repo();
    let topic = StreamTopic::FuturesInverseBtcUsdLastPrice;

    complete_subscribe(
        repo.clone(),
        &mut request_rx,
        vec![topic.clone(), topic.clone()],
    )
    .await;

    assert_eq!(repo.subscriptions().await, HashSet::from([topic]));
}

#[tokio::test]
async fn subscribe_skips_already_subscribed_topics() {
    let (repo, mut request_rx) = test_repo();
    let topic = StreamTopic::FuturesInverseBtcUsdLastPrice;

    complete_subscribe(repo.clone(), &mut request_rx, vec![topic.clone()]).await;

    repo.subscribe(vec![topic.clone()])
        .await
        .expect("duplicate subscribe must be idempotent");

    assert_no_request(&mut request_rx);
    assert_eq!(repo.subscriptions().await, HashSet::from([topic]));
}

#[tokio::test]
async fn unsubscribe_deduplicates_input_topics() {
    let (repo, mut request_rx) = test_repo();
    let topic = StreamTopic::FuturesInverseBtcUsdLastPrice;

    complete_subscribe(repo.clone(), &mut request_rx, vec![topic.clone()]).await;
    complete_unsubscribe(repo.clone(), &mut request_rx, vec![topic.clone(), topic]).await;

    assert!(repo.subscriptions().await.is_empty());
}

#[tokio::test]
async fn unsubscribe_skips_absent_topics() {
    let (repo, mut request_rx) = test_repo();

    repo.unsubscribe(vec![StreamTopic::FuturesInverseBtcUsdLastPrice])
        .await
        .expect("unsubscribe for absent topic must be idempotent");

    assert_no_request(&mut request_rx);
    assert!(repo.subscriptions().await.is_empty());
}

#[tokio::test]
async fn unsubscribe_errors_while_subscription_is_pending() {
    let (repo, mut request_rx) = test_repo();
    let topic = StreamTopic::FuturesInverseBtcUsdLastPrice;
    let subscribe_repo = repo.clone();
    let subscribe_topic = topic.clone();
    let subscribe_handle =
        tokio::spawn(async move { subscribe_repo.subscribe(vec![subscribe_topic]).await });
    let (_, response_tx) = receive_request(&mut request_rx).await;

    let error = repo
        .unsubscribe(vec![topic.clone()])
        .await
        .expect_err("unsubscribe must fail while subscribe is pending");

    assert!(matches!(
        error,
        StreamApiError::UnsubscribeWithSubscriptionPending(pending_topic)
            if pending_topic == topic
    ));
    assert_no_request(&mut request_rx);

    response_tx
        .send(Ok(StreamJsonRpcResult::Subscribe {
            subscribed: vec![topic.clone()],
            metadata: stream_metadata(),
        }))
        .expect("subscribe response must be received");
    subscribe_handle
        .await
        .expect("subscribe task must complete")
        .expect("subscribe must succeed");
}

#[tokio::test]
async fn subscribe_errors_while_unsubscription_is_pending() {
    let (repo, mut request_rx) = test_repo();
    let topic = StreamTopic::FuturesInverseBtcUsdLastPrice;

    complete_subscribe(repo.clone(), &mut request_rx, vec![topic.clone()]).await;

    let unsubscribe_repo = repo.clone();
    let unsubscribe_topic = topic.clone();
    let unsubscribe_handle =
        tokio::spawn(async move { unsubscribe_repo.unsubscribe(vec![unsubscribe_topic]).await });
    let (_, response_tx) = receive_request(&mut request_rx).await;

    let error = repo
        .subscribe(vec![topic.clone()])
        .await
        .expect_err("subscribe must fail while unsubscribe is pending");

    assert!(matches!(
        error,
        StreamApiError::SubscribeWithUnsubscriptionPending(pending_topic)
            if pending_topic == topic
    ));
    assert_no_request(&mut request_rx);

    response_tx
        .send(Ok(StreamJsonRpcResult::Unsubscribe {
            unsubscribed: vec![topic],
            metadata: stream_metadata(),
        }))
        .expect("unsubscribe response must be received");
    unsubscribe_handle
        .await
        .expect("unsubscribe task must complete")
        .expect("unsubscribe must succeed");
}
