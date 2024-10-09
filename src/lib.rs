use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
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

// Define the structure of our JSON response
#[derive(Serialize, Deserialize)]
pub struct JsonResponse {
    code: u16,                 // HTTP status code
    message: String,           // Small response summary
    data: Value,               // Processed Data (arbitrary JSON)
    errors: Value,             // Any errors (arbitrary JSON)
    timestamp: String,         // Timestamp of process response
    cache: bool,               // Is the response from cache
    cost: f64,                 // Processing cost (adjust type as needed)
    hash: String               // SHA256 Hash of response excluding self
}

impl JsonResponse {
    // Constructor for JsonResponse
    pub fn new(code: u16, message: String, data: Value, errors: Value, cache: bool, cost: f64) -> Self {
        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .to_string();

        // Create response object without hash
        let response = Self {
            code,
            message,
            data,
            errors,
            timestamp,
            cache,
            cost,
            hash: String::new(), // Placeholder, will be filled later
        };

        // Serialize response to JSON bytes (excluding hash)
        let json_bytes = serde_json::to_vec(&serde_json::json!({
            "code": response.code,
            "message": &response.message,
            "data": &response.data,
            "errors": &response.errors,
            "timestamp": &response.timestamp,
            "cache": response.cache,
            "cost": response.cost,
        })).expect("Serialization failed");

        // Calculate SHA256 hash of JSON bytes
        let hash = Sha256::digest(&json_bytes);
        let hash_hex = hex::encode(hash);

        // Return final response with hash included
        Self { hash: hash_hex, ..response }
    }
}

// Define Server structure
pub struct Server {
    processes: Vec<String>,
}

impl Server {
    // Constructor for Server
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Server {
            processes: Vec::new(),
        })
    }

    // Handler for incoming requests
    async fn index(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        // Create a new JsonResponse
        let response = JsonResponse::new(
            200,
            "Success".to_string(),
            serde_json::json!({}),
            serde_json::json!([]),
            false,
            0.0,
        );
       
        // Serialize the JsonResponse to a string
        let json_response = serde_json::to_string(&response).unwrap();
   
        // Build the HTTP response
        let response = Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .unwrap();
   
        Ok(response)
    }

    // Method to start the server
    pub async fn start(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Bind to the specified address
        let listener = TcpListener::bind(addr).await?;
        println!("Server started on http://{}", addr);

        // Main server loop
        loop {
            // Accept incoming connections
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            // Spawn a new task for each connection
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(Self::index))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}