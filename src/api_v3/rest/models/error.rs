use thiserror::Error;

use crate::{api_v3::models::CrossLeverage, shared::models::error::QuantityValidationError};

use super::client_id::ClientId;

#[derive(Debug, Error)]
pub enum ClientIdValidationError {
    #[error(
        "Client ID must be at least {} character(s). Length: {len}",
        ClientId::MIN_LEN
    )]
    TooShort { len: usize },

    #[error(
        "Client ID must be at most {} characters. Length: {len}",
        ClientId::MAX_LEN
    )]
    TooLong { len: usize },
}

#[derive(Debug, Error)]
pub enum CrossLeverageValidationError {
    #[error(
        "CrossLeverage must be at least {}. Value: {value}",
        CrossLeverage::MIN
    )]
    TooLow { value: u64 },

    #[error(
        "CrossLeverage must be less than or equal to {}. Value: {value}",
        CrossLeverage::MAX
    )]
    TooHigh { value: u64 },

    #[error("CrossLeverage must be an integer. Value: {value}")]
    NotAnInteger { value: f64 },
}

#[derive(Debug, Error)]
pub enum FuturesIsolatedTradeRequestValidationError {
    #[error("Price cannot be set for market orders")]
    PriceSetForMarketOrder,

    #[error("Price must be set for limit orders")]
    MissingPriceForLimitOrder,

    #[error("[QuantityValidation] {0}")]
    QuantityValidation(#[from] QuantityValidationError),

    #[error("Stop loss must be lower than the entry price")]
    StopLossHigherThanPrice,

    #[error("Take profit must be higher than the entry price")]
    TakeProfitLowerThanPrice,
}
