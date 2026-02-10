use std::{
    net::SocketAddr,
    sync::{Arc, atomic::AtomicUsize, atomic::Ordering},
};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};

/// Server Configuration file
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
    pub max_connections: usize,
}
/// The TCP Server implementation
///
/// # Design Choices
///
/// ## Notes on `.fetch_add` and `.fetch_sub`
///
/// * These methods add and and subtract at the CPU level and typically compile
/// down to a single CPU instruction with a `LOCK` prefix.
///     * This instruction provides exlusive access to the memory location.
/// * `+ 1` and `- 1` are appended to `.fetch_add` and `.fetch_sub` as they
/// return the number _before_ the operation occurred.
///
/// ## A Short Discussion on `Ordering` Choice
///
/// Initially, `Ordering::SeqCst` (sequantially consistent) was used as this was
/// the default but, to the Rustonomicon's own admission, trying to understand
/// this is to [flirt with madness
/// itself](https://doc.rust-lang.org/nomicon/atomics.html). In the section
/// talking about when to use `SeqCst`, it mentions that "sequential
/// consistency" is definitely the right choice if you're not confident about
/// the other memory orders," but some discussions say that [it's not so
/// simple.](https://github.com/rust-lang/nomicon/issues/166)
///
/// Here is an opportunity to evaluate what ordering we need; in terms of this
/// specific use-case, there is no other dependency on the count other than to
/// report to the user the number of active-connections (which is only to serve
/// as an exercise in async, atomic programming), so the strong consistency
/// guarantee is "wasted." And if the point is to practice good software design,
/// then it might be prudent to use the less strict of the ordering choices
/// given the use-case (we're just counting).
///
/// And, we can always swap it out in the future. Thanks, Rust!
///
/// Some easier reading:
/// * [Chapter 3. Memory Ordering (mara.nl)](https://mara.nl/atomics/memory-ordering.html)
///
/// ## `tokio::spawn`, `async move`, and `Arc::clone()`
///
/// For proper async, multi-threaded worflows, all threads must have access to
/// their own references to data they're interested in. Sharing those pointers
/// with others might lead to dangling pointers as other threads or sections of
/// the code.
/// * `tokio::spawn` - Spawns a new thread.
/// * `async move` - There are actually 2 things happening here:
///    * `aync` - Makes the following code return a Future, makes it
///    `await`able, and therefore good to use in threads.
///    [(rust-lang.github.io/async-book)](https://rust-lang.github.io/async-book/part-guide/async-await.html#async-functions)
///     * `move` - Forces the closure to take ownership of all captured
///     variables instead of borrowing
///     them.[(doc.rust-lang)](https://doc.rust-lang.org/std/keyword.move.html)
/// * `Arc::clone()` - Returns a new, reference-counted pointer to the `Arc`
/// structure on the heap that is safe for multi-threading.
pub struct Server {
    config: ServerConfig,
    active_conns: Arc<AtomicUsize>,
}

impl ServerConfig {
    /// Provide a simple default with a "reasonable" limit on connections
    pub fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_owned(),
            port: 6379,
            max_connections: 100,
        }
    }
}

impl Server {
    /// Create a new server instance with the specific server configurations
    pub fn new(config: ServerConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            active_conns: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Start up the Redis server to and listen in on connections
    pub async fn run(self: Arc<Self>) -> Result<()> {
        let addr = format!("{}:{}", self.config.ip, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        println!("Redis server starting... {}", &addr);

        while self.active_conns.load(Ordering::Relaxed) < self.config.max_connections {
            tokio::select! {
                result = listener.accept() => {
                    let (socket, addr) = result?;
                    println!("{}", addr);

                    // let server = self.clone();
                    let server = Arc::clone(&self);
                    // let active_conns = self.active_conns.clone();
                    let active_conns = Arc::clone(&self.active_conns);

                    tokio::spawn(async move {
                        let count = active_conns.fetch_add(1, Ordering::Relaxed) + 1;
                        println!("Processing {} (active connections: {})", addr, count);

                        server.handle_connection(socket, addr).await;
                        println!("Client addr: {}", addr);
                        println!("Active connections: {}", active_conns.load(Ordering::Relaxed));

                        let count = active_conns.fetch_sub(1, Ordering::Relaxed) - 1;
                        println!("Finished {} (active connections: {})", addr, count);
                    });
                }

                _ = tokio::signal::ctrl_c() => {
                    self.shutdown();
                    break;
                }
            }
        }
        Ok(())
    }

    /// Shutdown the server with commands
    pub fn shutdown(self: Arc<Self>) {
        let final_count = &self.active_conns.load(Ordering::Relaxed);
        println!("Active connections: {}", final_count);
        println!("Ctrl + c detected, shutting down...")
    }

    /// Connection handler that carries out requests on the Redis server.
    async fn handle_connection(
        self: Arc<Self>, // Important for spawned tasks
        socket: TcpStream,
        addr: SocketAddr,
    ) {
        // Using `sleep` for now to simulate some work in the future
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        // Just using `socket` to keep Rust from complaining
        assert_eq!(addr, socket.peer_addr().unwrap());
    }
}
