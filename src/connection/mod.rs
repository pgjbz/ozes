use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::server::OzesResult;

pub struct OzesConnection {
    stream: Mutex<TcpStream>,
    socket_address: SocketAddr,
}

impl OzesConnection {
    pub fn new(stream: Mutex<TcpStream>, socket_address: SocketAddr) -> Self {
        Self {
            stream,
            socket_address,
        }
    }

    pub async fn send_message(&self, message: &str) -> OzesResult {
        let mut stream = self.stream.lock().await;
        stream.write_all(message.as_bytes()).await?;
        Ok(())
    }

    pub async fn send_error_message(&self, message: &str) -> OzesResult {
        self.send_message(&format!("error \"{message}\"")).await?;
        Ok(())
    }

    pub async fn read_message(&self) -> Option<String> {
        let mut buffer = [0; 1024];
        let mut stream = self.stream.lock().await;
        let size = match stream.read(&mut buffer).await {
            Ok(size) => {
                if size == 0 {
                    return None;
                }
                size
            }
            Err(_) => return None,
        };
        let mut vec = Vec::with_capacity(size);
        vec.extend_from_slice(&buffer[0..size]);
        match String::from_utf8(vec) {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn stream(&self) -> &Mutex<TcpStream> {
        &self.stream
    }
}
