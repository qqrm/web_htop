use axum::{extract::State, routing::get, Router, Server};
use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};
use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(get_cpu_load))
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

async fn get_cpu_load(State(state): State<AppState>) -> String {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();
    sys.cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| {
            let mut one_cpu_str = String::new();
            writeln!(&mut one_cpu_str, "CPU {} {}%", i + 1, cpu.cpu_usage());
            one_cpu_str
        })
        .fold(String::new(), |mut sum, one_cpu_data| {
            sum.push_str(one_cpu_data.as_str());
            sum
        })
}
