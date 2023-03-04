use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router, Server};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(root))
        .route("/index.js", get(index_js))
        .route("/api/cpus", get(get_cpu_load))
        .with_state(AppState {
            sys: Arc::new(Mutex::new(System::new())),
        });
    let server = Server::bind(&"0.0.0.0:15500".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");
    server.await.unwrap();
}

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>,
}

#[axum::debug_handler]
async fn root() -> impl IntoResponse {
    // Development solution, not for production
    let html = tokio::fs::read_to_string("src/index.html").await.unwrap();

    Html(html)
}

#[axum::debug_handler]
async fn index_js() -> impl IntoResponse {
    // Development solution, not for production
    let js = tokio::fs::read_to_string("src/index.js").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript")
        .body(js)
        .unwrap()
}

#[axum::debug_handler]
async fn get_cpu_load(State(state): State<AppState>) -> impl IntoResponse {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();

    let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    Json(v)
}
