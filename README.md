# LN Markets SDK

A Rust SDK for interacting with [LN Markets](https://lnmarkets.com/).

> **Note:** This is an unofficial SDK. The currently supported API surfaces are REST API v3
> and Stream API v1. REST API v3 support is functional but not yet feature-complete; for
> implementation status, see the
> [REST v3 implementation docs](https://github.com/flemosr/lnm-sdk/blob/main/docs/rest-v3-implementation.md).

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
operations. All necessary models can be imported via the `models` module under the relevant domain version.

```rust,ignore
use lnm_sdk::rest::v3::{RestClient, RestClientConfig, models::*, error::*};
```

Each `RestClient` includes an internal FIFO rate limiter that automatically paces requests to stay
within the API's rate limits, with separate queues for authenticated and public endpoints. This
behavior is enabled by default and can be configured or disabled via `RestClientConfig`.

Stream API v1 types are available through `lnm_sdk::stream::v1`.

```rust,ignore
use lnm_sdk::stream::v1::{StreamClient, StreamClientConfig, models::*, error::*};
```

## Examples

Complete runnable examples are available in the
[`lnm-sdk/examples`](https://github.com/flemosr/lnm-sdk/tree/main/examples) directory. 

### REST API v3 - Public

```rust,no_run
use lnm_sdk::rest::v3::{RestClient, RestClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest = RestClient::new(RestClientConfig::default())?;

    // Get the futures ticker
    let _ticker = rest.futures_data.get_ticker().await?;

    // Get candles (OHLCs) history
    let _candles = rest
        .futures_data
        .get_candles(None, None, None, None, None)
        .await?;

    Ok(())
}
```

For more complete public REST examples, see the
[`rest_v3_public` example](https://github.com/flemosr/lnm-sdk/blob/main/examples/rest_v3_public.rs).

### REST API v3 - Authenticated

```rust,no_run
use std::env;

use lnm_sdk::rest::v3::{
    RestClient, RestClientConfig,
    models::{Leverage, OrderQuantity, TradeExecution, TradeSide, TradeSize},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rest = RestClient::with_credentials(
        RestClientConfig::default(),
        env::var("LNM_API_KEY")?,
        env::var("LNM_API_SECRET")?,
        env::var("LNM_API_PASSPHRASE")?,
    )?;

    // Get account information
    let _account = rest.account.get_account().await?;

    // Place a new isolated trade
    let trade = rest
        .futures_isolated
        .new_trade(
            TradeSide::Buy,
            TradeSize::from(OrderQuantity::try_from(1)?), // 1 USD
            Leverage::try_from(30)?,                      // 30x leverage
            TradeExecution::Market,
            None, // stoploss
            None, // takeprofit
            None, // client trade id
        )
        .await?;

    // Close the trade
    let _closed_trade = rest.futures_isolated.close_trade(trade.id()).await?;

    // Place a new cross order
    let _new_order = rest
        .futures_cross
        .place_order(
            TradeSide::Buy,
            OrderQuantity::try_from(1)?, // 1 USD
            TradeExecution::Market,
            None, // client order id
        )
        .await?;

    let _close_order = rest.futures_cross.close_position().await?;

    Ok(())
}
```

For more complete authenticated REST API examples, see the
[`rest_v3_auth` example](https://github.com/flemosr/lnm-sdk/blob/main/examples/rest_v3_auth.rs).

### Stream API v1 - Public Subscriptions

```rust,no_run
use lnm_sdk::stream::v1::{
    StreamClient, StreamClientConfig,
    models::{StreamTopic, StreamUpdate},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = StreamClient::new(StreamClientConfig::default());
    let stream = client.connect().await?;
    let mut updates = stream.receiver().await?;

    stream
        .subscribe(vec![StreamTopic::FuturesInverseBtcUsdLastPrice])
        .await?;

    if let Ok(update) = updates.recv().await {
        match update {
            StreamUpdate::ConnectionStatus(status) => println!("status: {status}"),
            update => println!("update for {:?}: {update:?}", update.topic()),
        }
    }

    stream.disconnect().await?;

    Ok(())
}
```

For a complete public Stream example, see the
[`stream_v1_public` example](https://github.com/flemosr/lnm-sdk/blob/main/examples/stream_v1_public.rs).

### Stream API v1 - Authenticated Private Subscriptions

```rust,no_run
use std::env;

use lnm_sdk::stream::v1::{StreamClient, StreamClientConfig, models::StreamTopic};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = StreamClient::new(StreamClientConfig::default());
    let stream = client.connect().await?;

    stream
        .authenticate(
            &env::var("LNM_API_KEY")?,
            &env::var("LNM_API_SECRET")?,
            &env::var("LNM_API_PASSPHRASE")?,
        )
        .await?;

    stream
        .subscribe(vec![StreamTopic::FuturesInverseBtcUsdCrossOrders])
        .await?;

    // ...
    
    stream.disconnect().await?;

    Ok(())
}
```

For a complete authenticated Stream example, see the
[`stream_v1_auth` example](https://github.com/flemosr/lnm-sdk/blob/main/examples/stream_v1_auth.rs).

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

+ [LN Markets REST API Documentation](https://docs.lnmarkets.com/en/api)
+ [LN Markets Stream API Documentation](https://docs.lnmarkets.com/en/stream)

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
