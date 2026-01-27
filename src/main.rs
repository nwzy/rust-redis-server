use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(socket: TcpStream, addr: SocketAddr) {
    println!("Processing connection from {}", addr);

    // Simulation of RESP protocol parsing
    tokio::time::sleep(
        tokio::time::Duration::from_secs(5), // );
    )
    .await;

    println!("Finished processing {}", addr);
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").await?;

    println!("Redis server starting... listening");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        handle_connection(socket, addr).await;
    }
}
