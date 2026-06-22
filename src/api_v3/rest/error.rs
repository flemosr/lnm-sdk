use thiserror::Error;

use super::models::error::FuturesIsolatedTradeRequestValidationError;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RestApiV3Error {
    #[error("Invalid futures isolated trade request error: {0}")]
    FuturesIsolatedTradeRequestValidation(FuturesIsolatedTradeRequestValidationError),

    #[error("Unexpected 'ping' response error: {0}")]
    UnexpectedPingResponse(String),
}
