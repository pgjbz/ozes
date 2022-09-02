use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket_stream = TcpStream::connect("localhost:7656")?;
    socket_stream.write_all(b"subscribe foo with group bar")?;
    loop {
        let mut buffer = [0; 1024];
        let size = socket_stream.read(&mut buffer)?;
        let mut vec = Vec::with_capacity(size);
        vec.extend_from_slice(&buffer[0..size]);
        let message = String::from_utf8(vec).unwrap();
        println!("{}", message);
        socket_stream.write_all(b"ok")?;
        thread::sleep(Duration::from_secs(1));
    }
}
