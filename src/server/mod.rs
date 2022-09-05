use std::sync::Arc;

use tokio::{net::TcpListener, sync::Mutex};

use crate::{
    connection::OzesConnection,
    parser::{self, Command},
    server::message_queue::MQueue,
};

use self::error::OzResult;

pub(crate) mod error;
mod group;
mod message_queue;

pub async fn start_server(port: u16) -> OzResult<()> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}")).await?;
    log::info!("start listen on port {}", 7656);
    let queues = Arc::new(Mutex::new(MQueue::default()));
    loop {
        match listener.accept().await {
            Ok((stream, socket_address)) => {
                let queue = Arc::clone(&queues);
                tokio::task::spawn(handle_connection(
                    OzesConnection::new(Mutex::new(stream), socket_address),
                    queue,
                ));
            }
            Err(e) => log::error!("error on accept connection {}", e),
        }
    }
}

async fn handle_connection(
    ozes_connection: OzesConnection,
    message_queue: Arc<Mutex<MQueue>>,
) -> OzResult<()> {
    log::info!(
        "handle connection from address {}",
        ozes_connection.socket_address()
    );

    let connection = Arc::new(ozes_connection);
    let message = connection.read_message().await?;
    match parser::parse(message) {
        Ok(commands) => {
            for command in commands {
                let connection = Arc::clone(&connection);
                match command {
                    Command::Subscriber {
                        queue_name,
                        group_name,
                    } => {
                        message_queue
                            .lock()
                            .await
                            .add_listener(connection, &queue_name, &group_name)
                            .await;
                    }
                    Command::Publisher { queue_name } => {
                        tokio::task::spawn(handle_publisher(
                            connection,
                            Arc::clone(&message_queue),
                            queue_name,
                        ));
                    }
                    Command::Message { .. } => {
                        connection
                            .send_error_message("have to be a publisher before send a message")
                            .await?;
                    }
                    Command::Ok => {
                        connection
                            .send_error_message(
                                "ok command is able only when client receive a message",
                            )
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
            connection.send_error_message(&error.to_string()).await?;
            return Ok(());
        }
    }
    Ok(())
}

async fn handle_publisher(
    connection: Arc<OzesConnection>,
    message_queue: Arc<Mutex<MQueue>>,
    queue_name: String,
) -> OzResult<()> {
    if connection.send_message("Ok publisher").await.is_ok() {
        log::info!("handle publisher: {}", connection.socket_address());
        loop {
            let message = connection.read_message().await?;
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
                Err(error) => connection.send_error_message(&error.to_string()).await?,
            }
        }
    }
    Ok(())
}

async fn process_commands(
    commands: Vec<Command>,
    queue_name: String,
    publisher: Arc<OzesConnection>,
    message_queue: Arc<Mutex<MQueue>>,
) -> OzResult<()> {
    for command in commands {
        match command {
            Command::Message { message } => {
                process_message_command(
                    message,
                    &queue_name,
                    Arc::clone(&publisher),
                    Arc::clone(&message_queue),
                )
                .await?;
            }
            Command::Subscriber { .. } => {
                publisher
                    .send_error_message("you cannot subscribe to a queue when you are a publisher")
                    .await?
            }
            Command::Publisher { .. } => {
                publisher
                    .send_error_message("you cannot change queue to publish message")
                    .await?
            }
            Command::Ok => {
                publisher
                    .send_error_message(
                        "ok command is able only to subscribers when receive message",
                    )
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
    message_queue: Arc<Mutex<MQueue>>,
) -> OzResult<()> {
    log::info!("send {} to {} queue", message, queue_name);
    publisher.send_message("Ok message").await?;
    message_queue
        .lock()
        .await
        .send_message(&message, queue_name)
        .await?;
    Ok(())
}
