pub(in crate::stream::v1) mod market;
pub(in crate::stream::v1) mod metadata;
pub(in crate::stream::v1) mod rpc;
pub(in crate::stream::v1) mod topic;
pub(in crate::stream::v1) mod trade;
pub(in crate::stream::v1) mod update;
pub(in crate::stream::v1) mod wallet;

pub use uuid::Uuid;

pub use crate::shared::models::{
    client_id::ClientId,
    cross_leverage::CrossLeverage,
    leverage::Leverage,
    margin::Margin,
    ohlc::{OhlcCandle, OhlcRange},
    oracle::{Index, LastPrice},
    price::Price,
    quantity::order::OrderQuantity,
    ticker::TickerPrice,
    trade::{TradeExecutionType, TradeSide},
};

pub use market::{
    StreamAnnouncement, StreamBuckets, StreamFunding, StreamFundingRate, StreamTicker,
};
pub use metadata::{StreamRateLimit, StreamResponseMetadata};
pub use rpc::{AuthenticateResult, HelloResult, TimeResult, WhoamiResult};
pub use topic::StreamTopic;
pub use trade::{
    StreamCrossOrder, StreamCrossOrderEvent, StreamCrossPosition, StreamCrossPositionEvent,
    StreamIsolatedTrade, StreamIsolatedTradeEvent,
};
pub use update::StreamUpdate;
pub use wallet::{StreamWalletDeposit, StreamWalletWithdrawal};
