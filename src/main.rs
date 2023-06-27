mod application;
mod db;
mod engine;

use dotenv::dotenv;
use log::info;

use crate::application::Application;
use crate::db::Database;
use crate::engine::docker_engine::DockerEngine;

#[tokio::main]
async fn main() {
    // Load env variables
    dotenv().ok();

    // Setting up the connection to postgres db
    let postgres_db_user =
        std::env::var("POMEGRANATE_DB_USER").expect("POMEGRANATE_DB_USER must be set.");
    let postgres_db_pass =
        std::env::var("POMEGRANATE_DB_PWD").expect("POMEGRANATE_DB_PWD must be set.");
    let postgres_db_url =
        std::env::var("POMEGRANATE_DB_URL").expect("POMEGRANATE_DB_URL must be set.");
    let postgres_db_port =
        std::env::var("POMEGRANATE_DB_PORT").expect("POMEGRANATE_DB_PORT must be set.");

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

    let database = Database::new(
        postgres_db_user,
        postgres_db_pass,
        postgres_db_url,
        postgres_db_port,
    );
    
    let connection_db = database.connect();

    info!("Connecting to the PostgreSQL database: {:?}", connection_db);
}
