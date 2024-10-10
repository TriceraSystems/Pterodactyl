# Pterodactyl (JSON REST API Framework)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/TriceraSystems/Pterodactyl/.github%2Fworkflows%2Frust.yml?branch=main&style=flat-square&label=rust%20test) 
![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/TriceraSystems/Pterodactyl?style=flat-square)
![GitHub repo size](https://img.shields.io/github/repo-size/TriceraSystems/Pterodactyl?style=flat-square)

A lightweight, scalable JSON REST API framework designed for speed and agility. It features built-in capabilities like cost limiting (based on processing time) and standardisation. Unlike traditional routing APIs, all requests are directed to a central index as POST requests, which include essential details such as process ID, HTTP method, payload, and cache. This streamlined approach ensures efficient resource management while maintaining high performance.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [API](#api)
    - [`Server.start(addr: SocketAddr)`](#serverstartaddr-socketaddr)
    - [`Server.add_process<F>(name: &str, func: F)`](#serveradd_processfname-str-func-f)
- [Benchmarking](#benchmarking)

## Installation

To install **Pterodactyl** as a library, run the following Cargo command in your project directory:

```bash
cargo add pterodactyl
```

Or, manually add the following line to your `Cargo.toml`:

```toml
pterodactyl = "0.1.0"
```

## Usage

**Pterodactyl** is designed to offer full control over key aspects of your API, such as HTTP version (`http/1.1` or `http/2.0`) and resource limiters. However, the request and response schema are fixed to maintain consistency and standardisation across projects.

Here's a [simple example](/examples/simple.rs) to get you started:

```rust
use pterodactyl::{Server, ProcessResponse};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::new()?;

    server.add_process("example", || {

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
```

This basic setup provides an ideal starting point for building scalable and efficient servers with **Pterodactyl**, while still giving you flexibility over most components.


## API Documentation

The **Pterodactyl** package provides a public class called `Server`, which serves as the core of the package. All operations are performed through this class.

### `Server.start(addr: SocketAddr)`

This function starts the server, binding it to the provided socket address (`addr`).

- **Parameters**:
  - `addr`: A `SocketAddr` that specifies the address and port on which the server will listen.

#### Example

```rust
use pterodactyl::Server;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Define the address where the server will listen (127.0.0.1:3000)
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    // Create a new instance of the Server
    let server = Server::new();

    // Start the server asynchronously
    Arc::new(server).start(addr).await;
}
```

This example demonstrates how to start a new server instance listening on `127.0.0.1:3000` using the Tokio runtime for asynchronous execution.

### `Server.add_process<F>(name: &str, func: F)`

This method allows you to add a new process (handler) to the server. The process is identified by its name, and the associated function is invoked when the process is triggered.

- **Parameters**:
  - `name`: A `&str` that specifies the name of the process.
  - `func`: A function or closure with the signature `Fn() -> Result<ProcessResponse, ProcessResponse>`, which returns a `ProcessResponse` on success or failure.

#### Example

```rust
use pterodactyl::{Server, ProcessResponse};
use std::net::SocketAddr;
use std::sync::Arc;

fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    // Create a new server instance
    let server = Server::new();

    // Add a new process named "example" that returns a JSON response
    server.add_process("example", || {
        let response = ProcessResponse::new(
            200, // HTTP status code
            "Success".to_string(), // Status message
            serde_json::json!({    // JSON response body
                "message": "Hello, World!",
                "data": {
                    "key": "value"
                }
            }),
            [].to_vec(), // Optional headers
        );

        Ok(response) // Return the response
    });
}
```

In this example, we create a process named `"example"` that returns a JSON response with a status code of 200. This process will be available on the server once it's added.

## Benchmarking
