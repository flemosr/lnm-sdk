/// Number of satoshis (sats) in a Bitcoin: 100_000_000
pub const SATS_PER_BTC: f64 = 100_000_000.;

pub(crate) mod client_id;
pub(crate) mod cross_leverage;
pub(crate) mod error;
pub(crate) mod leverage;
pub(crate) mod margin;
pub(crate) mod ohlc;
pub(crate) mod oracle;
pub(crate) mod price;
pub(crate) mod quantity;
pub(crate) mod serde_util;
pub(crate) mod ticker;
pub(crate) mod trade;
