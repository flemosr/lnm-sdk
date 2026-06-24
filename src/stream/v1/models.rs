use std::{collections::HashSet, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
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

/// Raw subscription notification payload.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamSubscription {
    topic: StreamTopic,
    data: Value,
}

impl StreamSubscription {
    pub fn topic(&self) -> &StreamTopic {
        &self.topic
    }

    pub fn data(&self) -> &Value {
        &self.data
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
#[derive(Clone, Debug, PartialEq)]
pub(in crate::stream::v1) enum StreamJsonRpcMessage {
    Response {
        id: String,
        result: Result<Value, StreamJsonRpcError>,
        metadata: StreamResponseMetadata,
    },
    Subscription(StreamSubscription),
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

            return Ok(Self::Subscription(StreamSubscription {
                topic: params.topic,
                data: params.data,
            }));
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
    Subscription(StreamSubscription),
    ConnectionStatus(StreamConnectionStatus),
}

impl From<StreamConnectionStatus> for StreamUpdate {
    fn from(value: StreamConnectionStatus) -> Self {
        Self::ConnectionStatus(value)
    }
}

impl From<StreamSubscription> for StreamUpdate {
    fn from(value: StreamSubscription) -> Self {
        Self::Subscription(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "subscription",
            "params": {
                "topic": "futures/inverse/btc_usd/lastPrice",
                "data": { "time": 0, "lastPrice": 100000 }
            }
        }"#;

        let message: StreamJsonRpcMessage = serde_json::from_str(json).expect("must parse message");
        let StreamJsonRpcMessage::Subscription(subscription) = message else {
            panic!("expected subscription message");
        };

        assert_eq!(
            subscription.topic(),
            &StreamTopic::FuturesInverseBtcUsdLastPrice
        );
        assert_eq!(subscription.data()["lastPrice"], 100000);
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
