use std::io;
use std::str;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug)]
pub enum Frame {
    String(String),
    Array(Vec<Frame>),
}

pub struct Parser<'a> {
    command: Vec<&'a str>,
    pos: usize,
}

impl Parser<'_> {
    pub fn new(command: Vec<&str>) -> Parser {
        Parser {
            command,
            pos: 0,
        }
    }

    pub fn get_decimal(&mut self, src: &str) -> u64 {
        src.parse::<u64>().unwrap()
    }

    pub fn parse_frame(&mut self) -> Frame {
        let head = self.command[self.pos];
        match head.chars().next().unwrap() {
            '*' => {
                let len = self.get_decimal(&head[1..]) as usize;
                let mut out = Vec::with_capacity(len);
                for _ in 0..len {
                    self.pos += 1;
                    out.push(self.parse_frame());
                }
                Frame::Array(out)
            }
            '$' => {
                self.pos += 1;
                self.parse_frame()
            }
            _ => {
                self.pos += 1;
                Frame::String(head.to_string())
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("new client: {:?}", addr);
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            let mut map: HashMap<String, String> = HashMap::new();
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(_) => {
                        let request = str::from_utf8(&buf).expect("");
                        let frames: Vec<&str> = request.split("\r\n").collect();
                        println!("command\n{}", request);
                        let mut parser = Parser::new(frames);
                        let frame = parser.parse_frame();
                        println!("frame {:?}", frame);
                        match frame {
                            Frame::String(_) => {}
                            Frame::Array(array) => match array.as_slice() {
                                [Frame::String(command)] => {
                                    if command == "ping" {
                                        send_response(&mut socket, "+PONG\r\n".to_string()).await
                                    }
                                }
                                [Frame::String(command), Frame::String(key)] => {
                                    if command == "echo" {
                                        let response = bulk_string(key);
                                        send_response(&mut socket, response).await
                                    } else if command == "get" {
                                        let value = map.get(key).unwrap();
                                        let response = bulk_string(value);
                                        send_response(&mut socket, response).await
                                    }
                                }
                                [Frame::String(command), Frame::String(key), Frame::String(value)] => {
                                    if command == "set" {
                                        map.insert(key.to_string(), value.to_string());
                                        let response = bulk_string(&"OK".to_string());
                                        send_response(&mut socket, response).await
                                    }
                                }
                                _ => {

                                }
                            }
                        }
                    }
                    Err(_) => {
                        return;
                    }
                }
            }
        });
    }
}

fn bulk_string(value: &String) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}

async fn send_response(socket: &mut TcpStream, response: String) {
    if socket.write_all(response.as_bytes()).await.is_err() {
        eprintln!("write error");
    }
}
