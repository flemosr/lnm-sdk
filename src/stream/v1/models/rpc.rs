use std::fmt;

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;

use super::super::error::{ConnectionResult, StreamConnectionError, StreamJsonRpcError};
use super::{
    metadata::{StreamRateLimit, StreamResponseMetadata},
    topic::StreamTopic,
    update::StreamUpdate,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonRpcId {
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
    pub fn new(method: StreamJsonRpcReqMethod, params: Option<Value>) -> Self {
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

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn method(&self) -> &StreamJsonRpcReqMethod {
        &self.method
    }

    pub fn try_to_bytes(&self) -> ConnectionResult<Vec<u8>> {
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
struct JsonRpcEnvelope {
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
    pub fn into_rpc_result(
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

#[cfg(test)]
mod tests {
    use super::super::topic::topics_param;
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
