use thiserror::Error;

pub use crate::shared::{
    models::error::{
        ClientIdValidationError, CrossLeverageValidationError, CrossQuantityValidationError,
        LeverageValidationError, MarginValidationError, OhlcRangeParseError,
        PercentageCappedValidationError, PercentageValidationError, PriceValidationError,
        QuantityValidationError, TradeValidationError,
    },
    rest::error::RestApiError,
};

pub use super::models::error::FuturesIsolatedTradeRequestValidationError;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RestApiV3Error {
    #[error("Invalid futures isolated trade request error: {0}")]
    FuturesIsolatedTradeRequestValidation(FuturesIsolatedTradeRequestValidationError),

    #[error("Unexpected 'ping' response error: {0}")]
    UnexpectedPingResponse(String),
}
