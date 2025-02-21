mod error;
mod server;
mod service;
mod utils;

use server::run_server;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let _ = run_server().await;
}
