use std::{net::TcpStream, time::Duration, thread, io::Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket_stream = TcpStream::connect("localhost:7656")?;
    socket_stream.write_all(b"PUBLISH: foo")?;
    loop {
        thread::sleep(Duration::from_millis(500));
        socket_stream.write_all(b"FOOBAAA")?;
        thread::sleep(Duration::from_millis(500));
    }
}