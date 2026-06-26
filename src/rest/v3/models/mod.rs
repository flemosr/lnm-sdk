pub(in crate::rest) mod account;
pub(in crate::rest) mod error;
pub(in crate::rest) mod funding;
pub(in crate::rest) mod page;
pub(in crate::rest) mod ticker;
pub(in crate::rest) mod trade;
pub(in crate::rest) mod transfer;

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
