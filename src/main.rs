use std::io;
use std::str;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

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
                    Ok(n) => {
                        let all = str::from_utf8(&buf[..n]).expect("");
                        for line in all.split("\r\n") {
                            println!("{} length {}", line, line.len());
                            if line == "ping" {
                                if socket.write_all("+PONG\r\n".as_bytes()).await.is_err() {
                                    eprintln!("write error");
                                    return;
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
