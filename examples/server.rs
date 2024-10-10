use pterodactyl::Server;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::new()?;

    server.add_process("test", || {
        println!("test");
        Ok("test2".to_string())
    });

    Arc::new(server).start(addr).await
}