//! A CPU Load Monitoring Service
//!
//! This service provides real-time monitoring of CPU load
//! through both HTTP and WebSockets.

// Import dependencies.
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router, Server,
};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;

/// Snapshot of CPU usage data.
type Snapshot = Vec<f32>;

/// Application state.
#[derive(Clone)]
struct AppState {
    cpus: Arc<Mutex<Snapshot>>,
    tx: broadcast::Sender<Snapshot>,
}

/// Entry point for the application.
#[tokio::main]
async fn main() {
    // Create a broadcast channel for CPU load data.
    let (tx, _) = broadcast::channel(1);

    // Initialize the application state.
    let app_state = AppState {
        tx: tx.clone(),
        cpus: Arc::new(Mutex::new(vec![])),
    };

    // Define the router.
    let router = Router::new()
        .route("/", get(root))
        .route("/index.mjs", get(index_mjs))
        .route("/index.css", get(index_css))
        .route("/api/cpus", get(get_cpu_load))
        .route("/rt/cpus", get(rt_get_cpu_load))
        .with_state(app_state.clone());

    // Spawn a task for collecting CPU data.
    tokio::spawn(collect_cpu_data(tx));

    // Run the server.
    let addr = "0.0.0.0:8081".parse().unwrap();
    println!("Listening on http://{addr}");
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

/// Collect CPU load data periodically and send it over the broadcast channel.
///
/// # Arguments
///
/// * `tx` - The broadcast channel sender.
async fn collect_cpu_data(tx: broadcast::Sender<Snapshot>) {
    let mut sys = System::new();
    loop {
        sys.refresh_cpu();
        let usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let _ = tx.send(usage);
        tokio::time::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL).await;
    }
}

/// Serve the root HTML page.
#[axum::debug_handler]
async fn root() -> impl IntoResponse {
    Html(tokio::fs::read_to_string("src/index.html").await.unwrap())
}

/// Serve JavaScript resources.
#[axum::debug_handler]
async fn index_mjs() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "application/javascript")
        .body(tokio::fs::read_to_string("src/index.mjs").await.unwrap())
        .unwrap()
}

/// Serve CSS resources.
#[axum::debug_handler]
async fn index_css() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "text/css")
        .body(tokio::fs::read_to_string("src/index.css").await.unwrap())
        .unwrap()
}

/// HTTP Handler to get the current CPU load as a JSON response.
#[axum::debug_handler]
async fn get_cpu_load(State(app_state): State<AppState>) -> impl IntoResponse {
    Json(app_state.cpus.lock().unwrap().clone())
}

/// WebSocket Handler to stream the current CPU load in real-time.
#[axum::debug_handler]
async fn rt_get_cpu_load(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| rt_get_cpus_stream(app_state.clone(), socket))
}

/// Send CPU load data over WebSocket.
///
/// # Arguments
///
/// * `app_state` - The application state.
/// * `socket` - The WebSocket to send data over.
async fn rt_get_cpus_stream(app_state: AppState, mut socket: WebSocket) {
    let mut rx = app_state.tx.subscribe();
    while let Ok(snapshot) = rx.recv().await {
        if let Ok(payload) = serde_json::to_string(&snapshot) {
            if socket.send(Message::Text(payload)).await.is_err() {
                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
