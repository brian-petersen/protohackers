use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

use crate::util::Result;

const PREFIX: &str = "PRIMETIME";

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

#[derive(Debug, Deserialize)]
struct Request {
    #[allow(dead_code)]
    method: Methods,
    number: f64,
}

#[derive(Debug, Serialize)]
struct Response {
    method: Methods,
    prime: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Methods {
    IsPrime,
}

async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) {
    println!("[{}] Connection established from {}", PREFIX, addr);

    let (read_half, write_half) = socket.split();

    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    loop {
        let mut raw_request = String::new();

        match reader.read_line(&mut raw_request).await {
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                return;
            }
        };

        println!("[{}] Read message from {}: {}", PREFIX, addr, raw_request);

        let request = match serde_json::from_str::<Request>(&raw_request) {
            Ok(request) => request,
            Err(e) => {
                eprintln!("Failed to decode request: {}", e);

                if let Err(e) = send_message(&mut writer, &[0, 1, 2, 3]).await {
                    eprintln!("Failed to send message to socket: {}", e);
                    return;
                }

                if let Err(e) = socket.shutdown().await {
                    eprintln!("Failed to shutdown socket: {}", e);
                }

                return;
            }
        };

        println!("[{}] Parsed request from {}: {:?}", PREFIX, addr, request);

        let response = Response {
            method: Methods::IsPrime,
            prime: primes::is_prime(request.number as u64),
        };

        let mut raw_response = match serde_json::to_vec(&response) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed encode response: {}", e);
                return;
            }
        };

        // add new line
        raw_response.push(0xA);

        println!("[{}] Sending response to {}", PREFIX, addr);

        if let Err(e) = send_message(&mut writer, &raw_response).await {
            eprintln!("Failed to send message to socket: {}", e);
            return;
        }
    }
}

async fn send_message<W: AsyncWrite + Unpin>(
    writer: &mut BufWriter<W>,
    message: &[u8],
) -> Result<()> {
    writer.write_all(message).await?;
    writer.flush().await?;

    Ok(())
}
