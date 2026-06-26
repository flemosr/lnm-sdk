//! Example demonstrating how to subscribe to public Stream v1 market-data topics.

use std::error::Error;

use dotenv::dotenv;
use lnm_sdk::stream::v1::{
    StreamClient, StreamClientConfig, StreamConnectionStatus,
    models::{OhlcRange, StreamTopic, StreamUpdate},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let client = StreamClient::new(StreamClientConfig::default());
    let stream = client.connect().await?;

    let hello = stream
        .hello("lnm-sdk-stream-public-example", env!("CARGO_PKG_VERSION"))
        .await?;
    println!("Connected to Stream API version {}", hello.version());

    let server_time = stream.time().await?;
    println!("Server time: {server_time}");

    let mut updates = stream.receiver().await?;
    stream
        .subscribe(vec![
            StreamTopic::FuturesInverseBtcUsdLastPrice,
            StreamTopic::FuturesInverseBtcUsdIndex,
            StreamTopic::FuturesInverseBtcUsdOhlc(OhlcRange::OneMinute),
        ])
        .await?;

    println!("Subscribed to public Stream topics.");

    let max_messages = 100;
    let mut messages = 0;

    while let Ok(update) = updates.recv().await {
        match update {
            StreamUpdate::ConnectionStatus(status) => {
                println!("Connection status: {status}");
                if matches!(
                    status,
                    StreamConnectionStatus::Disconnected | StreamConnectionStatus::Failed(_)
                ) {
                    break;
                }
            }
            update => {
                if let Some(topic) = update.topic() {
                    println!("Update for {topic}: {update:?}");
                } else {
                    println!("Stream update: {update:?}");
                }

                messages += 1;
            }
        }

        if messages >= max_messages {
            println!("Received {max_messages} data messages, disconnecting...");
            break;
        }
    }

    if stream.is_connected().await {
        let unsubscribed = stream.unsubscribe_all().await?;
        println!("Unsubscribed from {} topics.", unsubscribed.len());
        stream.disconnect().await?;
    }

    Ok(())
}
