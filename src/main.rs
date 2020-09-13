use std::fs;
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

use std::io::prelude::*;

fn create_response(content: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContentLength: {}\r\n\r\n{}",
        content.len(),
        content
    )
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if let Ok(read_bytes) = stream.read(&mut buffer) {
        println!(
            "Request({} bytes):\n{}",
            read_bytes,
            String::from_utf8_lossy(&buffer[..])
        );
    };

    let site_contents = fs::read_to_string("index.html").unwrap();
    let response = create_response(&site_contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
