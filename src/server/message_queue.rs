use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use super::{group::Group, OzesConnection, OzesResult};

pub type OzesConnections = RwLock<Vec<Arc<OzesConnection>>>;

#[derive(Default)]

pub struct MQueue {
    queues: RwLock<HashMap<String, InnerQueue>>,
}

struct InnerQueue {
    groups: RwLock<Vec<RwLock<Group>>>,
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
                if group_read.name() == group_name {
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
            log::info!("adding new group {group_name} to queue {queue_name}");
            let group = Group::new(group_name.to_string());
            log::info!("adding connection to new group {group_name}");
            group.push_connection(connection).await;
            let inner_queue = InnerQueue {
                groups: Default::default(),
            };
            log::info!("adding group {group_name} to queue {queue_name}");
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
            log::info!(
                "queue {queue_name} founded, iter over {} groupus",
                groups.len()
            );
            for group in groups.iter() {
                let group_write = group.read().await;
                group_write.send_message(message).await?;
            }
            Ok(())
        } else {
            drop(queues_read);
            self.add_queue(queue_name.to_string()).await;
            Ok(())
        }
    }
}
