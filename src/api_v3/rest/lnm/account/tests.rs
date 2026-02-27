use std::{env, sync::Arc, time::Instant};

use dotenv::dotenv;

use crate::shared::rest::lnm::rate_limit::RateLimiter;

use super::super::super::config::RestClientConfig;
use super::*;

fn init_repository_from_env(rate_limiter: Option<RateLimiter>) -> LnmAccountRepository {
    dotenv().ok();

    let domain =
        env::var("LNM_API_DOMAIN").expect("LNM_API_DOMAIN environment variable must be set");
    let key = env::var("LNM_API_V3_KEY").expect("LNM_API_V3_KEY environment variable must be set");
    let secret =
        env::var("LNM_API_V3_SECRET").expect("LNM_API_V3_SECRET environment variable must be set");
    let passphrase = env::var("LNM_API_V3_PASSPHRASE")
        .expect("LNM_API_V3_PASSPHRASE environment variable must be set");

    let base = LnmRestBase::with_credentials(
        RestClientConfig::default().timeout(),
        domain,
        key,
        passphrase,
        SignatureGeneratorV3::new(secret),
        rate_limiter,
    )
    .expect("Can create `LnmApiBase`");

    LnmAccountRepository::new(base)
}

#[tokio::test]
#[ignore]
async fn test_api() {
    let repo = init_repository_from_env(None);

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

    let _ = time_test!("test_get_account", repo.get_account().await);
}

/// Fires 60 concurrent `get_account` requests through a rate-limited client.
///
/// The rate limiter paces authenticated requests at 5 req/s so all complete without 429's.
#[tokio::test]
#[ignore]
async fn test_v3_rate_limiter_prevents_auth_429() {
    let config = RestClientConfig::default();
    let rate_limiter = RateLimiter::new(
        config.rate_limit_auth_interval(),
        config.rate_limit_unauth_interval(),
    );
    let repo = Arc::new(init_repository_from_env(Some(rate_limiter)));

    let total_requests = 30;
    let mut handles = Vec::with_capacity(total_requests);

    let start = Instant::now();

    for i in 0..total_requests {
        let repo = repo.clone();
        handles.push(tokio::spawn(async move {
            let result = repo.get_account().await;
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

    println!("\n{total_requests} concurrent requests completed in {elapsed:?}");
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
