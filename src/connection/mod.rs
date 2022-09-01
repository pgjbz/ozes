use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

pub struct OzesConnection {
    stream: RwLock<TcpStream>,
    socket_address: SocketAddr,
}

pub type IOResult = std::io::Result<()>;
pub type OzesConnections = RwLock<Vec<Arc<OzesConnection>>>;
pub type MessageQueue = Arc<RwLock<HashMap<String, OzesConnections>>>;

impl OzesConnection {
    pub fn new(stream: RwLock<TcpStream>, socket_address: SocketAddr) -> Self {
        Self {
            stream,
            socket_address,
        }
    }

    pub async fn send_message(&self, message: &str) -> IOResult {
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
