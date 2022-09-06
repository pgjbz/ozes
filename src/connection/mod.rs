use std::net::SocketAddr;

use bytes::Bytes;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};

use crate::{
    server::error::{OzResult, OzesError},
    BUFFER_SIZE,
};

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

    pub async fn send_message(&self, message: Bytes) -> OzResult<()> {
        let mut stream = self.stream.write().await;
        stream.write_all(&message).await?;
        Ok(())
    }

    pub async fn send_error_message(&self, message: Bytes) -> OzResult<()> {
        let message_string = String::from_utf8_lossy(&message);
        let message = format!("error \"{message_string}\"");
        self.send_message(Bytes::copy_from_slice(message.as_bytes()))
            .await
    }

    pub async fn ok_subscribed(&self) -> OzResult<()> {
        self.send_message(Bytes::from_static(b"ok subscribed"))
            .await
    }

    pub async fn ok_publisher(&self) -> OzResult<()> {
        self.send_message(Bytes::from_static(b"ok publisher")).await
    }

    pub async fn ok_message(&self) -> OzResult<()> {
        self.send_message(Bytes::from_static(b"ok message")).await
    }

    pub async fn read_message(&self) -> OzResult<Bytes> {
        let stream = self.stream.read().await;
        let mut buffer = vec![0; BUFFER_SIZE];

        let size = loop {
            stream.readable().await?;
            match stream.try_read(&mut buffer) {
                Ok(size) => {
                    if size == 0 {
                        log::info!("connection from {} is closed", self.socket_address());
                        return Err(OzesError::WithouConnection);
                    }
                    buffer.truncate(size);
                    break size;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(error) => {
                    log::error!(
                        "error on read message from connection {}: {}",
                        self.socket_address(),
                        error
                    );
                    return Err(error)?;
                }
            }
        };
        if size > BUFFER_SIZE {
            return Err(OzesError::ToLongMessage);
        }
        Ok(Bytes::copy_from_slice(&buffer[0..size]))
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn stream(&self) -> &RwLock<TcpStream> {
        &self.stream
    }
}
