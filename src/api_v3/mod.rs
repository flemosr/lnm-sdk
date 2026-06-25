pub(crate) mod rest;

pub use rest::{
    RestClient,
    config::RestClientConfig,
    repositories::{
        AccountRepository, FuturesCrossRepository, FuturesDataRepository,
        FuturesIsolatedRepository, OracleRepository, UtilitiesRepository,
    },
};

pub mod error {
    pub use crate::shared::{
        models::error::{
            ClientIdValidationError, CrossLeverageValidationError, CrossQuantityValidationError,
            LeverageValidationError, MarginValidationError, OhlcRangeParseError,
            PercentageCappedValidationError, PercentageValidationError, PriceValidationError,
            QuantityValidationError, TradeValidationError,
        },
        rest::error::RestApiError,
    };

    pub use super::rest::{
        error::RestApiV3Error,
        models::error::{CrossExposureValidationError, FuturesIsolatedTradeRequestValidationError},
    };
}

pub mod models {
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
            TradeExecution, TradeExecutionType, TradeSide, TradeSize, TradeStatus,
            util as trade_util,
        },
    };

    #[allow(deprecated)]
    pub use crate::shared::models::quantity::Quantity;

    pub use super::rest::models::{
        account::Account,
        funding::{CrossFunding, FundingSettlement, IsolatedFunding},
        page::Page,
        ticker::Ticker,
        trade::{CrossExposure, CrossExposureRunning, CrossOrder, CrossPosition, Trade},
        transfer::CrossTransfer,
    };
}
