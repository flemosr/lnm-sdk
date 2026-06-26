use std::time::Instant;

use dotenvy::dotenv;

use super::super::super::config::RestClientConfig;
use super::*;

fn init_repository_from_env() -> LnmUtilitiesRepository {
    dotenv().ok();

    let config = RestClientConfig::default();

    let base = LnmRestBase::new(config.timeout(), config.endpoint().to_string(), None)
        .expect("Can create `LnmApiBase`");

    LnmUtilitiesRepository::new(base)
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

    time_test!("test_ping", repo.ping().await).unwrap();

    let _ = time_test!("test_time", repo.time().await).unwrap();
}
