use std::{
    collections::{HashMap, HashSet},
    env,
    num::NonZeroU64,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::Utc;
use dotenv::dotenv;
use tokio::{sync::broadcast, time};
use uuid::Uuid;

use crate::{
    rest::v3::{RestClient, RestClientConfig},
    shared::models::{
        client_id::ClientId,
        leverage::Leverage,
        price::PercentageCapped,
        quantity::order::OrderQuantity,
        trade::{TradeExecution, TradeSide},
    },
    stream::v1::{
        StreamClient, StreamClientConfig, StreamConnection,
        models::{
            topic::{StreamOhlcTimeframe, StreamTopic},
            update::StreamUpdate,
        },
    },
};

const LIVE_TEST_CLIENT_NAME: &str = "lnm-sdk-live-tests";
const LIVE_TEST_UPDATE_TIMEOUT: Duration = Duration::from_secs(120);
const LIVE_TEST_OHLC_TIMEFRAME: StreamOhlcTimeframe = StreamOhlcTimeframe::OneMinute;
const MIN_CROSS_MARGIN_SATS: u64 = 4_000;

struct LiveCredentials {
    key: String,
    secret: String,
    passphrase: String,
}

fn live_credentials_from_env() -> LiveCredentials {
    dotenv().ok();

    LiveCredentials {
        key: env::var("LNM_API_V3_KEY").expect("LNM_API_V3_KEY environment variable must be set"),
        secret: env::var("LNM_API_V3_SECRET")
            .expect("LNM_API_V3_SECRET environment variable must be set"),
        passphrase: env::var("LNM_API_V3_PASSPHRASE")
            .expect("LNM_API_V3_PASSPHRASE environment variable must be set"),
    }
}

fn init_rest_client(credentials: &LiveCredentials) -> Arc<RestClient> {
    RestClient::with_credentials(
        RestClientConfig::default(),
        &credentials.key,
        &credentials.secret,
        &credentials.passphrase,
    )
    .expect("must create authenticated REST client")
}

async fn connect_authenticated_stream(credentials: &LiveCredentials) -> StreamConnection {
    let client = StreamClient::new(StreamClientConfig::default());
    let stream = client.connect().await.expect("must connect to Stream API");

    assert!(stream.is_connected().await, "stream must be connected");
    assert!(
        stream.connection_status().await.is_connected(),
        "connection status must report connected"
    );

    let hello = stream
        .hello(LIVE_TEST_CLIENT_NAME, env!("CARGO_PKG_VERSION"))
        .await
        .expect("hello must succeed");
    assert!(!hello.version().is_empty(), "hello version must be set");

    stream.ping().await.expect("ping must return pong");

    let server_time = stream.time().await.expect("time must succeed");
    let clock_delta = Utc::now()
        .signed_duration_since(server_time)
        .num_seconds()
        .abs();
    assert!(
        clock_delta < 300,
        "server time must be close to local time; delta was {clock_delta}s"
    );

    let auth = stream
        .authenticate(
            &credentials.key,
            &credentials.secret,
            &credentials.passphrase,
        )
        .await
        .expect("authenticate must succeed");
    assert!(
        auth.authenticated(),
        "stream authentication must be accepted"
    );
    assert!(
        !auth.permissions().is_empty(),
        "authenticated session must include permissions"
    );

    let whoami = stream.whoami().await.expect("whoami must succeed");
    assert!(!whoami.api_key().is_empty(), "whoami api key must be set");
    assert!(!whoami.user_id().is_empty(), "whoami user id must be set");
    assert!(
        !whoami.permissions().is_empty(),
        "whoami permissions must be set"
    );

    stream
}

async fn disconnect_stream(stream: &StreamConnection) {
    if stream.is_connected().await {
        let _ = stream.unsubscribe_all().await;
        let _ = stream.disconnect().await;
    }
}

async fn cleanup_trading_state(rest: &RestClient) {
    let _ = rest.futures_isolated.cancel_all_trades().await;

    if let Ok(running_trades) = rest.futures_isolated.get_running_trades().await {
        for trade in running_trades {
            let _ = rest.futures_isolated.close_trade(trade.id()).await;
        }
    }

    let _ = rest.futures_cross.cancel_all_orders().await;

    if let Ok(position) = rest.futures_cross.get_position().await
        && position.quantity() != 0
    {
        let _ = rest.futures_cross.close_position().await;
    }
}

async fn ensure_cross_margin(rest: &RestClient, min_margin: u64) -> u64 {
    let position = rest
        .futures_cross
        .get_position()
        .await
        .expect("must get cross position");

    if position.margin() >= min_margin {
        return 0;
    }

    let deposit_amount = min_margin - position.margin();
    rest.futures_cross
        .deposit(NonZeroU64::try_from(deposit_amount).expect("deposit amount must be non-zero"))
        .await
        .expect("must deposit enough margin for cross live tests");

    deposit_amount
}

async fn withdraw_test_cross_margin(rest: &RestClient, deposited_margin: u64) {
    if deposited_margin == 0 {
        return;
    }

    let Ok(position) = rest.futures_cross.get_position().await else {
        return;
    };

    if position.quantity() != 0 || position.margin() == 0 {
        return;
    }

    let withdrawal_amount = deposited_margin.min(position.margin());
    if let Ok(withdrawal_amount) = NonZeroU64::try_from(withdrawal_amount) {
        let _ = rest.futures_cross.withdraw(withdrawal_amount).await;
    }
}

async fn recv_update(
    receiver: &mut broadcast::Receiver<StreamUpdate>,
    timeout: Duration,
    context: &str,
) -> StreamUpdate {
    match time::timeout(timeout, receiver.recv()).await {
        Ok(Ok(update)) => update,
        Ok(Err(broadcast::error::RecvError::Lagged(skipped))) => {
            panic!("receiver lagged by {skipped} messages while waiting for {context}")
        }
        Ok(Err(broadcast::error::RecvError::Closed)) => {
            panic!("receiver closed while waiting for {context}")
        }
        Err(_) => panic!("timed out after {timeout:?} while waiting for {context}"),
    }
}

async fn recv_matching_update<F>(
    receiver: &mut broadcast::Receiver<StreamUpdate>,
    timeout: Duration,
    context: &str,
    mut predicate: F,
) -> StreamUpdate
where
    F: FnMut(&StreamUpdate) -> bool,
{
    let deadline = time::Instant::now() + timeout;

    loop {
        let now = time::Instant::now();
        assert!(
            now < deadline,
            "timed out after {timeout:?} while waiting for {context}"
        );

        let update = recv_update(receiver, deadline - now, context).await;
        println!("Received Stream update while waiting for {context}: {update:?}");

        if predicate(&update) {
            return update;
        }
    }
}

async fn collect_expected_topics(
    receiver: &mut broadcast::Receiver<StreamUpdate>,
    expected_topics: HashSet<StreamTopic>,
    timeout: Duration,
) -> HashMap<StreamTopic, StreamUpdate> {
    let mut updates = HashMap::new();
    let deadline = time::Instant::now() + timeout;

    while updates.len() < expected_topics.len() {
        let now = time::Instant::now();
        assert!(
            now < deadline,
            "timed out after {timeout:?} while waiting for topics: {:?}",
            expected_topics
                .difference(&updates.keys().cloned().collect())
                .collect::<Vec<_>>()
        );

        let update = recv_update(receiver, deadline - now, "public topic updates").await;
        if let Some(topic) = update.topic()
            && expected_topics.contains(&topic)
        {
            assert_public_update(&update);
            updates.entry(topic).or_insert(update);
        }
    }

    updates
}

fn assert_public_update(update: &StreamUpdate) {
    match update {
        StreamUpdate::FuturesInverseBtcUsdTicker(ticker) => {
            assert!(ticker.time().timestamp_millis() > 0);
            assert!(ticker.last_price().is_some() || ticker.index().is_some());
            assert!(ticker.funding().time().timestamp_millis() > 0);
        }
        StreamUpdate::FuturesInverseBtcUsdLastPrice(last_price) => {
            assert!(last_price.time().timestamp_millis() > 0);
            assert!(last_price.last_price().as_f64() > 0.0);
        }
        StreamUpdate::FuturesInverseBtcUsdIndex(index) => {
            assert!(index.time().timestamp_millis() > 0);
            assert!(index.index().as_f64() > 0.0);
        }
        StreamUpdate::FuturesInverseBtcUsdBuckets(buckets) => {
            assert!(buckets.time().timestamp_millis() > 0);
            assert!(!buckets.buckets().is_empty());
            for bucket in buckets.buckets() {
                assert!(bucket.min_size() <= bucket.max_size());
                assert!(bucket.ask_price().as_f64() > 0.0);
                assert!(bucket.bid_price().as_f64() > 0.0);
            }
        }
        StreamUpdate::FuturesInverseBtcUsdOhlc { timeframe, candle } => {
            assert_eq!(*timeframe, LIVE_TEST_OHLC_TIMEFRAME);
            assert!(candle.time().timestamp_millis() > 0);
            assert!(candle.open().as_f64() > 0.0);
            assert!(candle.high().as_f64() > 0.0);
            assert!(candle.low().as_f64() > 0.0);
            assert!(candle.close().as_f64() > 0.0);
        }
        unexpected => panic!("unexpected public update: {unexpected:?}"),
    }
}

fn live_client_id(prefix: &str) -> ClientId {
    let value = format!("{prefix}-{}", Uuid::new_v4().simple());
    ClientId::try_from(value).expect("live test client id must be valid")
}

fn matches_id_or_client_id(
    id: Option<Uuid>,
    client_id: Option<&ClientId>,
    expected_id: Uuid,
    expected_client_id: &ClientId,
) -> bool {
    id == Some(expected_id) || client_id == Some(expected_client_id)
}

#[tokio::test]
#[ignore]
async fn test_live_stream_api_methods_and_public_updates() {
    let credentials = live_credentials_from_env();
    let stream = connect_authenticated_stream(&credentials).await;
    let mut receiver = stream
        .receiver()
        .await
        .expect("must create stream receiver");

    let public_topics = HashSet::from([
        StreamTopic::FuturesInverseBtcUsdTicker,
        StreamTopic::FuturesInverseBtcUsdLastPrice,
        StreamTopic::FuturesInverseBtcUsdIndex,
        StreamTopic::FuturesInverseBtcUsdBuckets,
        StreamTopic::FuturesInverseBtcUsdOhlc(LIVE_TEST_OHLC_TIMEFRAME),
    ]);

    macro_rules! time_test {
        ($test_name: expr, $test_block: expr) => {{
            println!("\nStarting test: {}", $test_name);
            let start = Instant::now();
            let result = $test_block;
            let elapsed = start.elapsed();
            println!("Test '{}' took: {:?}", $test_name, elapsed);
            result
        }};
    }

    time_test!(
        "subscribe public topics",
        stream
            .subscribe(public_topics.iter().cloned().collect())
            .await
            .expect("must subscribe to public topics")
    );

    let subscriptions = stream.subscriptions().await;
    for topic in &public_topics {
        assert!(
            subscriptions.contains(topic),
            "subscription state must contain {topic}"
        );
    }

    time_test!(
        "receive public topic updates",
        collect_expected_topics(
            &mut receiver,
            public_topics.clone(),
            LIVE_TEST_UPDATE_TIMEOUT
        )
        .await
    );

    time_test!(
        "unsubscribe one public topic",
        stream
            .unsubscribe(vec![StreamTopic::FuturesInverseBtcUsdLastPrice])
            .await
            .expect("must unsubscribe one public topic")
    );

    assert!(
        !stream
            .subscriptions()
            .await
            .contains(&StreamTopic::FuturesInverseBtcUsdLastPrice),
        "unsubscribed topic must be absent from subscription state"
    );

    let unsubscribed = time_test!(
        "unsubscribe all public topics",
        stream
            .unsubscribe_all()
            .await
            .expect("must unsubscribe remaining public topics")
    );
    assert!(
        !unsubscribed.is_empty(),
        "unsubscribe_all must return topics"
    );
    assert!(stream.subscriptions().await.is_empty());

    disconnect_stream(&stream).await;
}

#[tokio::test]
#[ignore]
async fn test_live_stream_private_updates_triggered_by_rest() {
    let credentials = live_credentials_from_env();
    let rest = init_rest_client(&credentials);

    macro_rules! time_test {
        ($test_name: expr, $test_block: expr) => {{
            println!("\nStarting test: {}", $test_name);
            let start = Instant::now();
            let result = $test_block;
            let elapsed = start.elapsed();
            println!("Test '{}' took: {:?}", $test_name, elapsed);
            result
        }};
    }

    time_test!("cleanup trading state", cleanup_trading_state(&rest).await);
    let deposited_margin = time_test!(
        "ensure cross margin",
        ensure_cross_margin(&rest, MIN_CROSS_MARGIN_SATS).await
    );

    let stream = connect_authenticated_stream(&credentials).await;
    let mut receiver = stream
        .receiver()
        .await
        .expect("must create stream receiver");
    let private_topics = vec![
        StreamTopic::FuturesInverseBtcUsdIsolatedTrades,
        StreamTopic::FuturesInverseBtcUsdCrossOrders,
        StreamTopic::FuturesInverseBtcUsdCrossPosition,
    ];

    time_test!(
        "subscribe private topics",
        stream
            .subscribe(private_topics.clone())
            .await
            .expect("must subscribe to private topics")
    );

    let isolated_client_id = live_client_id("lnm-live-iso");
    let isolated_trade = time_test!(
        "create 1 USD isolated market trade",
        rest.futures_isolated
            .new_trade(
                TradeSide::Buy,
                OrderQuantity::try_from(1).unwrap().into(),
                Leverage::try_from(2).unwrap(),
                TradeExecution::Market,
                None,
                None,
                Some(isolated_client_id.clone()),
            )
            .await
            .expect("must create isolated market trade")
    );
    let isolated_trade_id = isolated_trade.id();

    let isolated_open_update = time_test!(
        "receive isolated trade stream update",
        recv_matching_update(
            &mut receiver,
            LIVE_TEST_UPDATE_TIMEOUT,
            "isolated trade update",
            |update| matches!(
                update,
                StreamUpdate::FuturesInverseBtcUsdIsolatedTrades(event)
                    if matches_id_or_client_id(
                        event.trade().id(),
                        event.trade().client_id(),
                        isolated_trade_id,
                        &isolated_client_id,
                    )
            ),
        )
        .await
    );
    let StreamUpdate::FuturesInverseBtcUsdIsolatedTrades(event) = isolated_open_update else {
        panic!("expected isolated trade update");
    };
    assert_eq!(event.pair(), "btc_usd");
    assert!(!event.event().is_empty());

    time_test!(
        "close isolated market trade",
        rest.futures_isolated
            .close_trade(isolated_trade.id())
            .await
            .expect("must close isolated trade")
    );

    time_test!(
        "receive isolated close stream update",
        recv_matching_update(
            &mut receiver,
            LIVE_TEST_UPDATE_TIMEOUT,
            "isolated close update",
            |update| matches!(
                update,
                StreamUpdate::FuturesInverseBtcUsdIsolatedTrades(event)
                    if event.trade().id() == Some(isolated_trade_id)
            ),
        )
        .await
    );

    let ticker = rest
        .futures_data
        .get_ticker()
        .await
        .expect("must get ticker for limit order price");
    let limit_price = ticker
        .last_price()
        .apply_discount(PercentageCapped::try_from(30).unwrap())
        .unwrap();
    let cross_limit_client_id = live_client_id("lnm-live-xlim");
    let cross_limit_order = time_test!(
        "place 1 USD cross limit order",
        rest.futures_cross
            .place_order(
                TradeSide::Buy,
                OrderQuantity::try_from(1).unwrap(),
                TradeExecution::Limit(limit_price),
                Some(cross_limit_client_id.clone()),
            )
            .await
            .expect("must place cross limit order")
    );
    let cross_limit_order_id = cross_limit_order.id();

    let cross_order_update = time_test!(
        "receive cross order stream update",
        recv_matching_update(
            &mut receiver,
            LIVE_TEST_UPDATE_TIMEOUT,
            "cross order update",
            |update| matches!(
                update,
                StreamUpdate::FuturesInverseBtcUsdCrossOrders(event)
                    if matches_id_or_client_id(
                        event.order().id(),
                        event.order().client_id(),
                        cross_limit_order_id,
                        &cross_limit_client_id,
                    )
            ),
        )
        .await
    );
    let StreamUpdate::FuturesInverseBtcUsdCrossOrders(event) = cross_order_update else {
        panic!("expected cross order update");
    };
    assert_eq!(event.pair(), "btc_usd");
    assert!(!event.event().is_empty());

    time_test!(
        "cancel cross limit order",
        rest.futures_cross
            .cancel_order(cross_limit_order.id())
            .await
            .expect("must cancel cross limit order")
    );

    time_test!(
        "receive cross order cancel stream update",
        recv_matching_update(
            &mut receiver,
            LIVE_TEST_UPDATE_TIMEOUT,
            "cross order cancel update",
            |update| matches!(
                update,
                StreamUpdate::FuturesInverseBtcUsdCrossOrders(event)
                    if event.order().id() == Some(cross_limit_order_id)
            ),
        )
        .await
    );

    let cross_market_order = time_test!(
        "place 1 USD cross market order",
        rest.futures_cross
            .place_order(
                TradeSide::Buy,
                OrderQuantity::try_from(1).unwrap(),
                TradeExecution::Market,
                None,
            )
            .await
            .expect("must place cross market order")
    );
    assert!(cross_market_order.filled());

    let cross_position_update = time_test!(
        "receive cross position stream update",
        recv_matching_update(
            &mut receiver,
            LIVE_TEST_UPDATE_TIMEOUT,
            "cross position update",
            |update| matches!(
                update,
                StreamUpdate::FuturesInverseBtcUsdCrossPosition(event)
                    if event.position().quantity().is_some()
            ),
        )
        .await
    );
    let StreamUpdate::FuturesInverseBtcUsdCrossPosition(event) = cross_position_update else {
        panic!("expected cross position update");
    };
    assert_eq!(event.pair(), "btc_usd");
    assert!(!event.event().is_empty());

    time_test!(
        "close cross position",
        rest.futures_cross
            .close_position()
            .await
            .expect("must close cross position")
    );

    time_test!(
        "receive cross position close stream update",
        recv_matching_update(
            &mut receiver,
            LIVE_TEST_UPDATE_TIMEOUT,
            "cross position close update",
            |update| matches!(
                update,
                StreamUpdate::FuturesInverseBtcUsdCrossPosition(event)
                    if event.pair() == "btc_usd" && !event.event().is_empty()
            ),
        )
        .await
    );

    time_test!("cleanup trading state", cleanup_trading_state(&rest).await);
    time_test!(
        "withdraw test cross margin",
        withdraw_test_cross_margin(&rest, deposited_margin).await
    );

    stream
        .unsubscribe_all()
        .await
        .expect("must unsubscribe private topics");
    assert!(stream.subscriptions().await.is_empty());
    disconnect_stream(&stream).await;
}
