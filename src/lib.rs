use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::SystemTime;

use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use http_body_util::Full;

#[derive(Serialize, Deserialize)]
pub struct JsonResponse {
    pub code: u16,                 // HTTP status code
    pub message: String,           // Small response summary
    pub data: Value,               // Processed Data (arbitrary JSON)
    pub errors: Value,             // Any errors (arbitrary JSON)
    pub cache: bool,               // Is the response from cache
    pub cost: f64,                 // Processing cost (adjust type as needed)
    pub timestamp: String,         // Timestamp of process response
    pub hash: String               // SHA256 Hash of response excluding self
}

impl JsonResponse {
    pub fn new(code: u16, message: String, data: Value, errors: Value, cache: bool, cost: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .to_string();

        let json_string = serde_json::json!({
            "code": code,
            "message": message,
            "data": data,
            "errors": errors,
            "cache": cache,
            "cost": cost,
            "timestamp": timestamp,
        }).to_string();

        let mut hasher = Sha256::new();
        hasher.update(json_string);
        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);

        JsonResponse {
            code,
            message,
            data,
            errors,
            cache,
            cost,
            timestamp,
            hash: hash_hex,
        }
    }
}

async fn index(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = JsonResponse::new(
        200,
        "Success".to_string(),
        serde_json::json!({}),
        serde_json::json!([]),
        false,
        0.0,
    );
    
    let json_response = serde_json::to_string(&response).unwrap();

    let response = Response::builder()
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json_response)))
        .unwrap();

    Ok(response)
}

pub async fn start_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;

    println!("Server started on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(index))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}