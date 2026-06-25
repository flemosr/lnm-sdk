use std::fmt;

use serde::Deserialize;

use super::price::Price;

/// One bid/ask price bucket for futures ticker and order-size ladder payloads.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TickerPrice {
    ask_price: Price,
    bid_price: Price,
    min_size: u64,
    max_size: u64,
}

impl TickerPrice {
    /// Get the ask price.
    pub fn ask_price(&self) -> Price {
        self.ask_price
    }

    /// Get the bid price.
    pub fn bid_price(&self) -> Price {
        self.bid_price
    }

    /// Get the minimum size.
    pub fn min_size(&self) -> u64 {
        self.min_size
    }

    /// Get the maximum size.
    pub fn max_size(&self) -> u64 {
        self.max_size
    }

    pub fn as_data_str(&self) -> String {
        format!(
            "ask_price: {}\nbid_price: {}\nmin_size: {}\nmax_size: {}",
            self.ask_price, self.bid_price, self.min_size, self.max_size
        )
    }
}

impl fmt::Display for TickerPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ticker Price:")?;
        for line in self.as_data_str().lines() {
            write!(f, "\n  {line}")?;
        }
        Ok(())
    }
}
