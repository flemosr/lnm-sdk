use std::{collections::HashSet, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, DeserializeOwned},
};
use serde_json::{Value, json};

use super::error::{ConnectionResult, StreamConnectionError};
use super::state::StreamConnectionStatus;

/// Rate-limit metadata returned by the Stream API on JSON-RPC responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct StreamRateLimit {
    remaining: u64,
    limit: u64,
}

impl StreamRateLimit {
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }
}

/// Server-side timing and rate-limit metadata returned with JSON-RPC responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamResponseMetadata {
    us_in: Option<u64>,
    us_out: Option<u64>,
    us_diff: Option<u64>,
    rate_limit: Option<StreamRateLimit>,
}

impl StreamResponseMetadata {
    pub(super) fn new(
        us_in: Option<u64>,
        us_out: Option<u64>,
        us_diff: Option<u64>,
        rate_limit: Option<StreamRateLimit>,
    ) -> Self {
        Self {
            us_in,
            us_out,
            us_diff,
            rate_limit,
        }
    }

    pub fn us_in(&self) -> Option<u64> {
        self.us_in
    }

    pub fn us_out(&self) -> Option<u64> {
        self.us_out
    }

    pub fn us_diff(&self) -> Option<u64> {
        self.us_diff
    }

    pub fn rate_limit(&self) -> Option<StreamRateLimit> {
        self.rate_limit
    }
}

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

/// Stream v1 OHLC timeframe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamOhlcTimeframe {
    OneMinute,
    ThreeMinutes,
    FiveMinutes,
    TenMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    FortyFiveMinutes,
    OneHour,
    TwoHours,
    ThreeHours,
    FourHours,
    OneDay,
    OneWeek,
    OneMonth,
    ThreeMonths,
}

impl StreamOhlcTimeframe {
    fn as_str(&self) -> &'static str {
        match self {
            StreamOhlcTimeframe::OneMinute => "1m",
            StreamOhlcTimeframe::ThreeMinutes => "3m",
            StreamOhlcTimeframe::FiveMinutes => "5m",
            StreamOhlcTimeframe::TenMinutes => "10m",
            StreamOhlcTimeframe::FifteenMinutes => "15m",
            StreamOhlcTimeframe::ThirtyMinutes => "30m",
            StreamOhlcTimeframe::FortyFiveMinutes => "45m",
            StreamOhlcTimeframe::OneHour => "1h",
            StreamOhlcTimeframe::TwoHours => "2h",
            StreamOhlcTimeframe::ThreeHours => "3h",
            StreamOhlcTimeframe::FourHours => "4h",
            StreamOhlcTimeframe::OneDay => "1d",
            StreamOhlcTimeframe::OneWeek => "1w",
            StreamOhlcTimeframe::OneMonth => "1month",
            StreamOhlcTimeframe::ThreeMonths => "3months",
        }
    }
}

impl fmt::Display for StreamOhlcTimeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for StreamOhlcTimeframe {
    type Err = StreamConnectionError;

    fn from_str(value: &str) -> ConnectionResult<Self> {
        match value {
            "1m" => Ok(StreamOhlcTimeframe::OneMinute),
            "3m" => Ok(StreamOhlcTimeframe::ThreeMinutes),
            "5m" => Ok(StreamOhlcTimeframe::FiveMinutes),
            "10m" => Ok(StreamOhlcTimeframe::TenMinutes),
            "15m" => Ok(StreamOhlcTimeframe::FifteenMinutes),
            "30m" => Ok(StreamOhlcTimeframe::ThirtyMinutes),
            "45m" => Ok(StreamOhlcTimeframe::FortyFiveMinutes),
            "1h" => Ok(StreamOhlcTimeframe::OneHour),
            "2h" => Ok(StreamOhlcTimeframe::TwoHours),
            "3h" => Ok(StreamOhlcTimeframe::ThreeHours),
            "4h" => Ok(StreamOhlcTimeframe::FourHours),
            "1d" => Ok(StreamOhlcTimeframe::OneDay),
            "1w" => Ok(StreamOhlcTimeframe::OneWeek),
            "1month" => Ok(StreamOhlcTimeframe::OneMonth),
            "3months" => Ok(StreamOhlcTimeframe::ThreeMonths),
            _ => Err(StreamConnectionError::UnknownOhlcTimeframe(
                value.to_string(),
            )),
        }
    }
}

impl Serialize for StreamOhlcTimeframe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StreamOhlcTimeframe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(de::Error::custom)
    }
}

/// Subscription topics supported by the Stream v1 API.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamTopic {
    Announcements,
    FuturesInverseBtcUsdTicker,
    FuturesInverseBtcUsdLastPrice,
    FuturesInverseBtcUsdIndex,
    FuturesInverseBtcUsdBuckets,
    FuturesInverseBtcUsdFunding,
    FuturesInverseBtcUsdIsolatedTrades,
    FuturesInverseBtcUsdCrossOrders,
    FuturesInverseBtcUsdCrossPosition,
    FuturesInverseBtcUsdOhlc(StreamOhlcTimeframe),
    WalletDeposit,
    WalletWithdrawal,
}

