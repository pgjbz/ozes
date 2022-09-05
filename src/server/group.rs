use std::sync::{Arc, Mutex};

use crate::{
    connection::OzesConnection,
    parser::{self, Command},
};

use super::{message_queue::OzesConnections, OzesResult};

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

    async fn pop_current_connection(&mut self) {
        let actual_con = *self.actual_con.lock().unwrap();
        self.connections.remove(actual_con);
    }

    pub async fn push_connection(&mut self, connection: Arc<OzesConnection>) {
        self.connections.push(connection);
    }

    pub async fn send_message(&mut self, message: &str) -> OzesResult {
        loop {
            if self.connections.is_empty() {
                break;
            }
            let actual_con = *self.actual_con.lock().unwrap();
            if let Some(connection) = self.connections.get(actual_con) {
                let connection = Arc::clone(connection);
                match connection.send_message(message).await {
                    Ok(_) => {
                        let msg = connection.read_message().await;
                        if let Ok(msg) = msg {
                            let commands = parser::parse(msg);
                            match commands {
                                Ok(cmds) => {
                                    if cmds.len() != 1 {
                                        if connection
                                            .send_error_message("expected exactly one command\n")
                                            .await
                                            .is_ok()
                                        {
                                            continue;
                                        }
                                        self.pop_current_connection().await;
                                    }
                                    if cmds[0] != Command::Ok {
                                        if connection
                                            .send_error_message("expected 'Ok' one command\n")
                                            .await
                                            .is_ok()
                                        {
                                            continue;
                                        }
                                        self.pop_current_connection().await;
                                        continue;
                                    }
                                    self.next_connection();
                                    break;
                                }
                                Err(error) => {
                                    if connection
                                        .send_error_message(&error.to_string())
                                        .await
                                        .is_err()
                                    {
                                        self.pop_current_connection().await;
                                    }
                                }
                            }
                        } else {
                            self.pop_current_connection().await;
                            continue;
                        }
                    }
                    Err(e) => {
                        log::error!("error on send message {message} to currently connection {e}");
                        self.pop_current_connection().await;
                        continue;
                    },
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

}
