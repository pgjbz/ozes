use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::TcpListener, sync::RwLock};

use crate::{
    connection::OzesConnection,
    parser::{self, Command},
    server::message_queue::MQueue,
};

mod message_queue;
pub type OzesResult = std::io::Result<()>;

pub async fn start_server(port: u16) -> OzesResult {
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}")).await?;
    log::info!("start listen on port {}", 7656);
    let queues = Arc::new(MQueue::default());
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
    message_queue: Arc<MQueue>,
) -> OzesResult {
    log::info!(
        "handle connection from address {}",
        ozes_connection.socket_address()
    );

    let connection = Arc::new(ozes_connection);
    let message = read_to_string(Arc::clone(&connection)).await?;
    match parser::parse(message) {
        Ok(commands) => {
            for command in commands {
                let connection = Arc::clone(&connection);
                match command {
                    Command::Subscriber(queue_name, group) => {
                        message_queue
                            .add_listener(connection, &queue_name, &group)
                            .await;
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
                    Command::Ok => {
                        connection
                            .send_message("ok command is able only when client receive a message")
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

async fn handle_publisher(
    connection: Arc<OzesConnection>,
    message_queue: Arc<MQueue>,
    queue_name: String,
) -> OzesResult {
    if connection.send_message("Ok publisher").await.is_ok() {
        log::info!("handle publisher: {}", connection.socket_address());
        loop {
            let message = read_to_string(Arc::clone(&connection)).await?;
            let commands = parser::parse(message);
            match commands {
                Ok(commands) => {
                    process_commands(
                        commands,
                        queue_name.clone(),
                        Arc::clone(&connection),
                        Arc::clone(&message_queue),
                    )
                    .await?;
                    continue;
                }
                Err(error) => connection.send_message(&error.to_string()).await?,
            }
        }
    }
    Ok(())
}

async fn process_commands(
    commands: Vec<Command>,
    queue_name: String,
    publisher: Arc<OzesConnection>,
    message_queue: Arc<MQueue>,
) -> std::io::Result<()> {
    for command in commands {
        match command {
            Command::Message(message) => {
                process_message_command(
                    message,
                    &queue_name,
                    Arc::clone(&publisher),
                    Arc::clone(&message_queue),
                )
                .await?;
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
            Command::Ok => {
                publisher
                    .send_message("ok command is able only to subscribers when receive message")
                    .await?
            }
        }
    }
    Ok(())
}

async fn process_message_command(
    message: String,
    queue_name: &str,
    publisher: Arc<OzesConnection>,
    message_queue: Arc<MQueue>,
) -> std::io::Result<()> {
    log::info!("send {} to {} queue", message, queue_name);
    publisher.send_message("Ok message").await?;
    message_queue.send_message(&message, queue_name).await?;
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
