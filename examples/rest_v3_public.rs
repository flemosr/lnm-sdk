//! Example demonstrating how to use the REST API v3 public client.

use dotenvy::dotenv;
use lnm_sdk::rest::v3::{RestClient, RestClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let rest = RestClient::new(RestClientConfig::default())?;

    // Utilities endpoints

    // Ping
    rest.utilities.ping().await?;
    println!("Pinged server successfully");

    // Get the server time
    let server_time = rest.utilities.time().await?;
    println!("Got server time: {}", server_time.time());

    // Futures Data endpoints

    // Get funding settlement history
    let funding_settlements = rest
        .futures_data
        .get_funding_settlements(None, None, None, None)
        .await?;
    println!(
        "Got funding settlements. Len: {}",
        funding_settlements.data().len()
    );

    // Get the futures ticker (index and last price)
    let ticker = rest.futures_data.get_ticker().await?;
    println!(
        "Got futures ticker. Index: {}, last price: {}",
        ticker.index(),
        ticker.last_price()
    );

    // Get candles (OHLCs) history
    let candles = rest
        .futures_data
        .get_candles(None, None, None, None, None)
        .await?;
    println!("Got candles. Len: {}", candles.data().len());

    Ok(())
}
