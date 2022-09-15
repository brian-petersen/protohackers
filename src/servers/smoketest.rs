use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::util::Result;

const PREFIX: &str = "SMOKETEST";

pub async fn start(port: &str) -> Result<()> {
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address).await?;

    println!("[{}] Server listening on {}", PREFIX, &address);

    loop {
        let (socket, addr) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(socket, addr).await;
        });
    }
}

async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) {
    println!("[{}] Connection established from {}", PREFIX, addr);

    let mut bytes = [0; 1024];

    loop {
        let bytes_read = match socket.read(&mut bytes).await {
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                return;
            }
        };

        if let Err(e) = socket.write_all(&bytes[0..bytes_read]).await {
            eprintln!("Failed to write to socket: {}", e);
            return;
        }
    }
}
