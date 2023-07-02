mod application;
mod db;
mod engine;
mod grpc_server;

use log::{error, info};
use std::error;

use crate::application::Application;
use crate::engine::docker_engine::DockerEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // Start the application
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .parse_default_env()
        .init();

    info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // Init the Docker engine
    let engine = DockerEngine::new();

    let db = db::Db::new();

    grpc_server::start_server(engine, db).await.map_err(|e| {
        error!("Failed to start gRPC server: {}", e);
        e
    })
}