impl StreamTopic {
    fn as_string(&self) -> String {
        match self {
            StreamTopic::Announcements => "announcements".to_string(),
            StreamTopic::FuturesInverseBtcUsdTicker => "futures/inverse/btc_usd/ticker".to_string(),
            StreamTopic::FuturesInverseBtcUsdLastPrice => {
                "futures/inverse/btc_usd/lastPrice".to_string()
            }
            StreamTopic::FuturesInverseBtcUsdIndex => "futures/inverse/btc_usd/index".to_string(),
            StreamTopic::FuturesInverseBtcUsdBuckets => {
                "futures/inverse/btc_usd/buckets".to_string()
            }
            StreamTopic::FuturesInverseBtcUsdFunding => {
                "futures/inverse/btc_usd/funding".to_string()
            }
            StreamTopic::FuturesInverseBtcUsdIsolatedTrades => {
                "futures/inverse/btc_usd/isolated/trades".to_string()
            }
            StreamTopic::FuturesInverseBtcUsdCrossOrders => {
                "futures/inverse/btc_usd/cross/orders".to_string()
            }
            StreamTopic::FuturesInverseBtcUsdCrossPosition => {
                "futures/inverse/btc_usd/cross/position".to_string()
            }
            StreamTopic::FuturesInverseBtcUsdOhlc(timeframe) => {
                format!("futures/inverse/btc_usd/ohlc/{timeframe}")
            }
            StreamTopic::WalletDeposit => "wallet/deposit".to_string(),
            StreamTopic::WalletWithdrawal => "wallet/withdrawal".to_string(),
        }
    }
}

impl fmt::Display for StreamTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl FromStr for StreamTopic {
    type Err = StreamConnectionError;

    fn from_str(value: &str) -> ConnectionResult<Self> {
        match value {
            "announcements" => Ok(StreamTopic::Announcements),
            "futures/inverse/btc_usd/ticker" => Ok(StreamTopic::FuturesInverseBtcUsdTicker),
            "futures/inverse/btc_usd/lastPrice" => Ok(StreamTopic::FuturesInverseBtcUsdLastPrice),
            "futures/inverse/btc_usd/index" => Ok(StreamTopic::FuturesInverseBtcUsdIndex),
            "futures/inverse/btc_usd/buckets" => Ok(StreamTopic::FuturesInverseBtcUsdBuckets),
            "futures/inverse/btc_usd/funding" => Ok(StreamTopic::FuturesInverseBtcUsdFunding),
            "futures/inverse/btc_usd/isolated/trades" => {
                Ok(StreamTopic::FuturesInverseBtcUsdIsolatedTrades)
            }
            "futures/inverse/btc_usd/cross/orders" => {
                Ok(StreamTopic::FuturesInverseBtcUsdCrossOrders)
            }
            "futures/inverse/btc_usd/cross/position" => {
                Ok(StreamTopic::FuturesInverseBtcUsdCrossPosition)
            }
            "wallet/deposit" => Ok(StreamTopic::WalletDeposit),
            "wallet/withdrawal" => Ok(StreamTopic::WalletWithdrawal),
            value => {
                const OHLC_PREFIX: &str = "futures/inverse/btc_usd/ohlc/";
                if let Some(timeframe) = value.strip_prefix(OHLC_PREFIX) {
                    return Ok(StreamTopic::FuturesInverseBtcUsdOhlc(
                        StreamOhlcTimeframe::from_str(timeframe)?,
                    ));
                }

                Err(StreamConnectionError::UnknownTopic(value.to_string()))
            }
        }
    }
}

impl Serialize for StreamTopic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

impl<'de> Deserialize<'de> for StreamTopic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(de::Error::custom)
    }
}

fn deserialize_timestamp_millis<'de, D>(
    deserializer: D,
) -> std::result::Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = i64::deserialize(deserializer)?;
    DateTime::<Utc>::from_timestamp_millis(millis)
        .ok_or_else(|| de::Error::custom(format!("invalid timestamp milliseconds: {millis}")))
}

fn deserialize_optional_timestamp_millis<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = Option::<i64>::deserialize(deserializer)?;
    millis
        .map(|millis| {
            DateTime::<Utc>::from_timestamp_millis(millis).ok_or_else(|| {
                de::Error::custom(format!("invalid timestamp milliseconds: {millis}"))
            })
        })
        .transpose()
}

fn decode_subscription_data<T>(data: Value) -> ConnectionResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(data).map_err(StreamConnectionError::DecodeJson)
}

/// Platform announcement notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamAnnouncement {
    id: String,
    title: String,
    message: String,
    link: String,
}

impl StreamAnnouncement {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn link(&self) -> &str {
        &self.link
    }
}

/// Funding rate and settlement timestamp payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamFundingRate {
    rate: f64,
    #[serde(deserialize_with = "deserialize_timestamp_millis")]
    time: DateTime<Utc>,
}

impl StreamFundingRate {
    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }
}

/// Inverse futures aggregated ticker notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamTicker {
    #[serde(deserialize_with = "deserialize_timestamp_millis")]
    time: DateTime<Utc>,
    last_price: Option<f64>,
    index: Option<f64>,
    funding: StreamFundingRate,
}

impl StreamTicker {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn last_price(&self) -> Option<f64> {
        self.last_price
    }

    pub fn index(&self) -> Option<f64> {
        self.index
    }

    pub fn funding(&self) -> &StreamFundingRate {
        &self.funding
    }
}

/// Inverse futures last trade price notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamLastPrice {
    #[serde(deserialize_with = "deserialize_timestamp_millis")]
    time: DateTime<Utc>,
    last_price: f64,
}

impl StreamLastPrice {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn last_price(&self) -> f64 {
        self.last_price
    }
}

/// Inverse futures index price notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamIndex {
    #[serde(deserialize_with = "deserialize_timestamp_millis")]
    time: DateTime<Utc>,
    index: f64,
}

impl StreamIndex {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn index(&self) -> f64 {
        self.index
    }
}

/// One inverse futures volume ladder bucket.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBucket {
    min_size: f64,
    max_size: f64,
    ask_price: f64,
    bid_price: f64,
}

impl StreamBucket {
    pub fn min_size(&self) -> f64 {
        self.min_size
    }

    pub fn max_size(&self) -> f64 {
        self.max_size
    }

