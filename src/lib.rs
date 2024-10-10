use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use http_body_util::Full;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize)]
struct JsonResponse {
    code: u16,              // HTTP status code
    message: String,        // Small response summary
    data: Value,            // Processed Data (arbitrary JSON)
    errors: Vec<String>,    // Any errors (arbitrary JSON)
    timestamp: String,      // Timestamp of process response
    cache: bool,            // Is the response from cache
    cost: f64,              // Processing cost (adjust type as needed)
    hash: String,           // SHA256 Hash of response excluding self
}

impl JsonResponse {
    // Constructor for JsonResponse
    fn new(code: u16, message: String, data: Value, errors: Vec<String>, cache: bool, cost: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .to_string();

        // Create response object without hash
        let mut response = Self {
            code,
            message,
            data,
            errors,
            timestamp,
            cache,
            cost,
            hash: String::new(),
        };

        // Serialize and calculate SHA256 hash of JSON bytes (excluding hash)
        let json_bytes = serde_json::to_vec(&serde_json::json!({
            "code": response.code,
            "message": &response.message,
            "data": &response.data,
            "errors": &response.errors,
            "timestamp": &response.timestamp,
            "cache": response.cache,
            "cost": response.cost,
        }))
        .expect("Serialization failed");

        response.hash = hex::encode(Sha256::digest(&json_bytes));
        response
    }
}

pub struct ProcessResponse {
    code: u16,
    message: String,
    data: Value,
    errors: Vec<String>,
}

impl ProcessResponse {
    pub fn new(code: u16, message: String, data: Value, errors: Vec<String>) -> Self {
        Self {
            code,
            message,
            data,
            errors,
        }
    }

    // Serialize ProcessResponse to JsonResponse
    fn to_json_response(&self, cache: bool, cost: f64) -> JsonResponse {
        JsonResponse::new(self.code, self.message.clone(), self.data.clone(), self.errors.clone(), cache, cost)
    }
}

type FuncType = Box<dyn Fn() -> Result<ProcessResponse, ProcessResponse> + Send + Sync>;

pub struct Server {
    processes: Arc<RwLock<HashMap<String, FuncType>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn incoming(self: Arc<Self>, _req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let process_id = "test2"; // Placeholder for the actual process identifier

        let processes = self.processes.read().unwrap();
        let process = match processes.get(process_id) {
            Some(p) => p,
            None => {
                let response = JsonResponse::new(
                    404,
                    "Not Found".to_string(),
                    serde_json::json!({}),
                    vec!["Process not found".to_string()],
                    false,
                    0.0,
                );
                let json_response = serde_json::to_string(&response).unwrap();

                return Ok(Response::builder()
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json_response)))
                    .unwrap());
            }
        };

        let start = Instant::now();
        let result = process();
        let duration = start.elapsed();
        println!("Execution time: {:?}", duration);

        let response = match result {
            Ok(success_response) => success_response.to_json_response(false, duration.as_secs_f64()),
            Err(error_response) => error_response.to_json_response(false, 0.0),
        };

        let json_response = serde_json::to_string(&response).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .unwrap())
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
                    .serve_connection(io, service_fn(move |req| self_clone.clone().incoming(req)))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}
