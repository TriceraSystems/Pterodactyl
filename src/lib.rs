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

// Struct that represents a JSON response to the client
#[derive(Serialize, Deserialize)]
struct JsonResponse {
    code: u16,              // HTTP status code
    message: String,        // A short message describing the result
    data: Value,            // Arbitrary JSON data returned in the response
    errors: Vec<String>,    // List of errors (if any occurred)
    timestamp: String,      // Timestamp of when the response was generated
    cache: bool,            // Indicates if the response was retrieved from cache
    cost: f64,              // The time cost in seconds for processing
    hash: String,           // A SHA256 hash of the response data for verification
}

// Struct representing the result of processing a request
pub struct ProcessResponse {
    code: u16,
    message: String,
    data: Value,
    errors: Vec<String>,
}

// The main server struct that holds all available processes
pub struct Server {
    // A thread-safe, shared collection of processes
    processes: Arc<RwLock<HashMap<String, Box<dyn Fn() -> Result<ProcessResponse, ProcessResponse> + Send + Sync>>>>,
}

impl JsonResponse {
    // Constructor to create a new JsonResponse
    fn new(code: u16, message: String, data: Value, errors: Vec<String>, cache: bool, cost: f64) -> Self {
        // Capture current system time as a UNIX timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .to_string();

        // Initialize the response object with all fields except the hash
        let mut response = Self {
            code,
            message,
            data,
            errors,
            timestamp,
            cache,
            cost,
            hash: String::new(), // Set hash to an empty string initially
        };

        // Serialize the JSON response data (excluding the hash field) to calculate the SHA256 hash
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

        // Calculate the SHA256 hash and assign it to the response
        response.hash = hex::encode(Sha256::digest(&json_bytes));
        response
    }
}

impl ProcessResponse {
    // Constructor for creating a new ProcessResponse
    pub fn new(code: u16, message: String, data: Value, errors: Vec<String>) -> Self {
        Self {
            code,
            message,
            data,
            errors,
        }
    }

    // Convert the ProcessResponse into a JsonResponse, adding cache status and processing cost
    fn to_json_response(&self, cache: bool, cost: f64) -> JsonResponse {
        JsonResponse::new(self.code, self.message.clone(), self.data.clone(), self.errors.clone(), cache, cost)
    }
}

impl Server {
    // Constructor to create a new Server instance
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())), // Initialize a thread-safe map of processes
        }
    }

    // Function to handle incoming HTTP requests asynchronously
    async fn incoming(self: Arc<Self>, _req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let process_id = "test2"; // Placeholder for process identifier, should be dynamic in a real system

        // Access the process map for reading
        let processes = self.processes.read().unwrap();
        let process = match processes.get(process_id) {
            // If the process is found, use it
            Some(p) => p,
            // If the process is not found, return a 404 response
            None => {
                let response = JsonResponse::new(
                    404,
                    "Not Found".to_string(),
                    serde_json::json!({}), // Empty JSON data
                    vec!["Process not found".to_string()],
                    false,
                    0.0,
                );
                let json_response = serde_json::to_string(&response).unwrap();

                // Build and return the HTTP response with JSON body
                return Ok(Response::builder()
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json_response)))
                    .unwrap());
            }
        };

        // Start a timer to measure processing time
        let start = Instant::now();
        let result = process(); // Call the process
        let duration = start.elapsed();
        println!("Execution time: {:?}", duration);

        // Depending on the result of the process, create a success or error response
        let response = match result {
            Ok(success_response) => success_response.to_json_response(false, duration.as_secs_f64()),
            Err(error_response) => error_response.to_json_response(false, 0.0),
        };

        let json_response = serde_json::to_string(&response).unwrap();

        // Build and return the final JSON HTTP response
        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json_response)))
            .unwrap())
    }

    // Function to add a new process to the server
    pub fn add_process<F>(&self, name: &str, func: F)
    where
        F: Fn() -> Result<ProcessResponse, ProcessResponse> + Send + Sync + 'static,
    {
        let mut processes = self.processes.write().unwrap();
        // Insert the process function into the map, identified by the given name
        processes.insert(name.to_string(), Box::new(func));
    }

    // Function to start the server and listen for incoming connections
    pub async fn start(self: Arc<Self>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(addr).await?; // Bind to the given address
        println!("Server started on http://{}", addr);

        // Server loop to handle incoming connections
        loop {
            let (stream, _) = listener.accept().await?;
            let io = hyper_util::rt::TokioIo::new(stream); // Wrap the TCP stream for Hyper
            let self_clone = Arc::clone(&self); // Clone the server to move into the async task

            // Spawn a new task to handle the connection
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(move |req| self_clone.clone().incoming(req)))
                    .await
                {
                    // Log errors that occur during connection handling
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}
