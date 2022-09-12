use std::sync::{Arc, Mutex};

use bytes::Bytes;

use crate::{
    connection::{Connection, OzesConnection},
    parser::{self, Command},
};

use super::{
    error::{OzResult, OzesError},
    message_queue::OzesConnections,
};

pub struct Group {
    name: String,
    connections: OzesConnections,
    actual_con: Mutex<usize>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Self {
            name,
            connections: OzesConnections::default(),
            actual_con: Mutex::new(0),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn actual_con(&self) -> usize {
        *self.actual_con.lock().unwrap()
    }

    async fn pop_current_connection(&self) {
        let actual_con = self.actual_con();
        if let Some(connection) = self.connections.get(actual_con).await {
            log::info!("pop connection {}", connection.socket_address());
            self.connections.remove(actual_con).await;
        }
    }

    pub async fn push_connection(&self, connection: Arc<OzesConnection>) {
        self.connections.push(connection).await;
    }

    pub async fn send_message(&self, message: &Bytes) -> OzResult<()> {
        loop {
            if self.connections.is_empty().await {
                break;
            }
            if let Some(connection) = self.connections.get(self.actual_con()).await {
                match connection.send_message(message.clone()).await {
                    Ok(len) => match self.process_client_return(connection, len).await {
                        Err(error) if error.is_error(OzesError::WithouConnection) => {
                            self.pop_current_connection().await;
                            continue;
                        }
                        Err(error) if error.is_error(OzesError::InvalidLen(len)) => {
                            log::info!("invalid len: {} retry", len);
                            continue;
                        }
                        Err(error) if error.is_error(OzesError::UnknownError(String::new())) => {
                            //TODO: catch unknown error and handle correctly
                            log::info!("unknow error: {} retry", error);
                            continue;
                        }
                        Err(error) if error.is_error(OzesError::TimeOut) => {
                            self.next_connection();
                            continue;
                        }
                        Err(error) => {
                            //TODO: fix generic error catcher
                            log::info!("error {:?} retry", error);
                            continue;
                        }
                        Ok(_) => {
                            self.next_connection();
                            break;
                        }
                    },
                    Err(e) => {
                        log::error!(
                            "error on send message {} to currently connection {e}",
                            String::from_utf8_lossy(message)
                        );
                        self.pop_current_connection().await;
                        continue;
                    }
                }
            } else {
                self.reset_connection();
            }
        }
        Ok(())
    }

    fn reset_connection(&self) {
        *self.actual_con.lock().unwrap() = 0;
    }

    fn next_connection(&self) {
        *self.actual_con.lock().unwrap() += 1;
    }

    async fn process_client_return(
        &self,
        connection: Arc<OzesConnection>,
        msg_len: usize,
    ) -> OzResult<()> {
        let msg = connection.read_message().await;
        if let Ok(msg) = msg {
            let commands = parser::parse(msg);
            match commands {
                Ok(cmds) => {
                    match &cmds[..] {
                        [Command::Ok { len }] => {
                            if *len != msg_len {
                                return Err(OzesError::InvalidLen(*len));
                            }
                        }
                        [_] => {
                            connection
                                .send_error_message(Bytes::from_static(
                                    b"expected 'Ok' one command\n",
                                ))
                                .await?;
                        }
                        _ => {
                            connection
                                .send_error_message(Bytes::from_static(
                                    b"expected exactly one command\n",
                                ))
                                .await?;
                        }
                    }
                    Ok(())
                }
                Err(error) => {
                    println!("parse error: {}", error);
                    connection
                        .send_error_message(Bytes::copy_from_slice(error.to_string().as_bytes()))
                        .await?;
                    Ok(())
                }
            }
        } else {
            Err(OzesError::WithouConnection)
        }
    }
}
