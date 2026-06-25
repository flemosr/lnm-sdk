use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::shared::models::{
    ohlc::OhlcCandle,
    oracle::{Index, LastPrice},
    price::Price,
    serde_util,
    ticker::TickerPrice,
};

/// Inverse futures last trade price notification payload.
pub type StreamLastPrice = LastPrice;

/// Inverse futures index price notification payload.
pub type StreamIndex = Index;

/// One inverse futures volume ladder bucket.
pub type StreamBucket = TickerPrice;

/// Inverse futures OHLC candle notification payload.
pub type StreamOhlc = OhlcCandle;

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
    #[serde(deserialize_with = "serde_util::datetime_rfc3339_or_millis::deserialize")]
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
    #[serde(deserialize_with = "serde_util::datetime_rfc3339_or_millis::deserialize")]
    time: DateTime<Utc>,
    last_price: Option<Price>,
    index: Option<Price>,
    funding: StreamFundingRate,
}

impl StreamTicker {
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn last_price(&self) -> Option<Price> {
        self.last_price
    }

    pub fn index(&self) -> Option<Price> {
        self.index
    }

    pub fn funding(&self) -> &StreamFundingRate {
        &self.funding
    }
}

/// Inverse futures volume ladder bucket notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBuckets {
    #[serde(deserialize_with = "serde_util::datetime_rfc3339_or_millis::deserialize")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_announcement_deserializes() {
        let announcement: StreamAnnouncement = serde_json::from_str(
            r#"{ "id": "ann-1", "title": "title", "message": "message", "link": "https://example.com" }"#,
        )
        .expect("must deserialize announcement");

        assert_eq!(announcement.id(), "ann-1");
        assert_eq!(announcement.title(), "title");
        assert_eq!(announcement.message(), "message");
        assert_eq!(announcement.link(), "https://example.com");
    }

    #[test]
    fn stream_ticker_deserializes() {
        let ticker: StreamTicker = serde_json::from_str(
            r#"{ "time": 0, "lastPrice": 100000, "index": null, "funding": { "rate": 0.01, "time": 60000 } }"#,
        )
        .expect("must deserialize ticker");

        assert_eq!(ticker.time().timestamp_millis(), 0);
        assert_eq!(ticker.last_price().unwrap().as_f64(), 100000.0);
        assert_eq!(ticker.index(), None);
        assert_eq!(ticker.funding().rate(), 0.01);
        assert_eq!(ticker.funding().time().timestamp_millis(), 60000);
    }

    #[test]
    fn stream_index_deserializes() {
        let index: StreamIndex = serde_json::from_str(r#"{ "time": 0, "index": 100001 }"#)
            .expect("must deserialize index");

        assert_eq!(index.time().timestamp_millis(), 0);
        assert_eq!(index.index().as_f64(), 100001.0);
    }

    #[test]
    fn stream_buckets_deserializes() {
        let buckets: StreamBuckets = serde_json::from_str(
            r#"{ "time": 0, "buckets": [{ "minSize": 1, "maxSize": 2, "askPrice": 3, "bidPrice": 4 }] }"#,
        )
        .expect("must deserialize buckets");

        assert_eq!(buckets.time().timestamp_millis(), 0);
        assert_eq!(buckets.buckets()[0].min_size(), 1);
        assert_eq!(buckets.buckets()[0].max_size(), 2);
        assert_eq!(buckets.buckets()[0].ask_price().as_f64(), 3.0);
        assert_eq!(buckets.buckets()[0].bid_price().as_f64(), 4.0);
    }

    #[test]
    fn stream_funding_deserializes() {
        let funding: StreamFunding = serde_json::from_str(
            r#"{ "pair": "btc_usd", "current": { "rate": -0.01, "time": 120000 } }"#,
        )
        .expect("must deserialize funding");

        assert_eq!(funding.pair(), "btc_usd");
        assert_eq!(funding.current().rate(), -0.01);
        assert_eq!(funding.current().time().timestamp_millis(), 120000);
    }

    #[test]
    fn stream_ohlc_deserializes() {
        let candle: StreamOhlc = serde_json::from_str(
            r#"{ "time": 0, "open": 1, "high": 2, "low": 3, "close": 4, "volume": 5 }"#,
        )
        .expect("must deserialize ohlc candle");

        assert_eq!(candle.time().timestamp_millis(), 0);
        assert_eq!(candle.open().as_f64(), 1.0);
        assert_eq!(candle.high().as_f64(), 2.0);
        assert_eq!(candle.low().as_f64(), 3.0);
        assert_eq!(candle.close().as_f64(), 4.0);
        assert_eq!(candle.volume(), 5);
    }
}
