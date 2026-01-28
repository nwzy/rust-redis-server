use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(socket: TcpStream, addr: SocketAddr) -> Result<()> {
    println!("Processing connection from {}", addr);

    // Simulate potential error
    if addr.port() % 2 == 0 {
        anyhow::bail!("Simulated error for even ports");
    }

    // Simulation of RESP protocol parsing
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    println!("Finished processing {}", addr);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    println!("Redis server starting... {}", listener.local_addr()?.ip());

    loop {
        tokio::select! {
            // Accept new connections
            result = listener.accept() => {
                let (socket, addr) = result?;
                println!("Accepted connection from: {}", addr);

                // Spawn independent task for this connection
                // `async move` transfers ownership to task
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, addr).await {
                        eprintln!("Connection error for {}: {}", addr, e);
                    }
                });
            }

            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl + c detected, shutting down...");
                break;
            }
        }
    }

    println!("Server shutdown complete");
    Ok(())
}
