use std::convert::Infallible;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use http_body_util::Full;
use std::sync::{Arc, RwLock};
use hyper::body::{Bytes, Incoming};

// Define the structure of our JSON response
#[derive(Serialize, Deserialize)]
struct JsonResponse {
    code: u16,                 // HTTP status code
    message: String,           // Small response summary
    data: Value,               // Processed Data (arbitrary JSON)
    errors: Vec<String>,             // Any errors (arbitrary JSON)
    timestamp: String,         // Timestamp of process response
    cache: bool,               // Is the response from cache
    cost: f64,                 // Processing cost (adjust type as needed)
    hash: String               // SHA256 Hash of response excluding self
}

impl JsonResponse {
    // Constructor for JsonResponse
    fn new(code: u16, message: String, data: Value, errors: Vec<String>, cache: bool, cost: f64) -> Self {
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

pub struct ProcessResponse {
    code: u16,
    message: String,
    data: Value,
    errors: Vec<String>,
}

impl ProcessResponse {
    // Constructor for ProcessResponse
    pub fn new(code: u16, message: String, data: Value, errors: Vec<String>) -> Self {

        // Create response object without hash
        let response = Self {
            code,
            message,
            data,
            errors,
        };

        Self { ..response }
    }

    // Serialize the ProcessResponse to a JsonResponse
    fn to_json_response(&self) -> JsonResponse {
        JsonResponse::new(
            self.code,
            self.message.clone(),
            self.data.clone(),
            self.errors.clone(),
            false,
            0.0,
        )
    }
}

type FuncType = Box<dyn Fn() -> Result<ProcessResponse, ProcessResponse> + Send + Sync>;

pub struct Server {
    processes: Arc<RwLock<HashMap<String, FuncType>>>,
}

impl Server {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Server {
            processes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    async fn incoming(self: Arc<Self>, _req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        
        let process_id = "example";

        // Get the process function from the processes HashMap
        let processes = self.processes.read().unwrap();
        let process = processes.get(process_id).unwrap();

        // Call the process function
        let result = process();
        
        // Match on the result to call to_json_response
        let response = match result {
            Ok(success_response) => success_response.to_json_response(),
            Err(error_response) => error_response.to_json_response(),
        };
       
        // Serialize the JsonResponse to a string
        let json_response = serde_json::to_string(&response).unwrap();
   
        // Build the HTTP response
        let response = Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .unwrap();
   
        Ok(response)
    }

    pub fn add_process<F>(&self, name: &str, func: F)
    where
        F: Fn() -> Result<ProcessResponse, ProcessResponse> + Send + Sync + 'static,
    {
        let mut processes = self.processes.write().unwrap();
        processes.insert(name.to_string(), Box::new(func));
    }

    pub async fn start(self: Arc<Self>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server started on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = hyper_util::rt::TokioIo::new(stream);

            let self_clone = Arc::clone(&self);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io, 
                        service_fn(move |req| {
                            let self_clone = Arc::clone(&self_clone);
                            async move { self_clone.incoming(req).await }
                        })
                    )
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}