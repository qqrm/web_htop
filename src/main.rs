use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router, Server};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;

type Snapshot = Vec<f32>;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Snapshot>(1);

    let app_state = AppState {
        tx: tx.clone(),
        cpus: Arc::new(Mutex::new(vec![])),
    };

    let router = Router::new()
        .route("/", get(root))
        .route("/index.mjs", get(index_mjs))
        .route("/index.css", get(index_css))
        .route("/api/cpus", get(get_cpu_load))
        .route("/rt/cpus", get(rt_get_cpu_load))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let _ = tx.send(v);

            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let server = Server::bind(&"0.0.0.0:8081".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{addr}");
    server.await.unwrap();
}

#[derive(Clone)]
struct AppState {
    cpus: Arc<Mutex<Vec<f32>>>,
    tx: broadcast::Sender<Snapshot>,
}

#[axum::debug_handler]
async fn root() -> impl IntoResponse {
    let html = tokio::fs::read_to_string("src/index.html").await.unwrap();
    Html(html)
}

#[axum::debug_handler]
async fn index_mjs() -> impl IntoResponse {
    let js = tokio::fs::read_to_string("src/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript")
        .body(js)
        .unwrap()
}

#[axum::debug_handler]
async fn index_css() -> impl IntoResponse {
    let css = tokio::fs::read_to_string("src/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css")
        .body(css)
        .unwrap()
}

#[axum::debug_handler]
async fn get_cpu_load(State(state): State<AppState>) -> impl IntoResponse {
    let v = state.cpus.lock().unwrap().clone();
    Json(v)
}

#[axum::debug_handler]
async fn rt_get_cpu_load(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |ws: WebSocket| async { rt_get_cpus_stream(state, ws).await })
}

async fn rt_get_cpus_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
