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
                if connection.send_message(message).await.is_ok() {
                    let msg = connection.read_message().await;
                    if let Ok(msg) = msg {
                        let commands = parser::parse(msg);
                        match commands {
                            Ok(cmds) => {
                                if cmds.len() != 1 {
                                    if connection
                                        .send_message("expected exactly one command\n")
                                        .await
                                        .is_ok()
                                    {
                                        continue;
                                    }
                                    self.connections.remove(actual_con);
                                }
                                if cmds[0] != Command::Ok {
                                    if connection
                                        .send_message("expected 'Ok' one command\n")
                                        .await
                                        .is_ok()
                                    {
                                        continue;
                                    }
                                    self.connections.remove(actual_con);
                                    continue;
                                }
                                self.next_connection();
                                break;
                            }
                            Err(error) => {
                                if connection.send_message(&error.to_string()).await.is_err() {
                                    self.connections.remove(actual_con);
                                }
                            }
                        }
                    } else {
                        self.connections.remove(actual_con);
                        continue;
                    }
                } else {
                    self.connections.remove(actual_con);
                    continue;
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
