use std::io;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(_) => {
                        let all = str::from_utf8(&buf).expect("");
                        let command: Vec<&str> = all.split("\r\n").collect();
                        println!("command\n{}", all);
                        let mut parser = Parser::new(command);
                        let frame = parser.parse_frame();
                        println!("frame {:?}", frame);
                        match frame {
                            Frame::String(_) => {}
                            Frame::Array(array) => match array.as_slice() {
                                [Frame::String(msg)] => {
                                    if msg == "ping" {
                                        if socket.write_all("+PONG\r\n".as_bytes()).await.is_err() {
                                            eprintln!("write error");
                                        }
                                    }
                                }
                                [Frame::String(command), Frame::String(value)] => {
                                    if command == "echo" {
                                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                                        if socket.write_all(response.as_bytes()).await.is_err() {
                                            eprintln!("write error");
                                        }
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
