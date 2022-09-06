use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use tokio::sync::Mutex;

use super::{group::Group, OzResult, OzesConnection};

pub type OzesConnections = Vec<Arc<OzesConnection>>;

#[derive(Default)]

pub struct MQueue {
    queues: HashMap<String, Arc<InnerQueue>>,
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
        &mut self,
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
        if self.queues.contains_key(queue_name) {
            let inner_queue = Arc::clone(self.queues.get(queue_name).unwrap());
            let mut groups_write = inner_queue.groups.lock().await;
            for group in groups_write.iter_mut() {
                if group.name() == group_name {
                    group.push_connection(Arc::clone(&connection)).await;
                    founded = true;
                    log::debug!("finish to add consumer to existent group");
                    break;
                }
            }
            if !founded {
                let mut group = Group::new(group_name.to_string());
                group.push_connection(Arc::clone(&connection)).await;
                groups_write.push(group);
            }
            let _ = connection.ok_subscribed().await;

            log::debug!("finish to add consumer to existent grou in queue {queue_name}");
            return;
        }

        log::info!("adding new group {group_name} to queue {queue_name}");
        let mut group = Group::new(group_name.to_string());
        log::info!("adding connection to new group {group_name}");
        group.push_connection(Arc::clone(&connection)).await;
        let inner_queue = InnerQueue {
            groups: Default::default(),
        };
        log::info!("adding group {group_name} to queue {queue_name}");
        self.queues
            .insert(queue_name.to_string(), Arc::new(inner_queue));

        if connection.ok_subscribed().await.is_err() {
            return;
        }
        self.queues.get(queue_name).unwrap().push_group(group).await;

        log::info!("listener add to queue {queue_name} with group {group_name}");
    }

    pub async fn send_message(&mut self, message: Bytes, queue_name: Bytes) -> OzResult<()> {
        let queue_name = String::from_utf8_lossy(&queue_name[..]);
        log::info!("checking if {queue_name} exists",);
        if let Some(queue) = self.queues.get(&*queue_name) {
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
                .insert(queue_name.to_string(), Arc::new(inner_queue));
        }
        Ok(())
    }
}
