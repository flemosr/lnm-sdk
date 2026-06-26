use thiserror::Error;

use crate::shared::models::{
    cross_leverage::CrossLeverage,
    cross_quantity::CrossQuantity,
    error::{CrossQuantityValidationError, MarginValidationError, QuantityValidationError},
};

#[derive(Debug, Error)]
#[non_exhaustive]
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
#[non_exhaustive]
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
