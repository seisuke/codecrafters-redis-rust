use std::{io::BufWriter, io::Write, net::TcpListener};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                let mut writer = BufWriter::new(&_stream);
                writer
                    .write("+PONG\r\n".as_bytes())
                    .expect("SEND FAILURE!!!");
                writer.flush().unwrap();

                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
