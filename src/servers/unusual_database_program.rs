use std::collections::HashMap;

use crate::util::Result;
use tokio::net::UdpSocket;

const PREFIX: &str = "UDP";

pub async fn start(port: &str) -> Result<()> {
    let address = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(&address).await?;

    println!("[{}] Server listening on {}", PREFIX, &address);

    let reserved_keys = ["version".to_string()];
    let mut db = HashMap::new();
    db.insert(
        "version".to_string(),
        "luckywatcher's key-value store 1.0".to_string(),
    );

    let mut buffer = [0; 1000];

    loop {
        let (bytes, origin) = socket.recv_from(&mut buffer).await?;

        println!("[{}] Received message from {}", PREFIX, &origin);

        match parse_message(&buffer[0..bytes]) {
            Message::Insert(key, value) => {
                if !reserved_keys.contains(&key) {
                    db.insert(key, value);
                }
            }
            Message::Retrieve(key) => {
                let empty_string = "".to_string();
                let value = db.get(&key).unwrap_or(&empty_string);
                let message = format!("{}={}", &key, &value);
                socket.send_to(message.as_bytes(), origin).await?;
            }
        }
    }
}

#[derive(Debug)]
enum Message {
    Insert(String, String),
    Retrieve(String),
}

fn parse_message(buffer: &[u8]) -> Message {
    let maybe_index = buffer.iter().position(|c| c == &b'=');

    if let Some(index) = maybe_index {
        let key = &buffer[0..index];
        let value = &buffer[index + 1..];

        Message::Insert(
            String::from_utf8_lossy(key).to_string(),
            String::from_utf8_lossy(value).to_string(),
        )
    } else {
        Message::Retrieve(String::from_utf8_lossy(buffer).to_string())
    }
}
