use axum::response::{Html, IntoResponse, Response};
use axum::{extract::State, routing::get, Json, Router, Server};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let app_state = AppState::default();

    let router = Router::new()
        .route("/", get(root))
        .route("/index.mjs", get(index_mjs))
        .route("/index.css", get(index_css))
        .route("/api/cpus", get(get_cpu_load))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

            {
                let mut cpus = app_state.cpus.lock().unwrap();
                *cpus = v;
            }

            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let server = Server::bind(&"0.0.0.0:8081".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{addr}");
    server.await.unwrap();
}

#[derive(Default, Clone)]
struct AppState {
    cpus: Arc<Mutex<Vec<f32>>>,
}

#[axum::debug_handler]
async fn root() -> impl IntoResponse {
    // Development solution, not for production
    let html = tokio::fs::read_to_string("src/index.html").await.unwrap();

    Html(html)
}

#[axum::debug_handler]
async fn index_mjs() -> impl IntoResponse {
    // Development solution, not for production
    let js = tokio::fs::read_to_string("src/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript")
        .body(js)
        .unwrap()
}

#[axum::debug_handler]
async fn index_css() -> impl IntoResponse {
    // Development solution, not for production
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
