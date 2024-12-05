use crate::console::Message;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};
use tracing::debug;

/// Client for [Console].
pub struct Client {
    stream: Framed<TcpStream, BytesCodec>,
}

impl Client {
    pub async fn new(address: SocketAddr) -> anyhow::Result<Self> {
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
