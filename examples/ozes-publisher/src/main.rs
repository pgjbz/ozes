use std::time::Duration;

use ozes_connector::{
    errors::OzesConnectorError,
    publisher::{OzPublisher, Publisher},
};
use rand::random;
use serde::Serialize;

#[derive(Serialize)]
struct Foo {
    name: String,
    value: usize,
}

fn main() {
    let mut publisher = get_connection();
    let mut value = 0usize;
    loop {
        let name = format!("Name {}", random::<usize>());
        let foo = Foo { name: name, value };
        let foo_serialized = bincode::serialize(&foo).unwrap();
        match publisher.send_message(&foo_serialized) {
            Ok(_) => println!("message sended"),
            Err(error)
                if std::mem::discriminant(&OzesConnectorError::WithouConnection)
                    == std::mem::discriminant(&error) =>
            {
                publisher = get_connection();
                continue;
            }
            Err(error) => eprintln!("error on send message {}", error),
        }
        value += 1;
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn get_connection() -> OzPublisher {
    Publisher::builder()
        .with_host("localhost")
        .with_port(7656)
        .on_queue("foo")
        .build()
        .unwrap()
}
