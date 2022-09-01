use std::{sync::Arc, collections::HashMap};

use bytes::Bytes;
use tokio::sync::RwLock;

use super::OzesConnection;

pub type OzesConnections = RwLock<Vec<Arc<OzesConnection>>>;
pub type MessageQueue = Arc<RwLock<HashMap<String, OzesConnections>>>;

struct Group {
    name: String,
    connections: OzesConnections,
    idx: usize,
    actual_con: usize,
}

impl Group {
    fn new(name: String) -> Self {
        Self {
            name,
            connections: OzesConnections::default(),
            idx: 0,
            actual_con: 0,
        }
    }

    fn send_message(&self) {
        todo!()
    }
}

struct MQueue {
    queues: RwLock<HashMap<String, InnerQueue>>, 
}

struct InnerQueue {
    subscribers: Vec<OzesConnection>,
    messages: Vec<Bytes>
}