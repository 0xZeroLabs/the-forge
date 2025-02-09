mod server;
mod service;

use server::run_server;

#[tokio::main]
async fn main() {
    let _ = run_server().await;
}
