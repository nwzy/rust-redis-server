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
    // Note on `.fetch_add` and `.fetch_sub`:
    // These methods add and and subtract at the CPU level and typically compile
    // down to a single CPU instruction with a `LOCK` prefix. This instruction
    // provides exlusive access to the memory location.
    //
    // Note on `+ 1` and `- 1`:
    // Calling `.fetch_add` and `.fetch_sub` returns the number _before_ the
    // operation occurred, while incrementing/decrementing it in the background.
    // This necessiates the addition/subtraction to get the "actual" count
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
                // Note on `Arc::clone()`: Even though `handle_connection` takes in a
                // Arc<AtomicUsize> type, we have to create a new handle for it
                // so that ownership can be passed into the task in the thread,
                // otherwise the original handle would be cleaned up, violating
                // its lifetime requirements.
                // By creating a new handle and handing ownership to the task,
                // every thread can have their own reference to the counter
                // which can be safely cleaned up once the task completes.

                // Relevent doc for `Arc::clone()`:
                // The type Arc<T> provides shared ownership of a value of type
                // T, allocated in the heap. Invoking clone on Arc produces a
                // new Arc instance, which points to the same allocation on the
                // heap as the source Arc, while increasing a reference count.
                // When the last Arc pointer to a given allocation is destroyed,
                // the value stored in that allocation (often referred to as
                // "inner value") is also dropped.
                let active_conns = Arc::clone(&active_conns);

                println!("Accepted connection from: {}", addr);

                // `async move` transfers ownership of `socket`, `addr`, and
                // `active_conns` to allow the spawned task `handle_connection`
                // to continue running even if the main thread ends.
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
