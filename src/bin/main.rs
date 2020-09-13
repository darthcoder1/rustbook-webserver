use std::collections::HashMap;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::str::Lines;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
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

#[derive(Debug, PartialEq)]
enum RequestType {
    Get,
    Put,
    Post,
}

#[derive(Debug)]
struct HttpHeader {
    request_type: RequestType,
    uri: String,
    version: String,
    headers: HashMap<String, String>,
    content: Option<String>,
}

impl HttpHeader {
    fn parse(lines: &mut Lines) -> Option<HttpHeader> {
        if let Some(start_line) = lines.next() {
            let tokens: Vec<&str> = start_line.split(' ').collect();

            if tokens.len() != 3 {
                println!("Unable to parse message header");
                return None;
            }

            let req_type = match tokens[0] {
                "GET" => RequestType::Get,
                "PUT" => RequestType::Put,
                "POST" => RequestType::Post,
                _ => {
                    println!("Unable to parse message type: {}", tokens[0]);
                    return None;
                }
            };

            let uri = String::from(tokens[1]);
            let version = String::from(tokens[2]);

            let mut headers = HashMap::new();
            while let Some(line) = lines.next() {
                if line == "" {
                    break;
                }

                let tokens: Vec<&str> = line.split(":").collect();
                headers.insert(
                    String::from(tokens[0].trim()),
                    String::from(tokens[1].trim()),
                );
            }

            let mut content = String::new();

            if req_type != RequestType::Get {
                while let Some(line) = lines.next() {
                    content.push_str(line);
                    content.push_str("\r\n");
                }
            }
            return Some(HttpHeader {
                request_type: req_type,
                uri,
                version,
                headers,
                content: if content.len() > 0 {
                    Some(content)
                } else {
                    None
                },
            });
        }
        None
    }
}

fn respond_error_404(stream: &mut TcpStream) {
    let status_line = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
    let contents = fs::read_to_string("404.html").unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if let Ok(read_bytes) = stream.read(&mut buffer) {
        if let Some(header) = HttpHeader::parse(&mut String::from_utf8_lossy(&buffer[..]).lines()) {
            println!(">> HttpMessage ({} bytes)\n{:?}", read_bytes, header);

            match header.request_type {
                RequestType::Get => {
                    let uri_path = &header.uri[1..];
                    if let Ok(content) = fs::read_to_string(uri_path) {
                        let response = create_response(&content);
                        stream.write(response.as_bytes()).unwrap();
                    } else {
                        respond_error_404(&mut stream);
                    }
                }
                _ => respond_error_404(&mut stream),
            }
        }
    };

    stream.flush().unwrap();
}
