use std::net::{TcpListener, TcpStream};
use std::{io::BufRead, io::BufReader};
use std::{io::BufWriter, io::Write};

fn pong(writer: &mut BufWriter<&TcpStream>) {
    writer.write("+PONG\r\n".as_bytes()).unwrap();
    writer.flush().unwrap();
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                let mut reader = BufReader::new(&_stream);
                let mut writer = BufWriter::new(&_stream);

                loop {
                    let mut msg = String::new();
                    let result = reader.read_line(&mut msg);
                    match result {
                        Ok(_size) => {
                            println!("{}", msg);
                            if msg.as_str() == "ping\r\n" {
                                pong(&mut writer);
                            }
                        }
                        Err(_error) => {
                            break;
                        }
                    }
                }
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
