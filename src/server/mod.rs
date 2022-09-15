use std::sync::Arc;

use bytes::Bytes;
use ozes_parser::parser::{self, Command};
use tokio::net::TcpListener;

use crate::{
    connection::{Connection, OzesConnection},
    server::message_queue::MQueue,
    BASE_MESSAGE_LEN,
};

use self::error::{OzResult, OzesError};

pub(crate) mod error;
mod group;
mod message_queue;

type Queues = Arc<MQueue>;

pub async fn start_server(port: u16) -> OzResult<()> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}")).await?;
    log::info!("start listen on port {}", 7656);
    let queues = Arc::new(MQueue::default());
    tokio::spawn(process_queues(Arc::clone(&queues)));
    loop {
        match listener.accept().await {
            Ok((stream, socket_address)) => {
                let queue = Arc::clone(&queues);
                tokio::task::spawn(handle_connection(
                    OzesConnection::new(stream, socket_address),
                    queue,
                ));
            }
            Err(e) => log::error!("error on accept connection {}", e),
        }
    }
}

async fn process_queues(queues: Arc<MQueue>) {
    loop {
        let keys = queues.get_keys().await;
        for key in keys {
            if let Some(queue) = queues.get(&key).await {
                queue.process_message().await;
            }
        }
    }
}

async fn handle_connection(ozes_connection: OzesConnection, message_queue: Queues) -> OzResult<()> {
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
                            .add_listener(
                                connection,
                                &String::from_utf8_lossy(&queue_name),
                                &String::from_utf8_lossy(&group_name),
                            )
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
                            .send_error_message(Bytes::from_static(
                                b"have to be a publisher before send a message",
                            ))
                            .await?;
                    }
                    Command::Ok { .. } => {
                        connection
                            .send_error_message(Bytes::from_static(
                                b"ok command is able only when client receive a message",
                            ))
                            .await?;
                    }
                    Command::Error { .. } => {
                        connection
                            .send_error_message(Bytes::from_static(
                                b"cannot send error on first message",
                            ))
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
            connection
                .send_error_message(Bytes::copy_from_slice(error.to_string().as_bytes()))
                .await?;
            return Ok(());
        }
    }
    Ok(())
}

async fn handle_publisher(
    connection: Arc<OzesConnection>,
    message_queue: Queues,
    queue_name: Bytes,
) -> OzResult<()> {
    if connection.ok_publisher().await.is_ok() {
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
                }
                Err(error) => {
                    connection
                        .send_error_message(Bytes::copy_from_slice(error.to_string().as_bytes()))
                        .await?;
                }
            }
        }
    }
    Ok(())
}

async fn process_commands(
    commands: Vec<Command>,
    queue_name: Bytes,
    publisher: Arc<OzesConnection>,
    message_queue: Queues,
) -> OzResult<()> {
    for command in commands {
        match command {
            Command::Message { message, len } => {
                let message_len = message.len();
                let message_len_size = crate::number_len(message_len);
                let total_len = message_len + message_len_size + BASE_MESSAGE_LEN;

                if total_len != len {
                    log::error!(
                        "error on process message {:?} invalid len[{}] is provided",
                        message,
                        len
                    );
                    Err(OzesError::InvalidLen(total_len))?
                }

                process_message_command(
                    message,
                    queue_name.clone(),
                    Arc::clone(&publisher),
                    Arc::clone(&message_queue),
                )
                .await?;
            }
            Command::Subscriber { .. } => {
                publisher
                    .send_error_message(Bytes::from_static(b"cannot subscribe when is a publisher"))
                    .await?;
            }
            Command::Publisher { .. } => {
                publisher
                    .send_error_message(Bytes::from_static(
                        b"cannot change queue when already is a publisher",
                    ))
                    .await?;
            }
            Command::Ok { .. } => {
                publisher
                    .send_error_message(Bytes::from_static(
                        b"ok command is only able to subscribers when receive message",
                    ))
                    .await?;
            }
            Command::Error { message } => Err(OzesError::UnknownError(format!(
                "error with connection {} dropping with message {:?}",
                publisher.socket_address(),
                message
            )))?,
        }
    }
    Ok(())
}

async fn process_message_command(
    message: Bytes,
    queue_name: Bytes,
    publisher: Arc<OzesConnection>,
    message_queue: Queues,
) -> OzResult<()> {
    log::info!(
        "send {} bytes to {} queue",
        message.len(),
        String::from_utf8_lossy(&queue_name)
    );
    publisher.ok_message().await?;
    message_queue.push_message(message, queue_name).await?;
    Ok(())
}
