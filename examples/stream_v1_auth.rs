//! Example demonstrating how to authenticate and subscribe to private Stream v1 topics.

use std::{env, error::Error, io};

use dotenv::dotenv;
use lnm_sdk::stream::v1::{
    StreamClient, StreamClientConfig, StreamConnectionStatus,
    models::{StreamTopic, StreamUpdate},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let key = env::var("LNM_API_KEY").expect("LNM_API_KEY must be set");
    let secret = env::var("LNM_API_SECRET").expect("LNM_API_SECRET must be set");
    let passphrase = env::var("LNM_API_PASSPHRASE").expect("LNM_API_PASSPHRASE must be set");

    let client = StreamClient::new(StreamClientConfig::default());
    let stream = client.connect().await?;

    let auth = stream.authenticate(&key, &secret, &passphrase).await?;
    if !auth.authenticated() {
        return Err(io::Error::other("Stream authentication was rejected").into());
    }

    println!("Authenticated Stream session.");
    println!("Permissions: {:?}", auth.permissions());

    let session = stream.whoami().await?;
    println!("Authenticated as user {}", session.user_id());

    let mut updates = stream.receiver().await?;
    stream
        .subscribe(vec![
            StreamTopic::FuturesInverseBtcUsdIsolatedTrades,
            StreamTopic::FuturesInverseBtcUsdCrossOrders,
            StreamTopic::FuturesInverseBtcUsdCrossPosition,
            StreamTopic::WalletDeposit,
            StreamTopic::WalletWithdrawal,
        ])
        .await?;

    println!("Subscribed to private Stream topics.");

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
                    println!("Private update for {topic}: {update:?}");
                } else {
                    println!("Stream update: {update:?}");
                }

                messages += 1;
            }
        }

        if messages >= max_messages {
            println!("Received {max_messages} private data messages, disconnecting...");
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
