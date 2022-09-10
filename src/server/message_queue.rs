use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use bytes::Bytes;
use tokio::sync::{Mutex, RwLock};

use super::{group::Group, OzResult, OzesConnection};

#[derive(Default)]
pub struct OzesConnections(RwLock<Vec<Arc<OzesConnection>>>);

#[derive(Default)]
struct QueueWrapper(RwLock<HashMap<String, Arc<InnerQueue>>>);

#[derive(Default)]
pub struct MQueue {
    queues: QueueWrapper,
}

#[derive(Default)]
pub(super) struct InnerQueue {
    groups: Mutex<Vec<Group>>,
    messages: RwLock<VecDeque<Bytes>>,
}

impl InnerQueue {
    fn with_groups(groups: Vec<Group>) -> Self {
        Self {
            groups: Mutex::new(groups),
            messages: RwLock::default(),
        }
    }

    async fn get_message(&self) -> Option<Bytes> {
        self.messages.write().await.pop_front()
    }

    pub(crate) async fn process_message(&self) {
        if let Some(message) = self.get_message().await {
            let mut groups = self.groups.lock().await;
            for group in groups.iter_mut() {
                let _ = group.send_message(&message).await;
            }
        }
    }

    async fn push_message(&self, message: Bytes) {
        self.messages.write().await.push_back(message);
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

        let mut founded = false;
        if self.queues.contains_key(queue_name).await {
            let inner_queue = self.queues.get(queue_name).await.unwrap();
            let mut groups = inner_queue.groups.lock().await;
            for group in groups.iter_mut() {
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
                groups.push(group);
            }
            let _ = connection.ok_subscribed().await;

            log::debug!("finish to add consumer to existent grou in queue {queue_name}");
            return;
        }

        log::info!("adding new group {group_name} to queue {queue_name}");
        let mut group = Group::new(group_name.to_string());
        log::info!("adding connection to new group {group_name}");
        group.push_connection(Arc::clone(&connection)).await;
        log::info!("adding group {group_name} to queue {queue_name}");
        let inner_queue = InnerQueue::with_groups(vec![group]);

        if connection.ok_subscribed().await.is_ok() {
            self.queues.insert(queue_name, Arc::new(inner_queue)).await;
            log::info!("listener add to queue {queue_name} with group {group_name}");
        }
    }

    pub async fn send_message(&mut self, message: Bytes, queue_name: Bytes) -> OzResult<()> {
        let queue_name = String::from_utf8_lossy(&queue_name[..]);
        log::info!("checking if {queue_name} exists",);
        if let Some(queue) = self.queues.get(&*queue_name).await {
            queue.push_message(message).await;
            log::info!("queue {} founded, push message to queue", queue_name);
        } else {
            let inner_queue = InnerQueue::default();
            log::info!("adding new queue {queue_name}");
            inner_queue.push_message(message).await;
            self.queues
                .insert(&*queue_name, Arc::new(inner_queue))
                .await;
        }
        Ok(())
    }

    pub(super) async fn get_keys(&self) -> Vec<String> {
        self.queues.get_keys().await
    }

    pub(super) async fn get(&self, key: &str) -> Option<Arc<InnerQueue>> {
        self.queues.get(key).await
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

    pub async fn get_keys(&self) -> Vec<String> {
        let queues = self.0.read().await;
        queues.keys().cloned().collect()
    }
}

impl OzesConnections {
    pub(crate) async fn get(&self, idx: usize) -> Option<Arc<OzesConnection>> {
        self.0.read().await.get(idx).map(Arc::clone)
    }

    pub(crate) async fn remove(&self, idx: usize) {
        self.0.write().await.remove(idx);
    }

    pub(crate) async fn push(&self, connection: Arc<OzesConnection>) {
        self.0.write().await.push(connection)
    }

    pub(crate) async fn is_empty(&self) -> bool {
        self.0.read().await.is_empty()
    }
}
