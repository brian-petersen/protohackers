use std::{net::SocketAddr, sync::Arc};

use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::RwLock,
};
use uuid::Uuid;

use crate::util::Result;

const PREFIX: &str = "BUDGETCHAT";

struct Server {
    users: Vec<User>,
}

struct User {
    socket: OwnedWriteHalf,
    username: String,
    uuid: Uuid,
}

impl Server {
    fn new() -> Server {
        Server { users: vec![] }
    }

    async fn add_user(
        &mut self,
        username: String,
        socket: TcpStream,
        server_lock: Arc<RwLock<Server>>,
    ) -> Result<()> {
        let (read_half, mut write_half) = socket.into_split();

        let usernames = self.get_usernames();
        write_half
            .write_all(format!("* The room contains: {}\n", usernames).as_bytes())
            .await?;

        let user = User::new(username, write_half);
        let username = user.username.clone();
        let user_uuid = user.uuid;
        self.users.push(user);

        let message = format!("* {} has entered the room\n", username);
        self.broadcast_message(&user_uuid, message.as_bytes())
            .await?;

        tokio::spawn(async move {
            listen_for_messages(user_uuid, read_half, server_lock).await;
        });

        Ok(())
    }

    async fn broadcast_message(&mut self, sender: &Uuid, message: &[u8]) -> Result<()> {
        for user in &mut self.users {
            if user.uuid == *sender {
                continue;
            }

            user.socket.write_all(message).await?;
            user.socket.flush().await?;
        }

        Ok(())
    }

    async fn broadcast_prefixed_message(&mut self, sender: &Uuid, message: String) -> Result<()> {
        let sender_username = self
            .users
            .iter()
            .find(|u| u.uuid == *sender)
            .unwrap()
            .username
            .to_string();

        let message = format!("[{}] {}", sender_username, message);
        self.broadcast_message(sender, message.as_bytes()).await?;

        Ok(())
    }

    async fn disconnect(&mut self, user_uuid: &Uuid) -> Result<()> {
        let index = self
            .users
            .iter()
            .position(|u| u.uuid == *user_uuid)
            .unwrap();
        let removed_user = self.users.remove(index);

        let message = format!("* {} has left the room\n", removed_user.username);
        self.broadcast_message(user_uuid, message.as_bytes())
            .await?;

        Ok(())
    }

    fn get_usernames(&self) -> String {
        self.users
            .iter()
            .map(|u| u.username.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl User {
    fn new(username: String, socket: OwnedWriteHalf) -> User {
        User {
            socket,
            username,
            uuid: Uuid::new_v4(),
        }
    }
}

pub async fn start(port: &str) -> Result<()> {
    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address).await?;

    println!("[{}] Server listening on {}", PREFIX, &address);

    let server_lock = Arc::new(RwLock::new(Server::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        let server_lock_clone = server_lock.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr, server_lock_clone).await {
                eprintln!("[{}] Error occurred: {}", PREFIX, e);
            }
        });
    }
}

async fn get_username(socket: &mut TcpStream) -> Result<String> {
    socket
        .write_all("Welcome to budgetchat! What shall I call you?\n".as_bytes())
        .await?;
    socket.flush().await?;

    let mut name = String::new();
    let mut reader = BufReader::new(socket);
    reader.read_line(&mut name).await?;
    name = name.trim().to_string();

    // TODO should find a way to compile this regex only once
    let regex = Regex::new("^[a-zA-Z0-9]+$").unwrap();
    if !regex.is_match(&name) {
        return Err(format!("Invalid username {:?}", name).into());
    }

    Ok(name)
}

async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    server_lock: Arc<RwLock<Server>>,
) -> Result<()> {
    println!("[{}] Connection established from {}", PREFIX, addr);

    let name = get_username(&mut socket).await?;

    let mut server = server_lock.write().await;
    server.add_user(name, socket, server_lock.clone()).await?;

    Ok(())
}

async fn listen_for_messages(
    sender_uuid: Uuid,
    mut socket: OwnedReadHalf,
    server_lock: Arc<RwLock<Server>>,
) {
    let mut reader = BufReader::new(&mut socket);

    loop {
        let mut message = String::new();

        match reader.read_line(&mut message).await {
            Ok(n) if n == 0 => {
                let mut server = server_lock.write().await;
                server.disconnect(&sender_uuid).await.unwrap();
                return;
            }
            Ok(_) => {
                let mut server = server_lock.write().await;
                server
                    .broadcast_prefixed_message(&sender_uuid, message)
                    .await
                    .unwrap();
            }
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}
