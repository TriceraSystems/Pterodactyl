use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct JsonResponse {
    pub code: u16,                 // HTTP status code
    pub message: String,           // Small response summary
    pub data: Value,               // Processed Data (arbitrary JSON)
    pub errors: Value,             // Any errors (arbitrary JSON)
    pub timestamp: String,         // Timestamp of process response
    pub cache: bool,               // Is the response from cache
    pub cost: f64,                 // Processing cost (adjust type as needed)
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
            "timestamp": timestamp,
            "cache": cache,
            "cost": cost,
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
            timestamp,
            cache,
            cost,
            hash: hash_hex,
        }
    }
}