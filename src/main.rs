use http_body_util::Full;
use hyper_util::rt::TokioIo;
use pterodactyl::JsonResponse; 
use tokio::net::TcpListener;

use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper::server::conn::http1;
use hyper::service::service_fn;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `index` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(index))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}