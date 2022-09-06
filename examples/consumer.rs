use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
};

use ozes::BUFFER_SIZE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let group = env::var("GROUP_NAME").unwrap_or("bar".to_owned());
    let queue_name = env::var("QUEUE_NAME").unwrap_or("foo".to_owned());
    let mut socket_stream = TcpStream::connect("localhost:7656")?;
    socket_stream.write_all(format!("subscribe {queue_name} with group {group}").as_bytes())?;
    loop {
        let mut buffer = [0; BUFFER_SIZE];
        let size = socket_stream.read(&mut buffer)?;
        let mut vec = Vec::with_capacity(size);
        vec.extend_from_slice(&buffer[0..size]);
        let message = String::from_utf8(vec).unwrap();
        println!("{}", message);
        socket_stream.write_all(b"ok")?;
    }
}
