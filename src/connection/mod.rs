use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::{server::OzesResult, BUFFER_SIZE};

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

    pub async fn read_message(&self) -> std::io::Result<String> {
        let mut stream = self.stream().lock().await;
        let mut buffer = [0; BUFFER_SIZE];
        let size = match stream.read(&mut buffer).await {
            Ok(size) => {
                if size == 0 {
                    log::info!("connection from {} is closed", self.socket_address());
                    return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
                }
                size
            }
            Err(error) => {
                log::error!(
                    "error on read message from connection {}: {}",
                    self.socket_address(),
                    error
                );
                return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
            }
        };
        let mut vec = Vec::with_capacity(size);
        vec.extend_from_slice(&buffer[0..size]);
        let message = String::from_utf8(vec).unwrap();
        Ok(message)
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn stream(&self) -> &Mutex<TcpStream> {
        &self.stream
    }
}
