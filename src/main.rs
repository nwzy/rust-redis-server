mod server;

use server::{Server, ServerConfig};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig::default();
    let server = Server::new(config);
    server.run().await
}
