use std::{net::SocketAddr, result};

use lazy_static::lazy_static;
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
};

use crate::util::Result;

const PREFIX: &str = "MOB";

const UPSTREAM_ADDRESS: &str = "chat.protohackers.com:16963";

lazy_static! {
    static ref BOGUSCOIN_RE: Regex = Regex::new("(?m)^7[a-zA-Z0-9]{25,34}$").unwrap();
}

const TONYS_ADDRESS: &str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

pub async fn start(port: &str) -> Result<()> {
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address).await?;

    println!("[{}] Server listening on {}", PREFIX, &address);

    loop {
        let (socket, addr) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                eprintln!("[{}] Error occurred: {}", PREFIX, e);
            }
        });
    }
}

fn change_coins_in_message(message: String) -> String {
    message
        .split(' ')
        .map(|s| BOGUSCOIN_RE.replace(s, TONYS_ADDRESS).to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

async fn handle_connection(socket: TcpStream, addr: SocketAddr) -> Result<()> {
    println!("[{}] Connection established from {}", PREFIX, addr);

    let upstream_socket = TcpStream::connect(&UPSTREAM_ADDRESS).await?;

    let (down_read, down_write) = socket.into_split();
    let (up_read, up_write) = upstream_socket.into_split();

    let down_proxy_handle = tokio::spawn(proxy(down_read, up_write));
    let up_proxy_handle = tokio::spawn(proxy(up_read, down_write));

    tokio::select!(
        _ = down_proxy_handle => (),
        _ = up_proxy_handle => (),
    );

    Ok(())
}

async fn proxy(reader: OwnedReadHalf, writer: OwnedWriteHalf) -> result::Result<(), String> {
    let mut buffed_reader = BufReader::new(reader);
    let mut buffed_writer = BufWriter::new(writer);

    let mut message = String::new();

    loop {
        message.clear();
        buffed_reader.read_line(&mut message).await.unwrap();

        if message.is_empty() {
            return Ok(());
        }

        message = change_coins_in_message(message);

        buffed_writer.write_all(message.as_bytes()).await.unwrap();
        buffed_writer.flush().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_coins() {
        assert_eq!(
            change_coins_in_message("7F1u3wSD5RbOHQmupo9nx4TnhQ".to_string()),
            "7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string()
        );

        assert_eq!(
            change_coins_in_message(" 7F1u3wSD5RbOHQmupo9nx4TnhQ".to_string()),
            " 7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string()
        );

        assert_eq!(
            change_coins_in_message("7F1u3wSD5RbOHQmupo9nx4TnhQ ".to_string()),
            "7YWHMfk9JZe0LM0g1ZauHuiSxhI ".to_string()
        );

        assert_eq!(
            change_coins_in_message(" 7F1u3wSD5RbOHQmupo9nx4TnhQ ".to_string()),
            " 7YWHMfk9JZe0LM0g1ZauHuiSxhI ".to_string()
        );

        assert_eq!(
            change_coins_in_message("send to 7F1u3wSD5RbOHQmupo9nx4TnhQ".to_string()),
            "send to 7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string()
        );

        assert_eq!(
            change_coins_in_message("Please pay the ticket price of 15 Boguscoins to one of these addresses: 7YWHMfk9JZe0LM0g1ZauHuiSxhI 7YWHMfk9JZe0LMsljfsl180SxhI 7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string()),
            "Please pay the ticket price of 15 Boguscoins to one of these addresses: 7YWHMfk9JZe0LM0g1ZauHuiSxhI 7YWHMfk9JZe0LM0g1ZauHuiSxhI 7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string()
        );

        assert_eq!(
            change_coins_in_message(
                "Send product 7YWHMfk9JZe0LM0g1ZauHuiSxhI-uAlVQEafFrMMFNQVY5kC7ENf8VT-1234 to me"
                    .to_string()
            ),
            "Send product 7YWHMfk9JZe0LM0g1ZauHuiSxhI-uAlVQEafFrMMFNQVY5kC7ENf8VT-1234 to me"
                .to_string()
        );

        assert_eq!(
            change_coins_in_message(
                "Please send the payment of 750 Boguscoins to 7P6dFDNGsSJY9fbhQUGlrzSs4bn7benGM\n"
                    .to_string()
            ),
            "Please send the payment of 750 Boguscoins to 7YWHMfk9JZe0LM0g1ZauHuiSxhI\n"
                .to_string()
        );
    }
}