    pub fn ask_price(&self) -> f64 {
        self.ask_price
    }

    pub fn bid_price(&self) -> f64 {
        self.bid_price
    }
}

/// Inverse futures volume ladder bucket notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBuckets {
    #[serde(deserialize_with = "deserialize_timestamp_millis")]
    time: DateTime<Utc>,
    buckets: Vec<StreamBucket>,
}

impl StreamBuckets {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn buckets(&self) -> &[StreamBucket] {
        &self.buckets
    }
}

/// Inverse futures funding notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamFunding {
    pair: String,
    current: StreamFundingRate,
}

impl StreamFunding {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn current(&self) -> &StreamFundingRate {
        &self.current
    }
}

/// Inverse futures OHLC candle notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamOhlc {
    #[serde(deserialize_with = "deserialize_timestamp_millis")]
    time: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl StreamOhlc {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn open(&self) -> f64 {
        self.open
    }

    pub fn high(&self) -> f64 {
        self.high
    }

    pub fn low(&self) -> f64 {
        self.low
    }

    pub fn close(&self) -> f64 {
        self.close
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }
}

/// Inverse futures isolated-margin trade event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamIsolatedTradeEvent {
    pair: String,
    event: String,
    trade: StreamIsolatedTrade,
}

impl StreamIsolatedTradeEvent {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn trade(&self) -> &StreamIsolatedTrade {
        &self.trade
    }
}

/// Inverse futures isolated-margin trade payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamIsolatedTrade {
    id: Option<String>,
    side: Option<String>,
    #[serde(rename = "type")]
    trade_type: Option<String>,
    quantity: Option<f64>,
    margin: Option<f64>,
    leverage: Option<f64>,
    price: Option<f64>,
    opening_fee: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_timestamp_millis")]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    client_id: Option<String>,
}

impl StreamIsolatedTrade {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn side(&self) -> Option<&str> {
        self.side.as_deref()
    }

    pub fn trade_type(&self) -> Option<&str> {
        self.trade_type.as_deref()
    }

    pub fn quantity(&self) -> Option<f64> {
        self.quantity
    }

    pub fn margin(&self) -> Option<f64> {
        self.margin
    }

    pub fn leverage(&self) -> Option<f64> {
        self.leverage
    }

    pub fn price(&self) -> Option<f64> {
        self.price
    }

    pub fn opening_fee(&self) -> Option<f64> {
        self.opening_fee
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }
}

/// Inverse futures cross-margin order event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossOrderEvent {
    pair: String,
    event: String,
    order: StreamCrossOrder,
}

impl StreamCrossOrderEvent {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn order(&self) -> &StreamCrossOrder {
        &self.order
    }
}

/// Inverse futures cross-margin order payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossOrder {
    id: Option<String>,
    side: Option<String>,
    #[serde(rename = "type")]
    order_type: Option<String>,
    quantity: Option<f64>,
    price: Option<f64>,
    trading_fee: Option<f64>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_timestamp_millis")]
    created_at: Option<DateTime<Utc>>,
}

impl StreamCrossOrder {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn side(&self) -> Option<&str> {
        self.side.as_deref()
    }

    pub fn order_type(&self) -> Option<&str> {
        self.order_type.as_deref()
    }

    pub fn quantity(&self) -> Option<f64> {
        self.quantity
    }

    pub fn price(&self) -> Option<f64> {
        self.price
    }

    pub fn trading_fee(&self) -> Option<f64> {
        self.trading_fee
    }

    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }
}

/// Inverse futures cross-margin position event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossPositionEvent {
    pair: String,
    event: String,
    position: StreamCrossPosition,
}

impl StreamCrossPositionEvent {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn position(&self) -> &StreamCrossPosition {
        &self.position
    }
}

/// Inverse futures cross-margin position payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossPosition {
    quantity: Option<f64>,
    leverage: Option<f64>,
    margin: Option<f64>,
    entry_price: Option<f64>,
    liquidation: Option<f64>,
    total_pl: Option<f64>,
    funding_fees: Option<f64>,
    trading_fees: Option<f64>,
    initial_margin: Option<f64>,
    maintenance_margin: Option<f64>,
    running_margin: Option<f64>,
    delta_pl: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_timestamp_millis")]
    updated_at: Option<DateTime<Utc>>,
}

impl StreamCrossPosition {
    pub fn quantity(&self) -> Option<f64> {
        self.quantity
    }

    pub fn leverage(&self) -> Option<f64> {
        self.leverage
    }

    pub fn margin(&self) -> Option<f64> {
        self.margin
    }

    pub fn entry_price(&self) -> Option<f64> {
        self.entry_price
    }

    pub fn liquidation(&self) -> Option<f64> {
        self.liquidation
    }

    pub fn total_pl(&self) -> Option<f64> {
        self.total_pl
    }

    pub fn funding_fees(&self) -> Option<f64> {
        self.funding_fees
    }

    pub fn trading_fees(&self) -> Option<f64> {
        self.trading_fees
    }

    pub fn initial_margin(&self) -> Option<f64> {
        self.initial_margin
    }

    pub fn maintenance_margin(&self) -> Option<f64> {
        self.maintenance_margin
    }

    pub fn running_margin(&self) -> Option<f64> {
        self.running_margin
    }

