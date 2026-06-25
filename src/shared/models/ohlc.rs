use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::{error::OhlcRangeParseError, price::Price, serde_util};

/// Time range for OHLC (Open-High-Low-Close) candles.
///
/// Specifies the duration of each candlestick when querying historical price data or subscribing
/// to live candle updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OhlcRange {
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

impl OhlcRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            OhlcRange::OneMinute => "1m",
            OhlcRange::ThreeMinutes => "3m",
            OhlcRange::FiveMinutes => "5m",
            OhlcRange::TenMinutes => "10m",
            OhlcRange::FifteenMinutes => "15m",
            OhlcRange::ThirtyMinutes => "30m",
            OhlcRange::FortyFiveMinutes => "45m",
            OhlcRange::OneHour => "1h",
            OhlcRange::TwoHours => "2h",
            OhlcRange::ThreeHours => "3h",
            OhlcRange::FourHours => "4h",
            OhlcRange::OneDay => "1d",
            OhlcRange::OneWeek => "1w",
            OhlcRange::OneMonth => "1month",
            OhlcRange::ThreeMonths => "3months",
        }
    }
}

impl fmt::Display for OhlcRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for OhlcRange {
    type Err = OhlcRangeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "1m" => Ok(OhlcRange::OneMinute),
            "3m" => Ok(OhlcRange::ThreeMinutes),
            "5m" => Ok(OhlcRange::FiveMinutes),
            "10m" => Ok(OhlcRange::TenMinutes),
            "15m" => Ok(OhlcRange::FifteenMinutes),
            "30m" => Ok(OhlcRange::ThirtyMinutes),
            "45m" => Ok(OhlcRange::FortyFiveMinutes),
            "1h" => Ok(OhlcRange::OneHour),
            "2h" => Ok(OhlcRange::TwoHours),
            "3h" => Ok(OhlcRange::ThreeHours),
            "4h" => Ok(OhlcRange::FourHours),
            "1d" => Ok(OhlcRange::OneDay),
            "1w" => Ok(OhlcRange::OneWeek),
            "1month" => Ok(OhlcRange::OneMonth),
            "3months" => Ok(OhlcRange::ThreeMonths),
            _ => Err(OhlcRangeParseError::Unknown {
                value: value.to_string(),
            }),
        }
    }
}

impl Serialize for OhlcRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OhlcRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(de::Error::custom)
    }
}

/// OHLC (Open-High-Low-Close) candlestick data.
///
/// Represents price and volume data for a specific time period.
///
/// # Examples
///
/// ```no_run
/// # async fn example(rest: lnm_sdk::api_v3::RestClient) -> Result<(), Box<dyn std::error::Error>> {
/// use lnm_sdk::api_v3::models::{OhlcCandle, OhlcRange, Page};
///
/// let candles: Page<OhlcCandle> = rest
///     .futures_data
///     .get_candles(None, None, None, Some(OhlcRange::OneHour), None)
///     .await?;
///
/// for candle in candles.data() {
///     println!("Time: {}", candle.time());
///     println!("Open: {}", candle.open());
///     println!("High: {}", candle.high());
///     println!("Low: {}", candle.low());
///     println!("Close: {}", candle.close());
///     println!("Volume: {}", candle.volume());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct OhlcCandle {
    #[serde(deserialize_with = "serde_util::datetime_rfc3339_or_millis::deserialize")]
    time: DateTime<Utc>,
    open: Price,
    high: Price,
    low: Price,
    close: Price,
    volume: u64,
}

impl OhlcCandle {
    /// Timestamp of the OHLC candle.
    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    /// Opening price.
    pub fn open(&self) -> Price {
        self.open
    }

    /// Highest price.
    pub fn high(&self) -> Price {
        self.high
    }

    /// Lowest price.
    pub fn low(&self) -> Price {
        self.low
    }

    /// Closing price.
    pub fn close(&self) -> Price {
        self.close
    }

    /// Trading volume.
    pub fn volume(&self) -> u64 {
        self.volume
    }

    pub fn as_data_str(&self) -> String {
        format!(
            "time: {}\nopen: {}\nhigh: {}\nlow: {}\nclose: {}\nvolume: {}",
            self.time.to_rfc3339(),
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume
        )
    }
}

impl fmt::Display for OhlcCandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OHLC Candle:")?;
        for line in self.as_data_str().lines() {
            write!(f, "\n  {line}")?;
        }
        Ok(())
    }
}
