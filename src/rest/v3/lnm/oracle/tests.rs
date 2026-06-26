use std::time::Instant;

use dotenv::dotenv;

use super::super::super::config::RestClientConfig;
use super::*;

fn init_repository_from_env() -> LnmOracleRepository {
    dotenv().ok();

    let config = RestClientConfig::default();

    let base = LnmRestBase::new(config.timeout(), config.endpoint().to_string(), None)
        .expect("Can create `LnmApiBase`");

    LnmOracleRepository::new(base)
}

async fn test_get_index(repo: &LnmOracleRepository, limit: Option<NonZeroU64>) {
    let _ = repo
        .get_index(None, None, limit, None)
        .await
        .expect("must get index page");
}

async fn test_get_last_price(repo: &LnmOracleRepository, limit: Option<NonZeroU64>) {
    let _ = repo
        .get_last_price(None, None, limit, None)
        .await
        .expect("must get last price page");
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

    let limit = Some(NonZeroU64::new(10).unwrap());

    time_test!("test_get_index", test_get_index(&repo, limit).await);

    time_test!(
        "test_get_last_price",
        test_get_last_price(&repo, limit).await
    );
}
