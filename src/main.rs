use std::sync::Arc;

use fast_log::Config;
use ozes::{
    connection::{IOResult, MessageQueue, OzesConnection, OzesConnections},
    lexer::Lexer,
    parser::{Command, Parser},
};
use tokio::{io::AsyncReadExt, net::TcpListener, sync::RwLock};

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
                    OzesConnection::new(RwLock::new(stream), socket_address),
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
        ozes_connection.socket_address()
    );

    let connection = Arc::new(ozes_connection);
    let message = read_to_string(Arc::clone(&connection)).await?;
    let mut parser = Parser::new(Lexer::new(message));
    match parser.parse_commands() {
        Ok(commands) => {
            for command in commands {
                let connection = Arc::clone(&connection);
                match command {
                    Command::Subscriber(queue_name, _) => {
                        add_listener(Arc::clone(&message_queue), &queue_name, connection).await;
                    }
                    Command::Publisher(queue_name) => {
                        tokio::task::spawn(handle_publisher(
                            connection,
                            Arc::clone(&message_queue),
                            queue_name,
                        ));
                    }
                    Command::Message(_) => {
                        connection
                            .send_message("have to be a publisher before send a message")
                            .await?;
                    }
                }
            }
        }
        Err(error) => {
            log::error!(
                "error with connection {}: {error}",
                connection.socket_address()
            );
            connection.send_message(&error.to_string()).await?;
            return Ok(());
        }
    }
    Ok(())
}

async fn add_listener(
    message_queue: MessageQueue,
    queue_name: &str,
    connection: Arc<OzesConnection>,
) {
    let mut queue = message_queue.write().await;
    log::info!("add listener to queue {queue_name}");
    if connection.send_message("Ok subscriber").await.is_ok() {
        match queue.get(queue_name) {
            Some(queue) => {
                queue.write().await.push(connection);
            }
            None => {
                queue.insert(queue_name.to_owned(), RwLock::new(vec![connection]));
            }
        }
    } else {
        log::error!(
            "error to add listener {:?} to queue {queue_name}",
            connection.socket_address()
        )
    }
}

async fn handle_publisher(
    connection: Arc<OzesConnection>,
    message_queue: MessageQueue,
    queue_name: String,
) -> IOResult {
    if connection.send_message("Ok publisher").await.is_ok() {
        loop {
            let message = read_to_string(Arc::clone(&connection)).await?;
            let mut parser = Parser::new(Lexer::new(message));
            let commands = parser.parse_commands();
            match commands {
                Ok(commands) => {
                    let queue = message_queue.read().await;
                    match queue.get(&queue_name) {
                        Some(subs) => {
                            process_commands(commands, subs, &queue_name, &connection).await?;
                        }
                        None => {
                            let mut queue = message_queue.write().await;
                            queue.insert(queue_name.clone(), RwLock::new(vec![]));
                            continue;
                        }
                    }
                }
                Err(error) => connection.send_message(&error.to_string()).await?,
            }
        }
    }
    Ok(())
}

async fn process_commands(
    commands: Vec<Command>,
    subs: &OzesConnections,
    queue_name: &String,
    publisher: &Arc<OzesConnection>,
) -> std::io::Result<()> {
    for command in commands {
        match command {
            Command::Message(message) => {
                process_message_command(subs, message, queue_name, publisher).await?;
            }
            Command::Subscriber(_, _) => {
                publisher
                    .send_message("you cannot subscribe to a queue when you are a publisher")
                    .await?
            }
            Command::Publisher(_) => {
                publisher
                    .send_message("you cannot change queue to publish message")
                    .await?
            }
        }
    }
    Ok(())
}

async fn process_message_command(
    subs: &OzesConnections,
    message: String,
    queue_name: &String,
    publisher: &Arc<OzesConnection>,
) -> std::io::Result<()> {
    let subs_read = subs.read().await;
    let mut to_remove = Vec::with_capacity(subs_read.len());
    log::info!("send {} to {} queue", message, queue_name);
    publisher.send_message("Ok message").await?;
    for sub in subs_read.iter() {
        if sub.send_message(&message).await.is_err() {
            log::error!("address {} close connection", sub.socket_address());
            to_remove.push(sub.socket_address());
        }
    }
    for rmv in to_remove {
        let mut subs_write = subs.write().await;
        subs_write.retain(move |s| s.socket_address() == rmv);
    }
    Ok(())
}

async fn read_to_string(ozes_connection: Arc<OzesConnection>) -> std::io::Result<String> {
    let mut stream = ozes_connection.stream().write().await;
    let mut buffer = [0; 1024];
    let size = match stream.read(&mut buffer).await {
        Ok(size) => {
            if size == 0 {
                log::info!(
                    "connection from {} is closed",
                    ozes_connection.socket_address()
                );
                return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
            }
            size
        }
        Err(error) => {
            log::error!(
                "error on read message from connection {}: {}",
                ozes_connection.socket_address(),
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
