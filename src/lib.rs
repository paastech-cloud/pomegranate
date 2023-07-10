pub mod application;
mod config;
pub mod engine;
mod grpc_server;

use log::{error, info};
use std::error;

use crate::application::Application;
pub use crate::engine::docker_engine::DockerEngine;

pub async fn run_server() -> Result<(), Box<dyn error::Error>> {
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

    grpc_server::start_server(engine).await.map_err(|e| {
        error!("Failed to start gRPC server: {}", e);
        e
    })
}
