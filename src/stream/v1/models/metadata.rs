use serde::Deserialize;

/// Rate-limit metadata returned by the Stream API on JSON-RPC responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct StreamRateLimit {
    remaining: u64,
    limit: u64,
}

impl StreamRateLimit {
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }
}

/// Server-side timing and rate-limit metadata returned with JSON-RPC responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamResponseMetadata {
    us_in: Option<u64>,
    us_out: Option<u64>,
    us_diff: Option<u64>,
    rate_limit: Option<StreamRateLimit>,
}

impl StreamResponseMetadata {
    pub(in crate::stream::v1) fn new(
        us_in: Option<u64>,
        us_out: Option<u64>,
        us_diff: Option<u64>,
        rate_limit: Option<StreamRateLimit>,
    ) -> Self {
        Self {
            us_in,
            us_out,
            us_diff,
            rate_limit,
        }
    }

    pub fn us_in(&self) -> Option<u64> {
        self.us_in
    }

    pub fn us_out(&self) -> Option<u64> {
        self.us_out
    }

    pub fn us_diff(&self) -> Option<u64> {
        self.us_diff
    }

    pub fn rate_limit(&self) -> Option<StreamRateLimit> {
        self.rate_limit
    }
}
