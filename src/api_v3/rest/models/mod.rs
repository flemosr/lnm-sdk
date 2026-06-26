pub(in crate::api_v3) mod account;
pub(in crate::api_v3) mod error;
pub(in crate::api_v3) mod funding;
pub(in crate::api_v3) mod ohlc_candle;
pub(in crate::api_v3) mod oracle;
pub(in crate::api_v3) mod page;
pub(in crate::api_v3) mod ticker;
pub(in crate::api_v3) mod trade;
pub(in crate::api_v3) mod transfer;

pub use uuid::Uuid;

pub use crate::shared::models::{
    SATS_PER_BTC,
    client_id::ClientId,
    cross_leverage::CrossLeverage,
    cross_quantity::CrossQuantity,
    leverage::Leverage,
    margin::Margin,
    ohlc::{OhlcCandle, OhlcRange},
    oracle::{Index, LastPrice},
    price::{Percentage, PercentageCapped, Price},
    quantity::{OrderQuantity, QuantityLike},
    ticker::TickerPrice,
    trade::{
        TradeExecution, TradeExecutionType, TradeSide, TradeSize, TradeStatus, util as trade_util,
    },
};

#[allow(deprecated)]
pub use crate::shared::models::quantity::Quantity;

pub use account::Account;
pub use funding::{CrossFunding, FundingSettlement, IsolatedFunding};
pub use page::Page;
pub use ticker::Ticker;
pub use trade::{CrossExposure, CrossExposureRunning, CrossOrder, CrossPosition, Trade};
pub use transfer::CrossTransfer;
