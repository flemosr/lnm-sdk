use std::{env, sync::Arc, time::Instant};

use dotenv::dotenv;

use crate::shared::rest::lnm::rate_limit::RateLimiter;

use super::super::super::config::RestClientConfig;
use super::*;

fn init_repository_from_env_with_rate_limiter(
    rate_limiter: Option<RateLimiter>,
) -> LnmFuturesDataRepository {
    dotenv().ok();

    let domain =
        env::var("LNM_API_DOMAIN").expect("LNM_API_DOMAIN environment variable must be set");

    let base = LnmRestBase::new(RestClientConfig::default().timeout(), domain, rate_limiter)
        .expect("must create `LnmApiBase`");

    LnmFuturesDataRepository::new(base)
}

fn init_repository_from_env() -> LnmFuturesDataRepository {
    dotenv().ok();

    let domain =
        env::var("LNM_API_DOMAIN").expect("LNM_API_DOMAIN environment variable must be set");

    let base = LnmRestBase::new(RestClientConfig::default().timeout(), domain, None)
        .expect("must create `LnmApiBase`");

    LnmFuturesDataRepository::new(base)
}

async fn test_get_funding_settlements(repo: &LnmFuturesDataRepository) {
    let _ = repo
        .get_funding_settlements(None, None, None, None)
        .await
        .expect("must get funding settlements");
}

async fn test_ticker(repo: &LnmFuturesDataRepository) {
    let ticker = repo.get_ticker().await.expect("must get ticker");

    assert!(!ticker.prices().is_empty());
}

async fn test_get_max_candles(repo: &LnmFuturesDataRepository) {
    let limit = 1000.try_into().unwrap();
    let _ = repo
        .get_candles(None, None, Some(limit), Some(OhlcRange::OneMinute), None)
        .await
        .expect("must get candles");
}

async fn test_get_last_candle(repo: &LnmFuturesDataRepository) {
    let limit = 1.try_into().unwrap();
    let _ = repo
        .get_candles(None, None, Some(limit), Some(OhlcRange::OneMinute), None)
        .await
        .expect("must get candles");
}

#[tokio::test]
#[ignore]
async fn test_api() {
    let repo = init_repository_from_env();

    macro_rules! time_test {
        ($test_name: expr, $test_block: expr) => {{
            println!("\nStarting test: {}", $test_name);
            let start = Instant::now();
            let result = $test_block;
            let elapsed = start.elapsed();
            println!("Test '{}' took: {:?}", $test_name, elapsed);
            result
        }};
    }

    // Start tests

    time_test!("test_ticker", test_ticker(&repo).await);

    time_test!(
        "test_get_funding_settlements",
        test_get_funding_settlements(&repo).await
    );

    time_test!("test_get_max_candles", test_get_max_candles(&repo).await);

    time_test!("test_get_last_candle", test_get_last_candle(&repo).await);
}

// Fires 15 concurrent `get_ticker` requests through a rate-limited client.
//
// The rate limiter paces unauthenticated requests at 1 req/s so all complete without any 429's.
#[tokio::test]
#[ignore]
async fn test_v3_rate_limiter_prevents_unauth_429() {
    let config = RestClientConfig::default();
    let rate_limiter = RateLimiter::from(&config);
    let repo = Arc::new(init_repository_from_env_with_rate_limiter(Some(
        rate_limiter,
    )));

    let total_requests: usize = 15;
    let mut handles = Vec::with_capacity(total_requests);

    let start = Instant::now();

    for i in 0..total_requests {
        let repo = repo.clone();
        handles.push(tokio::spawn(async move {
            let result = repo.get_ticker().await;
            (i, result)
        }));
    }

    let mut successes = 0;
    let mut failures = Vec::new();

    for handle in handles {
        let (i, result) = handle.await.expect("task must not panic");
        match result {
            Ok(_) => successes += 1,
            Err(e) => failures.push((i, e)),
        }
    }

    let elapsed = start.elapsed();

    println!("\n{total_requests} concurrent unauth requests completed in {elapsed:?}");
    println!("  successes: {successes}");
    println!("  failures:  {}", failures.len());

    for (i, err) in &failures {
        println!("  request #{i} failed: {err}");
    }

    assert!(
        failures.is_empty(),
        "rate limiter should prevent all 429 errors, but {}/{total_requests} requests failed",
        failures.len(),
    );
}
