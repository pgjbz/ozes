use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket_stream = TcpStream::connect("localhost:7656")?;
    let mut buffer = [0; 1024];
    socket_stream.write_all(b"PUBLISHER foo")?;
    let size = socket_stream.read(&mut buffer)?;
    let mut vec = Vec::with_capacity(size);
    vec.extend_from_slice(&buffer[0..size]);
    let message = String::from_utf8(vec).unwrap();
    println!("{}", message);
    loop {
        let mut buffer = [0; 1024];
        thread::sleep(Duration::from_millis(500));
        socket_stream.write_all(b"message \"FOOBAAA\"")?;
        let size = socket_stream.read(&mut buffer)?;
        let mut vec = Vec::with_capacity(size);
        vec.extend_from_slice(&buffer[0..size]);
        let message = String::from_utf8(vec).unwrap();
        println!("{}", message);
        thread::sleep(Duration::from_millis(500));
    }
}
