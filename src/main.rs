mod application;
mod config;
mod db;
mod engine;

use log::info;

use crate::application::Application;
use crate::db::Database;
use crate::engine::docker_engine::DockerEngine;

#[tokio::main]
async fn main() {
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

    let mut database = Database::new().await;

    info!(
        "Connecting to the PostgreSQL database: {:?}",
        database.create_table_accounts().await
    );
}
