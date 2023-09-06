# web_htop

Some practice with frontend and Axum.

![Example GIF](https://github.com/qqrm/web_htop/blob/main/example.gif)

## Overview

`web_htop` is a service built with Axum and Rust that provides real-time monitoring of CPU load via HTTP and WebSockets. The project aims to demonstrate frontend technologies and Rust's async capabilities.

## Features

- Fetch current CPU load via HTTP
- Real-time CPU load monitoring through WebSockets
- Easy to understand, idiomatic Rust code

## Prerequisites

- Rust and Cargo
- Basic understanding of HTTP and WebSockets

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/qqrm/web_htop.git
cd web_htop
```

## Build and Run

### Build

To compile the project, navigate to the project directory and run:

```bash
cargo build
```

### Run

To start the service, run:

```bash
cargo run
```

Your service will start and listen at `http://0.0.0.0:8081`.

## Usage

### HTTP Endpoint

To fetch the current CPU load via HTTP, execute the following command:

```bash
curl http://localhost:8081/api/cpus
```

### WebSocket Endpoint

For real-time monitoring of CPU load, connect to the WebSocket endpoint using a WebSocket client:

```bash
ws://localhost:8081/rt/cpus
```
## Acknowledgments

- Thanks to the Rust community for the language and ecosystem
- The Axum and Tokio teams for their robust libraries
