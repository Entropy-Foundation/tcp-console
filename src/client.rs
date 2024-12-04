use crate::console::Message;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

/// Client for [Console].
pub struct Client {
    address: SocketAddr,
}

impl Client {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    /// Sends a message to [Console] with any serializable payload.
    pub async fn send<S: Serialize, M: Serialize>(
        &self,
        service_id: S,
        message: &M,
    ) -> anyhow::Result<()> {
        let console_message = Message::new(service_id, message)?;

        // Connect to the TCP console server.
        let mut stream = TcpStream::connect(self.address).await?;
        debug!("Connected to server");

        // Receive the welcome message
        let mut welcome = vec![0; 512];
        let _ = stream.read(&mut welcome).await?;

        // Create bytes to send.
        let bytes = bcs::to_bytes(&console_message)?;

        // Send bytes.
        stream.write_all(&bytes).await?;

        Ok(())
    }
}
