# Pterodactyl (REST API Framework)

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/TriceraSystems/Pterodactyl/.github%2Fworkflows%2Frust.yml?branch=main&style=flat-square&label=rust%20test) 
![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/TriceraSystems/Pterodactyl?style=flat-square)
![GitHub repo size](https://img.shields.io/github/repo-size/TriceraSystems/Pterodactyl?style=flat-square)

A lightweight, scalable API framework designed for speed and agility. It features built-in capabilities like cost limiting (based on processing time) and standardisation. Unlike traditional routing APIs, all requests are directed to a central index as POST requests, which include essential details such as process ID, HTTP method, payload, and cache. This streamlined approach ensures efficient resource management while maintaining high performance.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Api](#api)
    - [`Server.start(addr)`](#serverstartaddr)
    - [`Server.add_process(name, func)`](#serveradd_processname-func)
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

Here's a simple example to get you started:

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

## API

The **Pterodactyl** package has a public class `Server`, this class is the core of the package everything goes through this class.

### `Server.start(addr)` 
### `Server.add_process(name, func)` 

## Benchmarking


