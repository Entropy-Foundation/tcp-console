use crate::console::Message;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{BytesCodec, Framed};
use tracing::debug;

/// Client for [Console].
pub struct Client {
    stream: Framed<TcpStream, BytesCodec>,
}

impl Client {
    pub async fn new<A: ToSocketAddrs>(address: A) -> anyhow::Result<Self> {
        // Connect to the TCP console server.
        let mut stream = Framed::new(TcpStream::connect(address).await?, BytesCodec::new());
        debug!("Connected to server");

        // Receive the welcome message.
        match stream.next().await {
            Some(Ok(_bytes)) => Ok(Client { stream }),
            Some(Err(e)) => Err(anyhow::Error::from(e)),
            None => Err(anyhow::Error::msg("Connection closed unexpectedly")),
        }
    }

    /// Sends a message to [Console] with any serializable payload.
    pub async fn send<S: Serialize, M: Serialize>(
        &mut self,
        service_id: S,
        message: &M,
    ) -> anyhow::Result<()> {
        let console_message = Message::new(service_id, message)?;

        // Create bytes to send.
        let bytes: Bytes = bcs::to_bytes(&console_message)?.into();

        // Send bytes.
        self.stream.send(bytes).await?;

        Ok(())
    }

    /// Sends a message to [Console] with any text.
    pub async fn weak_send(&mut self, message: &str) -> anyhow::Result<()> {
        let bytes: Bytes = message.as_bytes().to_vec().into();
        self.stream.send(bytes).await?;

        Ok(())
    }

    /// Receives a text message from [Console].
    pub async fn weak_read(&mut self) -> anyhow::Result<String> {
        let bytes = self
            .stream
            .next()
            .await
            .ok_or(anyhow::anyhow!("Connection closed unexpectedly"))??
            .freeze();

        Ok(String::from_utf8_lossy(bytes.as_ref()).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Subscription, SubscriptionError};
    use async_trait::async_trait;
    use bytes::Bytes;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::time::Duration;
    use tokio::time;
    use tracing::debug;
    use tracing_subscriber::EnvFilter;

    #[tokio::test]
    async fn ipv4_vs_ipv6() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env()) // Read filter level from RUST_LOG
            .with_target(true) // Include target in logs
            .try_init();

        for address in [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9090),
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 2020),
        ] {
            let mut console = crate::Builder::new()
                .bind_address(address)
                .welcome("Welcome to TCP console!")
                .subscribe(1u8, Test)?
                .accept_only_localhost()
                .build()?;

            console.spawn().await?;

            let mut client = crate::Client::new(address)
                .await
                .expect("Failed to create client");

            client
                .weak_send(&format!("Client connects to {address:?}"))
                .await
                .expect("Failed to send unknown message");

            time::sleep(Duration::from_millis(100)).await;
            console.stop();
            time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    struct Test;

    #[async_trait]
    impl Subscription for Test {
        async fn handle(&self, _message: Bytes) -> Result<Option<Bytes>, SubscriptionError> {
            debug!("`Test` receives a strongly typed message");
            Ok(None)
        }

        async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError> {
            debug!("`Test` receives a text message: {message}");
            Ok(None)
        }
    }
}
