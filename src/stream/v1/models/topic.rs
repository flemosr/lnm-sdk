use std::{collections::HashSet, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::{Value, json};

use crate::shared::models::ohlc::OhlcRange;

use super::super::error::{ConnectionResult, StreamConnectionError};

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
    FuturesInverseBtcUsdOhlc(OhlcRange),
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
                        OhlcRange::from_str(timeframe).map_err(|_| {
                            StreamConnectionError::UnknownOhlcTimeframe(timeframe.to_string())
                        })?,
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

pub(in crate::stream::v1) fn topics_param(topics: Vec<StreamTopic>) -> Value {
    json!({ "topics": topics })
}

pub(in crate::stream::v1) fn topics_match(a: &[StreamTopic], b: &[StreamTopic]) -> bool {
    let set_a: HashSet<&StreamTopic> = a.iter().collect();
    let set_b: HashSet<&StreamTopic> = b.iter().collect();
    set_a == set_b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_topic_round_trips_dynamic_ohlc_topic() {
        let topic = StreamTopic::FuturesInverseBtcUsdOhlc(OhlcRange::OneHour);
        let encoded = serde_json::to_string(&topic).expect("must serialize topic");
        assert_eq!(encoded, "\"futures/inverse/btc_usd/ohlc/1h\"");

        let decoded: StreamTopic = serde_json::from_str(&encoded).expect("must deserialize topic");
        assert_eq!(decoded, topic);
    }
}
