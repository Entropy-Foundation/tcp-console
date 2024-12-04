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
        .build()?;

    console.spawn().await?;

    tokio::spawn(async move {
        let client = console::Client::new(
            format!("127.0.0.1:{port}")
                .parse()
                .expect("Failed to parse socket address"),
        );

        client
            .send(Services::Logger, &"Typed LoggerMessage")
            .await
            .expect("Failed to send logger message");

        time::sleep(Duration::from_secs(3)).await;

        client
            .send(Services::Exec, &"Typed ExecMessage")
            .await
            .expect("Failed to send exec message");

        time::sleep(Duration::from_secs(3)).await;

        client
            .send(Services::Unknown, &"Typed UnknownMessage")
            .await
            .expect("Failed to send unknown message");
    });

    signal::ctrl_c().await?;

    console.stop();

    // Let console to actually stop.
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
    async fn handle(&self, message: Bytes) -> Result<Option<String>, SubscriptionError> {
        let message =
            bcs::from_bytes::<String>(message.as_ref()).expect("Must deserialize message");
        debug!("[Logger] request to process a strongly typed message: {message}");
        Ok(None)
    }

    async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError> {
        debug!("[Logger] request to process a text message: {message}");
        Ok(None)
    }
}

struct Exec;

#[async_trait]
impl Subscription for Exec {
    async fn handle(&self, message: Bytes) -> Result<Option<String>, SubscriptionError> {
        let message =
            bcs::from_bytes::<String>(message.as_ref()).expect("Must deserialize message");
        debug!("[Exec] request to process a strongly typed message: {message}");
        Ok(None)
    }

    async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError> {
        debug!("[Exec] request to process a text message: {message}");
        Ok(None)
    }
}

#[derive(Debug)]
/// A structure representing the status of the system.
struct Status {
    connections: u32,
    health: String,
}

#[async_trait]
impl Subscription for Status {
    async fn handle(&self, message: Bytes) -> Result<Option<String>, SubscriptionError> {
        debug!("[Status] request to process a strongly typed message");

        Ok(Some(format!("{self:#?}")))
    }

    async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError> {
        debug!("[Status] request to process a text message: {message}");

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
