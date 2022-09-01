use std::net::SocketAddr;

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::server::OzesResult;

pub struct OzesConnection {
    stream: RwLock<TcpStream>,
    socket_address: SocketAddr,
}

impl OzesConnection {
    pub fn new(stream: RwLock<TcpStream>, socket_address: SocketAddr) -> Self {
        Self {
            stream,
            socket_address,
        }
    }

    pub async fn send_message(&self, message: &str) -> OzesResult {
        let mut stream = self.stream.write().await;
        stream.write_all(message.as_bytes()).await?;
        Ok(())
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn stream(&self) -> &RwLock<TcpStream> {
        &self.stream
    }
}
