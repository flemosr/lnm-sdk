use std::sync::Arc;

use async_trait::async_trait;
use hyper::Method;

use crate::shared::rest::{error::Result, lnm::base::LnmRestBase};

use super::{
    super::{models::user::User, repositories::UserRepository},
    path::RestPathV2,
    signature::SignatureGeneratorV2,
};

pub(in crate::api_v2) struct LnmUserRepository {
    base: Arc<LnmRestBase<SignatureGeneratorV2>>,
}

impl LnmUserRepository {
    pub fn new(base: Arc<LnmRestBase<SignatureGeneratorV2>>) -> Self {
        Self { base }
    }
}

impl crate::sealed::Sealed for LnmUserRepository {}

#[async_trait]
impl UserRepository for LnmUserRepository {
    async fn get_user(&self) -> Result<User> {
        self.base
            .make_request_without_params(Method::GET, RestPathV2::UserGetUser, true)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::{env, sync::Arc, time::Instant};

    use dotenv::dotenv;

    use crate::shared::rest::lnm::rate_limit::RateLimiter;

    use super::super::super::config::RestClientConfig;
    use super::*;

    fn init_repository_from_env(rate_limiter: Option<RateLimiter>) -> LnmUserRepository {
        dotenv().ok();

        let domain =
            env::var("LNM_API_DOMAIN").expect("LNM_API_DOMAIN environment variable must be set");
        let key = env::var("LNM_API_V2_KEY").expect("LNM_API_KEY environment variable must be set");
        let secret =
            env::var("LNM_API_V2_SECRET").expect("LNM_API_SECRET environment variable must be set");
        let passphrase = env::var("LNM_API_V2_PASSPHRASE")
            .expect("LNM_API_V2_PASSPHRASE environment variable must be set");

        let base = LnmRestBase::with_credentials(
            RestClientConfig::default().timeout(),
            domain,
            key,
            passphrase,
            SignatureGeneratorV2::new(secret),
            rate_limiter,
        )
        .expect("must create `LnmApiBase`");

        LnmUserRepository::new(base)
    }

    async fn test_get_user(repo: &LnmUserRepository) -> User {
        repo.get_user().await.expect("must get user")
    }

    #[tokio::test]
    #[ignore]
    async fn test_api() {
        let repo = init_repository_from_env(None);

        let _ = test_get_user(&repo).await;
    }

    // Fires 65 concurrent `get_user` requests through a rate-limited client.
    // NOTE: As of Feb 2026, LNM seems to tolerate up to 120 req/min for this endpoint.
    //
    // The rate limiter paces authenticated requests at 60 req/min, in line with official doc limit.
    #[tokio::test]
    #[ignore]
    async fn test_v2_rate_limiter_prevents_auth_429() {
        let config = RestClientConfig::default();
        let rate_limiter = RateLimiter::from(&config);
        let repo = Arc::new(init_repository_from_env(Some(rate_limiter)));

        let total_requests = 65;
        let mut handles = Vec::with_capacity(total_requests);

        let start = Instant::now();

        for i in 0..total_requests {
            let repo = repo.clone();
            handles.push(tokio::spawn(async move {
                let result = repo.get_user().await;
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
}
