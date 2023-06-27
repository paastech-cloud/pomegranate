mod application;
mod engine;
mod server;

use log::{error, info};
use std::collections::HashMap;
use std::error;

use crate::application::Application;
use crate::engine::docker_engine::DockerEngine;
use crate::engine::engine::Engine;

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

    // TODO: Remove me
    // Start a sample application
    let sample_app = Application {
        project_id: String::from("test"),
        image_name: String::from("nginx"),
        env_variables: HashMap::from([(String::from("VERBOSITY"), String::from("5"))]),
    };

    engine.start_application(&sample_app);

    match server::start_server().await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to start gRPC server: {}", e);
            Err(e)
        }
    }
}