    pub fn delta_pl(&self) -> Option<f64> {
        self.delta_pl
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

/// Wallet deposit event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamWalletDeposit {
    currency: String,
    network: String,
    id: String,
    amount: f64,
    balance: f64,
    status: String,
    #[serde(default)]
    tx_id: Option<String>,
}

impl StreamWalletDeposit {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn network(&self) -> &str {
        &self.network
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn tx_id(&self) -> Option<&str> {
        self.tx_id.as_deref()
    }
}

/// Wallet withdrawal event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamWalletWithdrawal {
    currency: String,
    network: String,
    id: String,
    amount: f64,
    fee: f64,
    balance: f64,
    status: String,
    #[serde(default)]
    tx_id: Option<String>,
}

impl StreamWalletWithdrawal {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn network(&self) -> &str {
        &self.network
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn fee(&self) -> f64 {
        self.fee
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn tx_id(&self) -> Option<&str> {
        self.tx_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub(in crate::stream::v1) enum JsonRpcId {
    String(String),
    Unsigned(u64),
    Signed(i64),
}

impl fmt::Display for JsonRpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonRpcId::String(id) => write!(f, "{id}"),
            JsonRpcId::Unsigned(id) => write!(f, "{id}"),
            JsonRpcId::Signed(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::stream::v1) enum StreamJsonRpcReqMethod {
    Hello,
    Ping,
    Time,
    Authenticate,
    Whoami,
    Subscribe,
    Unsubscribe,
    UnsubscribeAll,
}

impl StreamJsonRpcReqMethod {
    fn as_str(&self) -> &'static str {
        match self {
            StreamJsonRpcReqMethod::Hello => "hello",
            StreamJsonRpcReqMethod::Ping => "ping",
            StreamJsonRpcReqMethod::Time => "time",
            StreamJsonRpcReqMethod::Authenticate => "authenticate",
            StreamJsonRpcReqMethod::Whoami => "whoami",
            StreamJsonRpcReqMethod::Subscribe => "subscribe",
            StreamJsonRpcReqMethod::Unsubscribe => "unsubscribe",
            StreamJsonRpcReqMethod::UnsubscribeAll => "unsubscribeAll",
        }
    }
}

impl fmt::Display for StreamJsonRpcReqMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct JsonRpcRequestWire<'a> {
    jsonrpc: &'static str,
    method: &'static str,
    id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<&'a Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::stream::v1) struct StreamJsonRpcRequest {
    method: StreamJsonRpcReqMethod,
    id: String,
    params: Option<Value>,
}

impl StreamJsonRpcRequest {
    pub(in crate::stream::v1) fn new(
        method: StreamJsonRpcReqMethod,
        params: Option<Value>,
    ) -> Self {
        let mut random_bytes = [0u8; 16];
        rand::rng().fill(&mut random_bytes);
        let id = hex::encode(random_bytes);

        Self { method, id, params }
    }

    #[cfg(test)]
    fn new_with_id(
        method: StreamJsonRpcReqMethod,
        id: impl ToString,
        params: Option<Value>,
    ) -> Self {
        Self {
            method,
            id: id.to_string(),
            params,
        }
    }

    pub(in crate::stream::v1) fn id(&self) -> &String {
        &self.id
    }

    pub(in crate::stream::v1) fn method(&self) -> &StreamJsonRpcReqMethod {
        &self.method
    }

    pub(in crate::stream::v1) fn try_to_bytes(&self) -> ConnectionResult<Vec<u8>> {
        let request = JsonRpcRequestWire {
            jsonrpc: "2.0",
            method: self.method.as_str(),
            id: &self.id,
            params: self.params.as_ref(),
        };
        let request_json =
            serde_json::to_string(&request).map_err(StreamConnectionError::EncodeJson)?;
        Ok(request_json.into_bytes())
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(in crate::stream::v1) struct JsonRpcEnvelope {
    jsonrpc: String,
    id: Option<JsonRpcId>,
    method: Option<String>,
    result: Option<Value>,
    error: Option<StreamJsonRpcError>,
    params: Option<Value>,
    #[serde(rename = "usIn")]
    us_in: Option<u64>,
    #[serde(rename = "usOut")]
    us_out: Option<u64>,
    #[serde(rename = "usDiff")]
    us_diff: Option<u64>,
    rate_limit: Option<StreamRateLimit>,
}

impl JsonRpcEnvelope {
    fn metadata(&self) -> StreamResponseMetadata {
        StreamResponseMetadata::new(self.us_in, self.us_out, self.us_diff, self.rate_limit)
    }
}

/// Result of a Stream JSON-RPC request.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::stream::v1) enum StreamJsonRpcResult {
    Hello(HelloResult),
    Pong(StreamResponseMetadata),
    Time(TimeResult),
    Authenticate(AuthenticateResult),
    Whoami(WhoamiResult),
    Subscribe {
        subscribed: Vec<StreamTopic>,
        metadata: StreamResponseMetadata,
    },
    Unsubscribe {
        unsubscribed: Vec<StreamTopic>,
        metadata: StreamResponseMetadata,
    },
    UnsubscribeAll {
        unsubscribed: Vec<StreamTopic>,
        metadata: StreamResponseMetadata,
    },
}

/// Result returned by the `hello` method.
#[derive(Debug, Clone, PartialEq)]
pub struct HelloResult {
    version: String,
    metadata: StreamResponseMetadata,
}

impl HelloResult {
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn metadata(&self) -> &StreamResponseMetadata {
        &self.metadata
    }
}

/// Result returned by the `time` method.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeResult {
    time: DateTime<Utc>,
    metadata: StreamResponseMetadata,
}

impl TimeResult {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn metadata(&self) -> &StreamResponseMetadata {
        &self.metadata
    }
}

/// Result returned by the `authenticate` method.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticateResult {
    authenticated: bool,
    permissions: Vec<String>,
    metadata: StreamResponseMetadata,
}

impl AuthenticateResult {
    pub fn authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    pub fn metadata(&self) -> &StreamResponseMetadata {
        &self.metadata
    }
}

/// Result returned by the `whoami` method.
#[derive(Debug, Clone, PartialEq)]
pub struct WhoamiResult {
    api_key: String,
    user_id: String,
    permissions: Vec<String>,
    metadata: StreamResponseMetadata,
}

impl WhoamiResult {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn permissions(&self) -> &[String] {
        &self.permissions
    }

