use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::RwLock;

use crate::parser::{self, Command};

use super::{OzesConnection, OzesResult};

pub type OzesConnections = RwLock<Vec<Arc<OzesConnection>>>;

#[derive(Default)]

pub struct MQueue {
    queues: RwLock<HashMap<String, InnerQueue>>,
}

pub struct Group {
    name: String,
    connections: OzesConnections,
    actual_con: Mutex<usize>,
}

struct InnerQueue {
    groups: RwLock<Vec<RwLock<Group>>>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Self {
            name,
            connections: OzesConnections::default(),
            actual_con: Mutex::new(0),
        }
    }

    async fn push_connection(&self, connection: Arc<OzesConnection>) {
        self.connections.write().await.push(connection);
    }

    pub async fn send_message(&self, message: &str) -> OzesResult {
        loop {
            let mut connections = self.connections.write().await;
            if connections.is_empty() {
                break;
            }
            let actual_con = *self.actual_con.lock().unwrap();
            if let Some(connection) = connections.get(actual_con) {
                let connection = Arc::clone(connection);
                if connection.send_message(message).await.is_ok() {
                    let msg = connection.read_message().await;
                    if let Some(msg) = msg {
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
                                    connections.remove(actual_con);
                                }
                                if cmds[0] != Command::Ok {
                                    if connection
                                        .send_message("expected 'Ok' one command\n")
                                        .await
                                        .is_ok()
                                    {
                                        continue;
                                    }
                                    connections.remove(actual_con);
                                    continue;
                                }
                                self.next_connection();
                                break;
                            }
                            Err(error) => {
                                if connection.send_message(&error.to_string()).await.is_err() {
                                    connections.remove(actual_con);
                                }
                            }
                        }
                    } else {
                        connections.remove(actual_con);
                        continue;
                    }
                } else {
                    connections.remove(actual_con);
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

impl InnerQueue {
    async fn push_group(&self, group: RwLock<Group>) {
        let mut groups_write = self.groups.write().await;
        groups_write.push(group);
    }
}

impl MQueue {
    #[inline(always)]
    pub async fn add_queue(&self, queue_name: String) {
        let mut queues_write = self.queues.write().await;
        let inner_queue = InnerQueue {
            groups: Default::default(),
        };
        queues_write.insert(queue_name, inner_queue);
        drop(queues_write)
    }

    pub async fn add_listener(
        &self,
        connection: Arc<OzesConnection>,
        queue_name: &str,
        group_name: &str,
    ) {
        log::info!(
            "add listener {} to queeue {queue_name} with group {group_name}",
            connection.socket_address()
        );

        let mut queues_write = self.queues.write().await;
        let mut founded = false;
        if let Some(inner_queue) = queues_write.get(queue_name) {
            let groups_read = inner_queue.groups.read().await;
            for group in groups_read.iter() {
                let group_read = group.read().await;
                if group_read.name == group_name {
                    group_read.push_connection(Arc::clone(&connection)).await;
                    founded = true;
                    break;
                }
            }
            if !founded {
                let mut groups_write = inner_queue.groups.write().await;
                let group = Group::new(group_name.to_string());
                group.push_connection(connection).await;
                groups_write.push(RwLock::new(group))
            }
        } else {
            let group = Group::new(group_name.to_string());
            group.push_connection(connection).await;
            let inner_queue = InnerQueue {
                groups: Default::default(),
            };
            queues_write.insert(queue_name.to_string(), inner_queue);
            queues_write
                .get(queue_name)
                .unwrap()
                .push_group(RwLock::new(group))
                .await;
        }
        log::info!("listener add to queue {queue_name} with group {group_name}");
    }

    pub async fn send_message(&self, message: &str, queue_name: &str) -> OzesResult {
        log::info!("checking if {queue_name} exists");
        let queues_read = self.queues.read().await;
        if let Some(queue) = queues_read.get(queue_name) {
            let groups = queue.groups.read().await;
            log::info!("queue {queue_name} founded, iter over {} groupus", groups.len());
            for group in groups.iter() {
                let group_write = group.read().await;
                group_write.send_message(message.clone()).await?;
            }
            Ok(())
        } else {
            drop(queues_read);
            self.add_queue(queue_name.to_string()).await;
            Ok(())
        }
    }
}
