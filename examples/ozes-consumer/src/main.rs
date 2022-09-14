use ozes_connector::{consumer::ConsumerClient, errors::OzesConnectorError};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Foo {
    name: String,
    value: usize,
}

fn main() {
    let mut consumer = get_consumer();
    loop {
        match consumer.read_message() {
            Ok(msg) => {
                let foo = bincode::deserialize::<Foo>(&msg);
                println!("{foo:#?}");
            }
            Err(e) if e == OzesConnectorError::WithouConnection => break,
            Err(e) => eprintln!("error {}", e),
        }
    }
}

fn get_consumer() -> ConsumerClient {
    ConsumerClient::builder()
        .with_host("localhost")
        .on_port(7656)
        .on_queue("foo")
        .with_group("bar")
        .build()
        .unwrap()
}