    pub fn metadata(&self) -> &StreamResponseMetadata {
        &self.metadata
    }
}

/// Message emitted by the Stream event loop after decoding an incoming JSON-RPC frame.
#[derive(Clone, Debug)]
pub(in crate::stream::v1) enum StreamJsonRpcMessage {
    Response {
        id: String,
        result: Result<Value, StreamJsonRpcError>,
        metadata: StreamResponseMetadata,
    },
    Subscription(StreamUpdate),
}

impl TryFrom<JsonRpcEnvelope> for StreamJsonRpcMessage {
    type Error = StreamConnectionError;

    fn try_from(envelope: JsonRpcEnvelope) -> ConnectionResult<Self> {
        if envelope.jsonrpc != "2.0" {
            return Err(StreamConnectionError::UnexpectedJsonRpcEnvelope(format!(
                "{envelope:?}"
            )));
        }

        if let Some(id) = envelope.id.as_ref() {
            let metadata = envelope.metadata();
            if let Some(error) = envelope.error {
                return Ok(Self::Response {
                    id: id.to_string(),
                    result: Err(error),
                    metadata,
                });
            }

            if let Some(result) = envelope.result {
                return Ok(Self::Response {
                    id: id.to_string(),
                    result: Ok(result),
                    metadata,
                });
            }

            return Err(StreamConnectionError::UnexpectedJsonRpcEnvelope(format!(
                "{envelope:?}"
            )));
        }

        if envelope.method.as_deref() == Some("subscription") {
            #[derive(Deserialize)]
            struct SubscriptionParams {
                topic: StreamTopic,
                data: Value,
            }

            let envelope_debug = format!("{envelope:?}");
            let params =
                envelope
                    .params
                    .ok_or(StreamConnectionError::UnexpectedJsonRpcEnvelope(
                        envelope_debug,
                    ))?;
            let params: SubscriptionParams =
                serde_json::from_value(params).map_err(StreamConnectionError::DecodeJson)?;

            let update = StreamUpdate::from_subscription(params.topic, params.data)?;

            return Ok(Self::Subscription(update));
        }

        Err(StreamConnectionError::UnexpectedJsonRpcEnvelope(format!(
            "{envelope:?}"
        )))
    }
}

impl<'de> Deserialize<'de> for StreamJsonRpcMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let envelope = JsonRpcEnvelope::deserialize(deserializer)?;
        StreamJsonRpcMessage::try_from(envelope).map_err(de::Error::custom)
    }
}

impl StreamJsonRpcMessage {
    pub(in crate::stream::v1) fn into_rpc_result(
        self,
        request: &StreamJsonRpcRequest,
    ) -> ConnectionResult<Option<StreamJsonRpcResult>> {
        let Self::Response {
            id,
            result,
            metadata,
        } = self
        else {
            return Ok(None);
        };

        if &id != request.id() {
            return Ok(None);
        }

        let result = result.map_err(StreamConnectionError::JsonRpcError)?;
        let result = parse_rpc_result(request.method(), result, metadata)?;
        Ok(Some(result))
    }
}

