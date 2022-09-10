use std::net::SocketAddr;

use bytes::Bytes;
use tokio::{
    net::TcpStream,
    time::{self, Duration},
};

use crate::{
    server::error::{OzResult, OzesError},
    BUFFER_SIZE,
};

pub struct OzesConnection {
    stream: TcpStream,
    socket_address: SocketAddr,
}

impl OzesConnection {
    pub fn new(stream: TcpStream, socket_address: SocketAddr) -> Self {
        Self {
            stream,
            socket_address,
        }
    }

    async fn send(&self, message: Bytes) -> OzResult<usize> {
        let sized = loop {
            self.stream.writable().await?;
            match self.stream.try_write(&message) {
                Ok(n) => break n,
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
        Ok(sized)
    }

    pub async fn send_message(&self, message: Bytes) -> OzResult<()> {
        tokio::select! {
            res = self.send(message) => {let _ = res?;}
            _ = time::sleep(Duration::from_millis(500)) => {
                log::error!("write message time out");
                return Err(OzesError::TimeOut);
            }
        }
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

    async fn read(&self) -> OzResult<Bytes> {
        loop {
            self.stream.readable().await?;
            let mut buffer = vec![0; BUFFER_SIZE];
            match self.stream.try_read(&mut buffer) {
                Ok(size) => {
                    if size == 0 {
                        log::info!("connection from {} is closed", self.socket_address());
                        return Err(OzesError::WithouConnection);
                    }
                    if size > BUFFER_SIZE {
                        return Err(OzesError::ToLongMessage);
                    }
                    buffer.truncate(size);
                    let bytes = Bytes::copy_from_slice(&buffer[..]);
                    break Ok(bytes);
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
        }
    }

    pub async fn read_message(&self) -> OzResult<Bytes> {
        let result = tokio::select! {
            res = self.read() => {res?}
            _ = time::sleep(Duration::from_millis(500)) => {
                log::error!("write message time out");
                return Err(OzesError::TimeOut);
            }
        };
        Ok(result)
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}
