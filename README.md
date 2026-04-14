# LN Markets SDK

A Rust SDK for interacting with [LN Markets](https://lnmarkets.com/).

> **Note:** This is an unofficial SDK. API v3 support is functional but not yet feature-complete. 
> For implementation status, see the
> [API v3 implementation docs](https://github.com/flemosr/lnm-sdk/blob/main/docs/api-v3-implementation.md).
>
> LN Markets disabled API v2 on Mar 31 2026. An implementation (REST + WebSocket) is currently
> retained in the `api_v2` module for reference only.

[![Crates.io Badge](https://img.shields.io/crates/v/lnm-sdk)](https://crates.io/crates/lnm-sdk)
[![Documentation Badge](https://docs.rs/lnm-sdk/badge.svg)](https://docs.rs/lnm-sdk/latest/lnm_sdk/)
[![License Badge](https://img.shields.io/crates/l/lnm-sdk)](https://github.com/flemosr/lnm-sdk/blob/main/LICENSE)

[Repository](https://github.com/flemosr/lnm-sdk) |
[Examples](https://github.com/flemosr/lnm-sdk/tree/main/examples) |
[Documentation](https://docs.rs/lnm-sdk/latest/lnm_sdk/)

## Getting Started

### Rust Version

This project's MSRV is `1.88`.

### Dependencies

```toml
[dependencies]
lnm-sdk = "<lnm-sdk-version>"
```

## Usage

This SDK provides strong type-safety with validated types for all parameters used in trade 
operations. All necessary models can be imported via the `models` mod of the API version in question.

```rust,ignore
use lnm_sdk::api_v3::{RestClient, RestClientConfig, models::*, error::*};
```

Each `RestClient` includes an internal FIFO rate limiter that automatically paces requests to stay
within the API's rate limits, with separate queues for authenticated and public endpoints. This
behavior is enabled by default and can be configured or disabled via `RestClientConfig`.

## Examples

Complete runnable examples are available in the
[`lnm-sdk/examples`](https://github.com/flemosr/lnm-sdk/tree/main/examples) directory. 

### REST API v3 - Public

```rust,ignore
use lnm_sdk::api_v3::{RestClient, RestClientConfig};

//...

let domain = env::var("LNM_API_DOMAIN").expect("LNM_API_DOMAIN must be set");

let rest = RestClient::new(RestClientConfig::default(), &domain)?;
    
// Get the futures ticker
let _ticker = rest.futures_data.get_ticker().await?;

// Get candles (OHLCs) history
let _candles = rest
    .futures_data
    .get_candles(None, None, None, None, None)
    .await?;
```

For more complete public API examples, see the
[`v3_rest_public` example](https://github.com/flemosr/lnm-sdk/blob/main/examples/v3_rest_public.rs).

### REST API v3 - Authenticated

```rust,ignore
use lnm_sdk::api_v3::{
    RestClient, RestClientConfig,
    models::{Leverage, Quantity, TradeExecution, TradeSide, TradeSize},
};

// ...

let domain = env::var("LNM_API_DOMAIN").expect("LNM_API_DOMAIN must be set");
let key = env::var("LNM_API_V3_KEY").expect("LNM_API_V3_KEY must be set");
let secret = env::var("LNM_API_V3_SECRET").expect("LNM_API_V3_SECRET must be set");
let passphrase = env::var("LNM_API_V3_PASSPHRASE").expect("LNM_API_V3_PASSPHRASE must be set");

let rest = RestClient::with_credentials(
    RestClientConfig::default(),
    &domain,
    key,
    secret,
    passphrase,
)?;
    
// Get account information
let _account = rest.account.get_account().await?;

// Place a new isolated trade
let trade = rest
    .futures_isolated
    .new_trade(
        TradeSide::Buy,
        TradeSize::from(Quantity::try_from(1)?), // 1 USD
        Leverage::try_from(30)?,                 // 30x leverage
        TradeExecution::Market,
        None, // stoploss
        None, // takeprofit
        None, // client trade id
    )
    .await?;

// Close the trade
let _closed_trade = rest
    .futures_isolated
    .close_trade(trade.id())
    .await?;
  
// Place a new cross order
let _new_order = rest
    .futures_cross
    .place_order(
        TradeSide::Buy,
        Quantity::try_from(1)?, // 1 USD
        TradeExecution::Market,
        None, // client order id
    )
    .await?;

let _close_order = rest.futures_cross.close_position().await?;
```

For more complete authenticated REST API examples, see the
[`v3_rest_auth` example](https://github.com/flemosr/lnm-sdk/blob/main/examples/v3_rest_auth.rs).

## Testing

Some tests require environment variables and are ignored by default. Moreover, said tests must be
run sequentially as they depend on exchange state. The full test suite can be executed by setting
the `LNM_API_*` variables or adding a `.env` file to the project root (a
[`.env.template`](https://github.com/flemosr/lnm-sdk/blob/main/.env.template) file is available),
and then running:

```bash
cargo test -- --include-ignored --test-threads=1
```

## API Reference

+ [LN Markets API v3 Documentation](https://api.lnmarkets.com/v3/)

## Development History

This crate was originally developed as part of the
[`quantoxide`](https://github.com/flemosr/quantoxide) repository before being extracted into a
standalone repository on 2025-12-26 at `quantoxide` commit
[`0d78ee08`](https://github.com/flemosr/quantoxide/commit/0d78ee08). The full development history
was preserved.

## License

This project is licensed under the
[Apache License (Version 2.0)](https://github.com/flemosr/lnm-sdk/blob/main/LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion by
you, shall be licensed as Apache-2.0, without any additional terms or conditions.
