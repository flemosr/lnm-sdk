#![doc = include_str!("../README.md")]

/// REST v3 implementation.
///
/// Contains all types, clients, and functionality necessary to work with REST API v3, including
/// client, models, and error types.
///
/// # Example
///
/// ```rust
/// use lnm_sdk::rest::v3::{RestClient, RestClientConfig, models::*, error::*};
/// ```
pub mod rest;

/// Stream API implementations.
pub mod stream;

mod shared;

mod sealed {
    pub trait Sealed {}
}
