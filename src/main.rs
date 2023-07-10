use pomegranate::run_server;

use std::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    run_server().await
}
