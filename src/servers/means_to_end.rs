use std::net::SocketAddr;

use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};

use crate::util::Result;

const PREFIX: &str = "MEANS2END";

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

#[derive(Debug)]
struct Deposit {
    timestamp: i32,
    price: i32,
}

#[derive(Debug)]
struct Query {
    min_time: i32,
    max_time: i32,
}

#[derive(Debug)]
enum Message {
    Insert(Deposit),
    Query(Query),
}

struct Account {
    deposits: Vec<Deposit>,
}

impl Account {
    fn new() -> Account {
        Account { deposits: vec![] }
    }

    fn deposit(&mut self, deposit: Deposit) {
        self.deposits.push(deposit);
    }

    fn query(&self, min_time: i32, max_time: i32) -> Result<i32> {
        let prices: Vec<i64> = self
            .deposits
            .iter()
            .filter(|d| d.timestamp >= min_time && d.timestamp <= max_time)
            .map(|d| d.price as i64)
            .collect();

        if prices.is_empty() {
            return Ok(0);
        }

        let sum = prices.iter().sum::<i64>();
        let count = prices.len() as i64;

        println!("{} / {}", sum, count);

        Ok((sum / count).try_into()?)
    }
}

async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) {
    println!("[{}] Connection established from {}", PREFIX, addr);

    let mut account = Account::new();

    let (read_half, write_half) = socket.split();

    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    loop {
        let mut raw_message = [0; 9];

        if let Err(e) = reader.read_exact(&mut raw_message).await {
            eprintln!("Failed to read from socket: {}", e);
            return;
        };

        let message = match decode_message(&raw_message) {
            Ok(message) => message,
            Err(e) => {
                eprintln!("Failed to decode message: {}", e);
                return;
            }
        };

        match message {
            Message::Insert(deposit) => account.deposit(deposit),
            Message::Query(query) => {
                let balance = match account.query(query.min_time, query.max_time) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Failed to query balance: {}", e);
                        return;
                    }
                };

                if let Err(e) = send_message(&mut writer, &balance.to_be_bytes()).await {
                    eprintln!("Failed to send message to socket: {}", e);
                    return;
                }
            }
        }
    }
}

fn decode_message(raw_message: &[u8; 9]) -> Result<Message> {
    match raw_message[0] {
        b'I' => Ok(Message::Insert(Deposit {
            timestamp: decode_int32(&raw_message[1..5])?,
            price: decode_int32(&raw_message[5..9])?,
        })),
        b'Q' => Ok(Message::Query(Query {
            min_time: decode_int32(&raw_message[1..5])?,
            max_time: decode_int32(&raw_message[5..9])?,
        })),
        _ => Err("Not supported".into()),
    }
}

fn decode_int32(bytes: &[u8]) -> Result<i32> {
    match bytes.try_into() {
        Ok(bytes) => Ok(i32::from_be_bytes(bytes)),
        _ => Err("Failed to parse i32".into()),
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
