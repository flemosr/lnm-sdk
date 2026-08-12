use std::fmt;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{price::Price, serde_util};

/// Index price data point.
///
/// Represents the index price at a specific point in time. This model is delivered as the payload
/// of the Stream API's `futures/inverse/btc_usd/index` topic.
///
/// # Examples
///
/// ```
/// use lnm_sdk::stream::v1::models::StreamUpdate;
///
/// fn handle_update(update: StreamUpdate) {
///     if let StreamUpdate::FuturesInverseBtcUsdIndex(index) = update {
///         println!("Time: {}", index.time());
///         println!("Index: {}", index.index());
///     }
/// }
/// ```
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Index {
    #[serde(deserialize_with = "serde_util::datetime_rfc3339_or_millis::deserialize")]
    time: DateTime<Utc>,
    index: Price,
}

impl Index {
    /// Timestamp of the index data point.
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    /// Index price value.
    pub fn index(&self) -> Price {
        self.index
    }

    pub fn as_data_str(&self) -> String {
        format!("time: {}\nindex: {}", self.time.to_rfc3339(), self.index)
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Index:")?;
        for line in self.as_data_str().lines() {
            write!(f, "\n  {line}")?;
        }
        Ok(())
    }
}

/// Last traded price data point.
///
/// Represents the last traded price at a specific point in time. This model is delivered as the
/// payload of the Stream API's `futures/inverse/btc_usd/lastPrice` topic.
///
/// # Examples
///
/// ```
/// use lnm_sdk::stream::v1::models::StreamUpdate;
///
/// fn handle_update(update: StreamUpdate) {
///     if let StreamUpdate::FuturesInverseBtcUsdLastPrice(last_price) = update {
///         println!("Time: {}", last_price.time());
///         println!("Last price: {}", last_price.last_price());
///     }
/// }
/// ```
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LastPrice {
    #[serde(deserialize_with = "serde_util::datetime_rfc3339_or_millis::deserialize")]
    time: DateTime<Utc>,
    last_price: Price,
}

impl LastPrice {
    /// Timestamp of the last price data point.
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    /// Last price value.
    pub fn last_price(&self) -> Price {
        self.last_price
    }

    pub fn as_data_str(&self) -> String {
        format!(
            "time: {}\nlast_price: {}",
            self.time.to_rfc3339(),
            self.last_price
        )
    }
}

impl fmt::Display for LastPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Last Price:")?;
        for line in self.as_data_str().lines() {
            write!(f, "\n  {line}")?;
        }
        Ok(())
    }
}
