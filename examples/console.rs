use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tcp_console as console;
use tcp_console::{Subscription, SubscriptionError};
use tokio::{signal, time};
use tracing::debug;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let port = 3838;

    let console = console::Builder::new()
        .port(port)
        .welcome("Welcome to TCP console!")
        .subscribe(Services::Logger, Logger)?
        .subscribe(Services::Exec, Exec)?
        .subscribe(
            Services::Status,
            Status {
                connections: 11,
                health: "Operational".to_string(),
            },
        )?
        .accept_only_localhost()
        .build()?;

    console.spawn().await?;

    // Send a few strongly-typed messages:
    // - a text message to [Services::Status]
    // - a message for [Services::Logger]
    // - a message for [Services::Exec]
    // - a message for [Services::Unknown],
    //      no subscription is present for this service,
    //      [Console] will emit a warning,
    tokio::spawn(async move {
        let mut client = console::Client::new(
            format!("127.0.0.1:{port}")
                .parse()
                .expect("Failed to parse socket address"),
        )
        .await
        .expect("Failed to create client");

        client
            .weak_send("status")
            .await
            .expect("Failed to send unknown message");

        let status = client.weak_read().await.expect("Failed to read");
        debug!("{status:?}");

        time::sleep(Duration::from_secs(2)).await;

        client
            .send(Services::Logger, &"Typed LoggerMessage")
            .await
            .expect("Failed to send logger message");

        time::sleep(Duration::from_secs(2)).await;

        client
            .send(Services::Exec, &"Typed ExecMessage")
            .await
            .expect("Failed to send exec message");

        time::sleep(Duration::from_secs(2)).await;

        client
            .send(Services::Unknown, &"Typed UnknownMessage")
            .await
            .expect("Failed to send unknown message");
    });

    signal::ctrl_c().await?;

    console.stop();

    // Let the console to actually stop.
    time::sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Services {
    Logger,
    Exec,
    Status,
    Unknown,
}

struct Logger;

#[async_trait]
impl Subscription for Logger {
    async fn handle(&self, message: Bytes) -> Result<Option<Bytes>, SubscriptionError> {
        let message =
            bcs::from_bytes::<String>(message.as_ref()).expect("Must deserialize message");
        debug!("[Logger] request to process a strongly typed message: `{message}`");
        Ok(None)
    }

    async fn weak_handle(&self, _message: &str) -> Result<Option<String>, SubscriptionError> {
        Ok(None)
    }
}

struct Exec;

#[async_trait]
impl Subscription for Exec {
    async fn handle(&self, message: Bytes) -> Result<Option<Bytes>, SubscriptionError> {
        let message =
            bcs::from_bytes::<String>(message.as_ref()).expect("Must deserialize message");
        debug!("[Exec] request to process a strongly typed message: `{message}`");
        Ok(None)
    }

    async fn weak_handle(&self, _message: &str) -> Result<Option<String>, SubscriptionError> {
        Ok(None)
    }
}

#[derive(Debug)]
#[allow(dead_code)] // This struct is for demonstration purposes only.
/// A structure representing the status of some system.
struct Status {
    connections: u32,
    health: String,
}

#[async_trait]
impl Subscription for Status {
    async fn handle(&self, _message: Bytes) -> Result<Option<Bytes>, SubscriptionError> {
        debug!("[Status] request to process a strongly typed message");

        Ok(None)
    }

    async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError> {
        Ok(if message == "status" {
            Some(format!("{self:#?}"))
        } else {
            None
        })
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()) // Read filter level from RUST_LOG
        .with_target(true) // Include target in logs
        .init();
}
