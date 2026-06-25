#![doc = include_str!("../README.md")]

/// API v3 implementation.
///
/// Contains all types, clients, and functionality necessary to work with API v3, including REST
/// client, models, and error types.
///
/// # Example
///
/// ```rust
/// use lnm_sdk::api_v3::{RestClient, RestClientConfig, models::*, error::*};
/// ```
pub mod api_v3;

/// Stream API implementations.
pub mod stream;

mod shared;

mod sealed {
    pub trait Sealed {}
}
