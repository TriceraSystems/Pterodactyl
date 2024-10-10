use pterodactyl::{Server, ProcessResponse};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::new();

    server.add_process("example", || {
        for i in 1..1_000_000_i32 {
            let _ = i+i;
        }

        let response = ProcessResponse::new(
            200,
            "Success".to_string(),
            serde_json::json!({
                "message": "Hello, World!",
                "data": {
                    "key": "value"
                }
            }),
            [].to_vec(),
        );

        Ok(response)
    });

    Arc::new(server).start(addr).await
}