fn parse_rpc_result(
    method: &StreamJsonRpcReqMethod,
    result: Value,
    metadata: StreamResponseMetadata,
) -> ConnectionResult<StreamJsonRpcResult> {
    match method {
        StreamJsonRpcReqMethod::Hello => {
            #[derive(Deserialize)]
            struct ResultBody {
                version: String,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            Ok(StreamJsonRpcResult::Hello(HelloResult {
                version: body.version,
                metadata,
            }))
        }
        StreamJsonRpcReqMethod::Ping => {
            if result == Value::String("pong".to_string()) {
                Ok(StreamJsonRpcResult::Pong(metadata))
            } else {
                Err(StreamConnectionError::UnexpectedJsonRpcResult {
                    method: method.to_string(),
                    result,
                })
            }
        }
        StreamJsonRpcReqMethod::Time => {
            #[derive(Deserialize)]
            struct ResultBody {
                time: i64,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            let time = DateTime::<Utc>::from_timestamp_millis(body.time)
                .ok_or(StreamConnectionError::InvalidTimestampMillis(body.time))?;
            Ok(StreamJsonRpcResult::Time(TimeResult { time, metadata }))
        }
        StreamJsonRpcReqMethod::Authenticate => {
            #[derive(Deserialize)]
            struct ResultBody {
                authenticated: bool,
                permissions: Vec<String>,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            Ok(StreamJsonRpcResult::Authenticate(AuthenticateResult {
                authenticated: body.authenticated,
                permissions: body.permissions,
                metadata,
            }))
        }
        StreamJsonRpcReqMethod::Whoami => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct ResultBody {
                api_key: String,
                user_id: String,
                permissions: Vec<String>,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            Ok(StreamJsonRpcResult::Whoami(WhoamiResult {
                api_key: body.api_key,
                user_id: body.user_id,
                permissions: body.permissions,
                metadata,
            }))
        }
        StreamJsonRpcReqMethod::Subscribe => {
            #[derive(Deserialize)]
            struct ResultBody {
                subscribed: Vec<StreamTopic>,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            Ok(StreamJsonRpcResult::Subscribe {
                subscribed: body.subscribed,
                metadata,
            })
        }
        StreamJsonRpcReqMethod::Unsubscribe => {
            #[derive(Deserialize)]
            struct ResultBody {
                unsubscribed: Vec<StreamTopic>,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            Ok(StreamJsonRpcResult::Unsubscribe {
                unsubscribed: body.unsubscribed,
                metadata,
            })
        }
        StreamJsonRpcReqMethod::UnsubscribeAll => {
            #[derive(Deserialize)]
            struct ResultBody {
                unsubscribed: Vec<StreamTopic>,
            }

            let body: ResultBody =
                serde_json::from_value(result).map_err(StreamConnectionError::DecodeJson)?;
            Ok(StreamJsonRpcResult::UnsubscribeAll {
                unsubscribed: body.unsubscribed,
                metadata,
            })
        }
    }
}

pub(in crate::stream::v1) fn topics_param(topics: Vec<StreamTopic>) -> Value {
    json!({ "topics": topics })
}

pub(in crate::stream::v1) fn topics_match(a: &[StreamTopic], b: &[StreamTopic]) -> bool {
    let set_a: HashSet<&StreamTopic> = a.iter().collect();
    let set_b: HashSet<&StreamTopic> = b.iter().collect();
    set_a == set_b
}

/// Updates emitted by a Stream v1 WebSocket connection.
#[derive(Debug, Clone)]
pub enum StreamUpdate {
    Announcements(StreamAnnouncement),
    FuturesInverseBtcUsdTicker(StreamTicker),
    FuturesInverseBtcUsdLastPrice(StreamLastPrice),
    FuturesInverseBtcUsdIndex(StreamIndex),
    FuturesInverseBtcUsdBuckets(StreamBuckets),
    FuturesInverseBtcUsdFunding(StreamFunding),
    FuturesInverseBtcUsdOhlc {
        timeframe: StreamOhlcTimeframe,
        candle: StreamOhlc,
    },
    FuturesInverseBtcUsdIsolatedTrades(StreamIsolatedTradeEvent),
    FuturesInverseBtcUsdCrossOrders(StreamCrossOrderEvent),
    FuturesInverseBtcUsdCrossPosition(StreamCrossPositionEvent),
    WalletDeposit(StreamWalletDeposit),
    WalletWithdrawal(StreamWalletWithdrawal),
    ConnectionStatus(StreamConnectionStatus),
}

impl StreamUpdate {
    /// Returns the subscription topic for topic updates, or `None` for connection-status updates.
    pub fn topic(&self) -> Option<StreamTopic> {
        match self {
            Self::Announcements(_) => Some(StreamTopic::Announcements),
            Self::FuturesInverseBtcUsdTicker(_) => Some(StreamTopic::FuturesInverseBtcUsdTicker),
            Self::FuturesInverseBtcUsdLastPrice(_) => {
                Some(StreamTopic::FuturesInverseBtcUsdLastPrice)
            }
            Self::FuturesInverseBtcUsdIndex(_) => Some(StreamTopic::FuturesInverseBtcUsdIndex),
            Self::FuturesInverseBtcUsdBuckets(_) => Some(StreamTopic::FuturesInverseBtcUsdBuckets),
            Self::FuturesInverseBtcUsdFunding(_) => Some(StreamTopic::FuturesInverseBtcUsdFunding),
            Self::FuturesInverseBtcUsdOhlc { timeframe, .. } => {
                Some(StreamTopic::FuturesInverseBtcUsdOhlc(*timeframe))
            }
            Self::FuturesInverseBtcUsdIsolatedTrades(_) => {
                Some(StreamTopic::FuturesInverseBtcUsdIsolatedTrades)
            }
            Self::FuturesInverseBtcUsdCrossOrders(_) => {
                Some(StreamTopic::FuturesInverseBtcUsdCrossOrders)
            }
            Self::FuturesInverseBtcUsdCrossPosition(_) => {
                Some(StreamTopic::FuturesInverseBtcUsdCrossPosition)
            }
            Self::WalletDeposit(_) => Some(StreamTopic::WalletDeposit),
            Self::WalletWithdrawal(_) => Some(StreamTopic::WalletWithdrawal),
            Self::ConnectionStatus(_) => None,
        }
    }

    pub(in crate::stream::v1) fn from_subscription(
        topic: StreamTopic,
        data: Value,
    ) -> ConnectionResult<Self> {
        Ok(match topic {
            StreamTopic::Announcements => Self::Announcements(decode_subscription_data(data)?),
            StreamTopic::FuturesInverseBtcUsdTicker => {
                Self::FuturesInverseBtcUsdTicker(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdLastPrice => {
                Self::FuturesInverseBtcUsdLastPrice(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdIndex => {
                Self::FuturesInverseBtcUsdIndex(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdBuckets => {
                Self::FuturesInverseBtcUsdBuckets(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdFunding => {
                Self::FuturesInverseBtcUsdFunding(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdOhlc(timeframe) => Self::FuturesInverseBtcUsdOhlc {
                timeframe,
                candle: decode_subscription_data(data)?,
            },
            StreamTopic::FuturesInverseBtcUsdIsolatedTrades => {
                Self::FuturesInverseBtcUsdIsolatedTrades(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdCrossOrders => {
                Self::FuturesInverseBtcUsdCrossOrders(decode_subscription_data(data)?)
            }
            StreamTopic::FuturesInverseBtcUsdCrossPosition => {
                Self::FuturesInverseBtcUsdCrossPosition(decode_subscription_data(data)?)
            }
            StreamTopic::WalletDeposit => Self::WalletDeposit(decode_subscription_data(data)?),
            StreamTopic::WalletWithdrawal => {
                Self::WalletWithdrawal(decode_subscription_data(data)?)
            }
        })
    }
}

impl From<StreamConnectionStatus> for StreamUpdate {
    fn from(value: StreamConnectionStatus) -> Self {
        Self::ConnectionStatus(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_subscription_update(topic: &str, data: &str) -> StreamUpdate {
        let json = format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "subscription",
                "params": {{
                    "topic": "{topic}",
                    "data": {data}
                }}
            }}"#
        );

        let message: StreamJsonRpcMessage =
            serde_json::from_str(&json).expect("must parse subscription message");
        let StreamJsonRpcMessage::Subscription(update) = message else {
            panic!("expected subscription message");
        };

        update
    }

    #[test]
    fn stream_topic_round_trips_dynamic_ohlc_topic() {
        let topic = StreamTopic::FuturesInverseBtcUsdOhlc(StreamOhlcTimeframe::OneHour);
        let encoded = serde_json::to_string(&topic).expect("must serialize topic");
        assert_eq!(encoded, "\"futures/inverse/btc_usd/ohlc/1h\"");

        let decoded: StreamTopic = serde_json::from_str(&encoded).expect("must deserialize topic");
        assert_eq!(decoded, topic);
    }

    #[test]
    fn json_rpc_request_serializes_object_params() {
        let request = StreamJsonRpcRequest::new_with_id(
            StreamJsonRpcReqMethod::Subscribe,
            "abc",
            Some(topics_param(vec![StreamTopic::WalletDeposit])),
        );

        let json = String::from_utf8(request.try_to_bytes().expect("must serialize request"))
            .expect("request must be utf8");
        assert_eq!(
            json,
            r#"{"jsonrpc":"2.0","method":"subscribe","id":"abc","params":{"topics":["wallet/deposit"]}}"#
        );
    }

    #[test]
    fn json_rpc_message_deserializes_subscription_notification() {
        let update = decode_subscription_update(
            "futures/inverse/btc_usd/lastPrice",
            r#"{ "time": 0, "lastPrice": 100000 }"#,
        );

        assert_eq!(
            update.topic(),
            Some(StreamTopic::FuturesInverseBtcUsdLastPrice)
        );

        let StreamUpdate::FuturesInverseBtcUsdLastPrice(last_price) = update else {
            panic!("expected last price update");
        };

        assert_eq!(last_price.time().timestamp_millis(), 0);
        assert_eq!(last_price.last_price(), 100000.0);
    }

    #[test]
    fn subscription_notifications_decode_public_payloads() {
        let update = decode_subscription_update(
            "announcements",
            r#"{ "id": "ann-1", "title": "title", "message": "message", "link": "https://example.com" }"#,
        );
        let StreamUpdate::Announcements(announcement) = update else {
            panic!("expected announcement update");
        };
        assert_eq!(announcement.id(), "ann-1");
        assert_eq!(announcement.title(), "title");
        assert_eq!(announcement.message(), "message");
        assert_eq!(announcement.link(), "https://example.com");

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/ticker",
            r#"{ "time": 0, "lastPrice": 100000, "index": null, "funding": { "rate": 0.01, "time": 60000 } }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdTicker(ticker) = update else {
            panic!("expected ticker update");
        };
        assert_eq!(ticker.time().timestamp_millis(), 0);
        assert_eq!(ticker.last_price(), Some(100000.0));
        assert_eq!(ticker.index(), None);
        assert_eq!(ticker.funding().rate(), 0.01);
        assert_eq!(ticker.funding().time().timestamp_millis(), 60000);

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/index",
            r#"{ "time": 0, "index": 100001 }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdIndex(index) = update else {
            panic!("expected index update");
        };
        assert_eq!(index.time().timestamp_millis(), 0);
        assert_eq!(index.index(), 100001.0);

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/buckets",
            r#"{ "time": 0, "buckets": [{ "minSize": 1, "maxSize": 2, "askPrice": 3, "bidPrice": 4 }] }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdBuckets(buckets) = update else {
            panic!("expected buckets update");
        };
        assert_eq!(buckets.time().timestamp_millis(), 0);
        assert_eq!(buckets.buckets()[0].min_size(), 1.0);
        assert_eq!(buckets.buckets()[0].max_size(), 2.0);
        assert_eq!(buckets.buckets()[0].ask_price(), 3.0);
        assert_eq!(buckets.buckets()[0].bid_price(), 4.0);

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/funding",
            r#"{ "pair": "btc_usd", "current": { "rate": -0.01, "time": 120000 } }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdFunding(funding) = update else {
            panic!("expected funding update");
        };
        assert_eq!(funding.pair(), "btc_usd");
        assert_eq!(funding.current().rate(), -0.01);
        assert_eq!(funding.current().time().timestamp_millis(), 120000);

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/ohlc/1m",
            r#"{ "time": 0, "open": 1, "high": 2, "low": 3, "close": 4, "volume": 5 }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdOhlc { timeframe, candle } = update else {
            panic!("expected ohlc update");
        };
        assert_eq!(timeframe, StreamOhlcTimeframe::OneMinute);
        assert_eq!(candle.time().timestamp_millis(), 0);
        assert_eq!(candle.open(), 1.0);
        assert_eq!(candle.high(), 2.0);
        assert_eq!(candle.low(), 3.0);
        assert_eq!(candle.close(), 4.0);
        assert_eq!(candle.volume(), 5.0);
    }

    #[test]
    fn subscription_notifications_decode_private_payloads() {
        let update = decode_subscription_update(
            "futures/inverse/btc_usd/isolated/trades",
            r#"{ "pair": "btc_usd", "event": "open", "trade": { "id": "trade-1", "side": "buy", "type": "limit", "quantity": 1, "margin": 2, "leverage": 3, "price": 4, "openingFee": 5, "createdAt": 0, "clientId": "client-1" } }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdIsolatedTrades(event) = update else {
            panic!("expected isolated trade update");
        };
        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "open");
        assert_eq!(event.trade().id(), Some("trade-1"));
        assert_eq!(event.trade().side(), Some("buy"));
        assert_eq!(event.trade().trade_type(), Some("limit"));
        assert_eq!(event.trade().quantity(), Some(1.0));
        assert_eq!(event.trade().margin(), Some(2.0));
        assert_eq!(event.trade().leverage(), Some(3.0));
        assert_eq!(event.trade().price(), Some(4.0));
        assert_eq!(event.trade().opening_fee(), Some(5.0));
        assert_eq!(event.trade().created_at().unwrap().timestamp_millis(), 0);
        assert_eq!(event.trade().client_id(), Some("client-1"));

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/cross/orders",
            r#"{ "pair": "btc_usd", "event": "new", "order": { "id": "order-1", "side": "buy", "type": "limit", "quantity": 1, "price": 2, "tradingFee": 3, "clientId": "client-1", "createdAt": 0 } }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdCrossOrders(event) = update else {
            panic!("expected cross order update");
        };
        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "new");
        assert_eq!(event.order().id(), Some("order-1"));
        assert_eq!(event.order().side(), Some("buy"));
        assert_eq!(event.order().order_type(), Some("limit"));
        assert_eq!(event.order().quantity(), Some(1.0));
        assert_eq!(event.order().price(), Some(2.0));
        assert_eq!(event.order().trading_fee(), Some(3.0));
        assert_eq!(event.order().client_id(), Some("client-1"));
        assert_eq!(event.order().created_at().unwrap().timestamp_millis(), 0);

        let update = decode_subscription_update(
            "futures/inverse/btc_usd/cross/position",
            r#"{ "pair": "btc_usd", "event": "new", "position": { "quantity": 1, "leverage": 2, "margin": 3, "entryPrice": 4, "liquidation": 5, "totalPl": 6, "fundingFees": 7, "tradingFees": 8, "initialMargin": 9, "maintenanceMargin": 10, "runningMargin": 11, "deltaPl": 12, "updatedAt": 0 } }"#,
        );
        let StreamUpdate::FuturesInverseBtcUsdCrossPosition(event) = update else {
            panic!("expected cross position update");
        };
        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "new");
        assert_eq!(event.position().quantity(), Some(1.0));
        assert_eq!(event.position().leverage(), Some(2.0));
        assert_eq!(event.position().margin(), Some(3.0));
        assert_eq!(event.position().entry_price(), Some(4.0));
        assert_eq!(event.position().liquidation(), Some(5.0));
        assert_eq!(event.position().total_pl(), Some(6.0));
        assert_eq!(event.position().funding_fees(), Some(7.0));
        assert_eq!(event.position().trading_fees(), Some(8.0));
        assert_eq!(event.position().initial_margin(), Some(9.0));
        assert_eq!(event.position().maintenance_margin(), Some(10.0));
        assert_eq!(event.position().running_margin(), Some(11.0));
        assert_eq!(event.position().delta_pl(), Some(12.0));
        assert_eq!(event.position().updated_at().unwrap().timestamp_millis(), 0);

        let update = decode_subscription_update(
            "wallet/deposit",
            r#"{ "currency": "btc", "network": "lightning", "id": "deposit-1", "amount": 1, "balance": 2, "status": "confirmed", "txId": "tx-1" }"#,
        );
        let StreamUpdate::WalletDeposit(deposit) = update else {
            panic!("expected deposit update");
        };
        assert_eq!(deposit.currency(), "btc");
        assert_eq!(deposit.network(), "lightning");
        assert_eq!(deposit.id(), "deposit-1");
        assert_eq!(deposit.amount(), 1.0);
        assert_eq!(deposit.balance(), 2.0);
        assert_eq!(deposit.status(), "confirmed");
        assert_eq!(deposit.tx_id(), Some("tx-1"));

        let update = decode_subscription_update(
            "wallet/withdrawal",
            r#"{ "currency": "btc", "network": "lightning", "id": "withdrawal-1", "amount": 1, "fee": 2, "balance": 3, "status": "confirmed", "txId": "tx-1" }"#,
        );
        let StreamUpdate::WalletWithdrawal(withdrawal) = update else {
            panic!("expected withdrawal update");
        };
        assert_eq!(withdrawal.currency(), "btc");
        assert_eq!(withdrawal.network(), "lightning");
        assert_eq!(withdrawal.id(), "withdrawal-1");
        assert_eq!(withdrawal.amount(), 1.0);
        assert_eq!(withdrawal.fee(), 2.0);
        assert_eq!(withdrawal.balance(), 3.0);
        assert_eq!(withdrawal.status(), "confirmed");
        assert_eq!(withdrawal.tx_id(), Some("tx-1"));
    }

    #[test]
    fn json_rpc_message_deserializes_time_response_metadata() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "abc",
            "result": { "time": 1747035005657 },
            "usIn": 1,
            "usOut": 3,
            "usDiff": 2,
            "rateLimit": { "remaining": 9, "limit": 10 }
        }"#;

        let message: StreamJsonRpcMessage = serde_json::from_str(json).expect("must parse message");
        let request = StreamJsonRpcRequest::new_with_id(StreamJsonRpcReqMethod::Time, "abc", None);
        let result = message
            .into_rpc_result(&request)
            .expect("must decode result")
            .expect("must match id");

        let StreamJsonRpcResult::Time(time) = result else {
            panic!("expected time result");
        };

        assert_eq!(time.time().timestamp_millis(), 1747035005657);
        assert_eq!(time.metadata().us_in(), Some(1));
        assert_eq!(time.metadata().us_out(), Some(3));
        assert_eq!(time.metadata().us_diff(), Some(2));
        assert_eq!(time.metadata().rate_limit().unwrap().remaining(), 9);
        assert_eq!(time.metadata().rate_limit().unwrap().limit(), 10);
    }
}
