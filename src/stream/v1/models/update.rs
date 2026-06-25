use serde::de::DeserializeOwned;
use serde_json::Value;

use super::super::{
    error::{ConnectionResult, StreamConnectionError},
    state::StreamConnectionStatus,
};
use super::{
    market::{
        StreamAnnouncement, StreamBuckets, StreamFunding, StreamIndex, StreamLastPrice, StreamOhlc,
        StreamTicker,
    },
    topic::{StreamOhlcTimeframe, StreamTopic},
    trade::{StreamCrossOrderEvent, StreamCrossPositionEvent, StreamIsolatedTradeEvent},
    wallet::{StreamWalletDeposit, StreamWalletWithdrawal},
};

fn decode_subscription_data<T>(data: Value) -> ConnectionResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(data).map_err(StreamConnectionError::DecodeJson)
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
    use serde_json::json;

    use super::*;

    fn assert_subscription_topic(topic: StreamTopic, data: Value) {
        let update = StreamUpdate::from_subscription(topic.clone(), data)
            .expect("must decode subscription update");

        assert_eq!(update.topic(), Some(topic));
    }

    #[test]
    fn from_subscription_maps_topics_to_update_variants() {
        assert_subscription_topic(
            StreamTopic::Announcements,
            json!({ "id": "ann-1", "title": "title", "message": "message", "link": "https://example.com" }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdTicker,
            json!({ "time": 0, "lastPrice": 1, "index": null, "funding": { "rate": 0.0, "time": 0 } }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdLastPrice,
            json!({ "time": 0, "lastPrice": 1 }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdIndex,
            json!({ "time": 0, "index": 1 }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdBuckets,
            json!({ "time": 0, "buckets": [{ "minSize": 1, "maxSize": 2, "askPrice": 3, "bidPrice": 4 }] }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdFunding,
            json!({ "pair": "btc_usd", "current": { "rate": 0.0, "time": 0 } }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdOhlc(StreamOhlcTimeframe::OneMinute),
            json!({ "time": 0, "open": 1, "high": 2, "low": 3, "close": 4, "volume": 5 }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdIsolatedTrades,
            json!({ "pair": "btc_usd", "event": "open", "trade": {} }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdCrossOrders,
            json!({ "pair": "btc_usd", "event": "new", "order": {} }),
        );
        assert_subscription_topic(
            StreamTopic::FuturesInverseBtcUsdCrossPosition,
            json!({ "pair": "btc_usd", "event": "new", "position": {} }),
        );
        assert_subscription_topic(
            StreamTopic::WalletDeposit,
            json!({ "currency": "btc", "network": "lightning", "id": "deposit-1", "amount": 1, "balance": 2, "status": "confirmed" }),
        );
        assert_subscription_topic(
            StreamTopic::WalletWithdrawal,
            json!({ "currency": "btc", "network": "lightning", "id": "withdrawal-1", "amount": 1, "fee": 2, "balance": 3, "status": "confirmed" }),
        );
    }

    #[test]
    fn connection_status_update_has_no_topic() {
        let update = StreamUpdate::ConnectionStatus(StreamConnectionStatus::Connected);

        assert_eq!(update.topic(), None);
    }
}
