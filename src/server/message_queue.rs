use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::parser::{self, Command};

use super::{OzesConnection, OzesResult};

pub type OzesConnections = RwLock<Vec<Arc<OzesConnection>>>;
pub type MessageQueue = Arc<RwLock<HashMap<String, OzesConnections>>>;

#[derive(Default)]

pub struct MQueue {
    queues: RwLock<HashMap<String, InnerQueue>>,
}

pub struct Group {
    name: String,
    connections: OzesConnections,
    actual_con: usize,
}

struct InnerQueue {
    groups: RwLock<Vec<RwLock<Group>>>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Self {
            name,
            connections: OzesConnections::default(),
            actual_con: 0,
        }
    }

    async fn drop_actual_connection(&self) {
        self.connections.write().await.remove(self.actual_con);
    }

    async fn push_connection(&self, connection: Arc<OzesConnection>) {
        self.connections.write().await.push(connection);
    }

    pub async fn send_message(&mut self, message: &str) -> OzesResult {
        loop {
            let connections = self.connections.read().await;
            if connections.is_empty() {
                break;
            }
            if let Some(connection) = connections.get(self.actual_con) {
                if connection.send_message(message).await.is_ok() {
                    let msg = connection.read_message().await;
                    if let Some(msg) = msg {
                        let commands = parser::parse(msg);
                        let is_ok = commands.map_or_else(
                            |_| false,
                            |cmds| cmds.iter().all(|cmd| cmd == &Command::Ok),
                        );
                        if is_ok {
                            self.actual_con += 1;
                            break;
                        } else {
                            self.drop_actual_connection().await;
                            continue;
                        }
                    } else {
                        self.drop_actual_connection().await;
                        continue;
                    }
                } else {
                    drop(connections);
                    self.drop_actual_connection().await;
                    continue;
                }
            } else {
                self.actual_con = 0;
            }
        }
        Ok(())
    }
}

impl InnerQueue {
    async fn push_group(&self, group: RwLock<Group>) {
        let mut groups_write = self.groups.write().await;
        groups_write.push(group);
    }
}

impl MQueue {
    pub async fn add_queue(&self, queue_name: String) {
        let mut queues_write = self.queues.write().await;
        let inner_queue = InnerQueue {
            groups: Default::default(),
        };
        queues_write.insert(queue_name, inner_queue);
    }

    pub async fn add_listener(
        &self,
        connection: Arc<OzesConnection>,
        queue_name: &str,
        group_name: &str,
    ) {
        log::info!("add listener {} to queeue {queue_name} with group {group_name}", connection.socket_address());
        let queues_read = self.queues.read().await;
        let mut founded = false;
        if let Some(inner_queue) = queues_read.get(queue_name) {
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
            let queues_write = self.queues.read().await;
            let group = Group::new(group_name.to_string());
            group.push_connection(connection).await;
            self.add_queue(queue_name.to_string()).await;
            queues_write
                .get(queue_name)
                .unwrap()
                .push_group(RwLock::new(group))
                .await;
        }
    }

    pub async fn send_message(&self, message: &str, queue_name: &str) -> OzesResult {
        let queues_read = self.queues.read().await;
        if let Some(queue) = queues_read.get(queue_name) {
            for group in queue.groups.read().await.iter() {
                let mut group_write = group.write().await;
                group_write.send_message(message).await?
            }
            Ok(())
        } else {
            self.add_queue(queue_name.to_string()).await;
            Ok(())
        }
    }
}
