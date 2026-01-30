use anyhow::Result;
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(
    socket: TcpStream,
    addr: SocketAddr,
    active_conns: Arc<AtomicUsize>,
) -> Result<()> {
    let count = active_conns.fetch_add(1, Ordering::SeqCst) + 1;
    println!("Processing {} (active connections: {})", addr, count);

    // Simulation of RESP protocol parsing
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let count = active_conns.fetch_sub(1, Ordering::SeqCst) - 1;
    println!("Finished {} (active connections: {})", addr, count);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let active_conns = Arc::new(AtomicUsize::new(0));

    println!("Redis server starting... {}", listener.local_addr()?.ip());

    loop {
        tokio::select! {
            // Accept new connections
            result = listener.accept() => {
                let (socket, addr) = result?;
                let active_conns = Arc::clone(&active_conns);

                println!("Accepted connection from: {}", addr);

                // Spawn independent task for this connection
                // `async move` transfers ownership to task
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, addr, active_conns).await {
                        eprintln!("Connection error for {}: {}", addr, e);
                    }
                });
            }

            _ = tokio::signal::ctrl_c() => {
                let final_count = active_conns.load(Ordering::SeqCst);
                println!("Active connections: {}", final_count);
                println!("Ctrl + c detected, shutting down...");
                break;
            }
        }
    }

    println!("Server shutdown complete");
    Ok(())
}
