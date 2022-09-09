use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use tokio::sync::{Mutex, RwLock};

use super::{group::Group, OzResult, OzesConnection};

pub type OzesConnections = Vec<Arc<OzesConnection>>;

#[derive(Default)]
struct QueueWrapper(RwLock<HashMap<String, Arc<InnerQueue>>>);

#[derive(Default)]
pub struct MQueue {
    queues: QueueWrapper,
}

struct InnerQueue {
    groups: Mutex<Vec<Group>>,
}

impl InnerQueue {
    async fn push_group(&self, group: Group) {
        let mut groups_write = self.groups.lock().await;
        groups_write.push(group);
    }
}

impl MQueue {
    pub async fn add_listener(
        &self,
        connection: Arc<OzesConnection>,
        queue_name: &str,
        group_name: &str,
    ) {
        log::info!(
            "add listener {} to queue {queue_name} with group {group_name}",
            connection.socket_address()
        );

        //let mut queues_write = self.queues.write().await;
        let mut founded = false;
        if self.queues.contains_key(queue_name).await {
            let inner_queue = self.queues.get(queue_name).await.unwrap();
            let mut groups_write = inner_queue.groups.lock().await;
            for group in groups_write.iter_mut() {
                if group.name() == group_name {
                    group.push_connection(Arc::clone(&connection));
                    founded = true;
                    log::debug!("finish to add consumer to existent group");
                    break;
                }
            }
            if !founded {
                let mut group = Group::new(group_name.to_string());
                group.push_connection(Arc::clone(&connection));
                groups_write.push(group);
            }
            let _ = connection.ok_subscribed().await;

            log::debug!("finish to add consumer to existent grou in queue {queue_name}");
            return;
        }

        log::info!("adding new group {group_name} to queue {queue_name}");
        let mut group = Group::new(group_name.to_string());
        log::info!("adding connection to new group {group_name}");
        group.push_connection(Arc::clone(&connection));
        let inner_queue = InnerQueue {
            groups: Default::default(),
        };
        log::info!("adding group {group_name} to queue {queue_name}");
        self.queues.insert(queue_name, Arc::new(inner_queue)).await;

        if connection.ok_subscribed().await.is_err() {
            return;
        }
        self.queues
            .get(queue_name)
            .await
            .unwrap()
            .push_group(group)
            .await;

        log::info!("listener add to queue {queue_name} with group {group_name}");
    }

    pub async fn send_message(&mut self, message: Bytes, queue_name: Bytes) -> OzResult<()> {
        let queue_name = String::from_utf8_lossy(&queue_name[..]);
        log::info!("checking if {queue_name} exists",);
        if let Some(queue) = self.queues.get(&*queue_name).await {
            let mut groups = queue.groups.lock().await;
            log::info!(
                "queue {} founded, iter over {} groupus",
                queue_name,
                groups.len()
            );
            for group in groups.iter_mut() {
                group.send_message(message.clone()).await?;
            }
        } else {
            let inner_queue = InnerQueue {
                groups: Default::default(),
            };
            log::info!("adding new queue {queue_name}");
            self.queues
                .insert(&*queue_name, Arc::new(inner_queue))
                .await;
        }
        Ok(())
    }
}

impl QueueWrapper {
    async fn contains_key(&self, key: &str) -> bool {
        self.0.read().await.contains_key(key)
    }

    async fn get(&self, key: &str) -> Option<Arc<InnerQueue>> {
        self.0.read().await.get(key).map(Arc::clone)
    }

    async fn insert(&self, key: &str, value: Arc<InnerQueue>) {
        self.0.write().await.insert(key.to_string(), value);
    }
}
