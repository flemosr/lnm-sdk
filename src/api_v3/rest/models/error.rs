use thiserror::Error;

use crate::shared::models::error::{MarginValidationError, QuantityValidationError};

use super::{client_id::ClientId, cross_leverage::CrossLeverage, cross_quantity::CrossQuantity};

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
    TooLow { value: i128 },

    #[error(
        "CrossLeverage must be less than or equal to {}. Value: {value}",
        CrossLeverage::MAX
    )]
    TooHigh { value: u128 },

    #[error("CrossLeverage must be an integer. Value: {value}")]
    NotAnInteger { value: f64 },
}

#[derive(Debug, Error)]
pub enum CrossQuantityValidationError {
    #[error(
        "CrossQuantity must be at least {}. Value: {value}",
        CrossQuantity::MIN
    )]
    TooLow { value: i128 },

    #[error(
        "CrossQuantity must be less than or equal to {}. Value: {value}",
        CrossQuantity::HARD_MAX
    )]
    TooHigh { value: u128 },

    #[error("CrossQuantity must be an integer. Value: {value}")]
    NotAnInteger { value: f64 },
}

#[derive(Debug, Error)]
pub enum CrossExposureValidationError {
    #[error("Cross margin is too low for the requested exposure")]
    CrossMarginTooLow,

    #[error("[CrossMarginValidation] {0}")]
    CrossMargin(#[from] MarginValidationError),

    #[error("Cross exposure running position is missing an entry price")]
    MissingEntryPrice,

    #[error("Cross quantity {qtd} exceeds maximum {max_qtd} for leverage {leverage}")]
    CrossQuantityTooHighForLeverage {
        qtd: CrossQuantity,
        max_qtd: CrossQuantity,
        leverage: CrossLeverage,
    },

    #[error("[CrossQuantityValidation] {0}")]
    CrossQuantityValidation(#[from] CrossQuantityValidationError),
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
