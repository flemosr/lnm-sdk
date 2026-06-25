# Examples

Example applications demonstrating different ways to use the `lnm-sdk` crate.

## Quick Templates

Direct source code links for quick reference:

| Category | Raw Source |
|----------|------------|
| **API v3 - Public Endpoints** | [v3_rest_public.rs](https://raw.githubusercontent.com/flemosr/lnm-sdk/refs/heads/main/examples/v3_rest_public.rs) |
| **API v3 - Authenticated Endpoints** | [v3_rest_auth.rs](https://raw.githubusercontent.com/flemosr/lnm-sdk/refs/heads/main/examples/v3_rest_auth.rs) |
| **Stream API v1 - Public Subscriptions** | [stream_v1_public.rs](https://raw.githubusercontent.com/flemosr/lnm-sdk/refs/heads/main/examples/stream_v1_public.rs) |
| **Stream API v1 - Authenticated Subscriptions** | [stream_v1_auth.rs](https://raw.githubusercontent.com/flemosr/lnm-sdk/refs/heads/main/examples/stream_v1_auth.rs) |

## Prerequisites

REST examples require:
- `LNM_API_DOMAIN` - The LN Markets API domain

Stream examples use `wss://stream.lnmarkets.com/v1` via `StreamClientConfig::default()`.

API v3 authenticated examples (`v3_rest_auth` and `stream_v1_auth`) require:
- `LNM_API_V3_KEY` - Your API v3 key
- `LNM_API_V3_SECRET` - Your API v3 secret
- `LNM_API_V3_PASSPHRASE` - Your API v3 passphrase

These environment variables should be set, or a `.env` file should be added in the project root.
A [`.env.template`](https://github.com/flemosr/lnm-sdk/blob/main/.env.template) file is available.

## API v3

The following examples demonstrate the current API v3 REST interface.

### v3_rest_public

Demonstrates how to use the API v3 REST public client to fetch market data, including utilities
endpoints, futures data, and oracle data.

**Usage:**
```bash
cargo run --example v3_rest_public
```

### v3_rest_auth

Demonstrates how to use the API v3 REST authenticated client to manage both isolated and
cross-margin futures positions, including placing orders, managing margin, and closing positions.

**Usage:**
```bash
cargo run --example v3_rest_auth
```

## Stream API v1

The following examples demonstrate the current Stream API v1 WebSocket interface.

### stream_v1_public

Demonstrates how to connect to Stream v1 and subscribe to public market-data topics.

**Usage:**
```bash
cargo run --example stream_v1_public
```

### stream_v1_auth

Demonstrates how to authenticate a Stream v1 session with REST v3 API credentials and subscribe to
private account topics.

**Usage:**
```bash
cargo run --example stream_v1_auth
```

