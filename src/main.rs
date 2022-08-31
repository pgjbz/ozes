use std::{cmp::Ordering, collections::HashMap, net::SocketAddr, sync::Arc};

use fast_log::Config;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};

type IOResult = std::io::Result<()>;
type MessageQueue = Arc<Mutex<HashMap<String, RwLock<Vec<Arc<OzesConnection>>>>>>;

struct OzesConnection {
    stream: RwLock<TcpStream>,
    socket_address: SocketAddr,
}

#[tokio::main]
async fn main() -> IOResult {
    fast_log::init(Config::new().console()).unwrap();
    let listener = TcpListener::bind("0.0.0.0:7656").await?;
    log::info!("start listen on port {}", 7656);
    let queues = MessageQueue::default();
    loop {
        match listener.accept().await {
            Ok((stream, socket_address)) => {
                let queue = Arc::clone(&queues);
                tokio::task::spawn(handle_connection(
                    OzesConnection {
                        stream: RwLock::new(stream),
                        socket_address,
                    },
                    queue,
                ));
            }
            Err(e) => log::error!("error on accept connection {}", e),
        }
    }
}

async fn handle_connection(
    ozes_connection: OzesConnection,
    message_queue: MessageQueue,
) -> IOResult {
    log::info!(
        "handle connection from address {}",
        ozes_connection.socket_address
    );

    let connection = Arc::new(ozes_connection);
    let message = read_to_string(Arc::clone(&connection)).await?;

    if let Some(queue_name) = message.strip_prefix("SUBSCRIBE: ") {
        add_listener(message_queue, queue_name, connection).await;
    } else if let Some(queue_name) = message.strip_prefix("PUBLISH: ") {
        handle_publisher(connection, message_queue, queue_name).await?
    }
    Ok(())
}

async fn add_listener(
    message_queue: MessageQueue,
    queue_name: &str,
    connection: Arc<OzesConnection>,
) {
    let mut queue = message_queue.lock().await;
    log::info!("add listener to queue {}", queue_name);
    match queue.get(queue_name) {
        Some(queue) => {
            queue.write().await.push(connection);
        }
        None => {
            queue.insert(queue_name.to_owned(), RwLock::new(vec![connection]));
        }
    }
}

async fn handle_publisher(
    connection: Arc<OzesConnection>,
    message_queue: MessageQueue,
    queue_name: &str,
) -> IOResult {
    loop {
        let msg = read_to_string(Arc::clone(&connection)).await?;
        let mut queue = message_queue.lock().await;
        match queue.get(queue_name) {
            Some(subs) => {
                let subs_read = subs.read().await;
                let mut to_remove = Vec::with_capacity(subs_read.len());
                for sub in subs_read.iter() {
                    let mut stream = sub.stream.write().await;
                    log::info!("send {} to {} queue", msg, queue_name);
                    if stream.write_all(msg.as_bytes()).await.is_err() {
                        log::error!("address {} close connection", sub.socket_address);
                        to_remove.push(&sub.socket_address);
                    }
                }
                for r in to_remove {
                    let mut subs_write = subs.write().await;
                    subs_write.retain(move |s| s.socket_address.cmp(r) != Ordering::Equal);
                }
            }
            None => {
                queue.insert(queue_name.to_string(), RwLock::new(vec![]));
                continue;
            }
        }
    }
}

async fn read_to_string(ozes_connection: Arc<OzesConnection>) -> std::io::Result<String> {
    let mut stream = ozes_connection.stream.write().await;
    let mut buffer = [0; 1024];
    let size = match stream.read(&mut buffer).await {
        Ok(size) => {
            if size == 0 {
                log::info!(
                    "connection from {} is closed",
                    ozes_connection.socket_address
                );
                return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
            }
            size
        }
        Err(error) => {
            log::error!(
                "error on read message from connection {}: {}",
                ozes_connection.socket_address,
                error
            );
            return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
        }
    };
    let mut vec = Vec::with_capacity(size);
    vec.extend_from_slice(&buffer[0..size]);
    let message = String::from_utf8(vec).unwrap();
    Ok(message)
}
