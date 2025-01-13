mod lib;

use lib::upload_file;

#[tokio::main]
async fn main() {
    upload_file().await;
}
