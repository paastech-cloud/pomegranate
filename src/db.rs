use tokio_postgres::{Client, Error, NoTls};

use crate::config::database_config::DatabaseConfig;

pub struct Database {
    client: Client,
}

impl Database {
    pub async fn new() -> Self {
        let configuration = DatabaseConfig::new();

        // Connect to postgresql client
        let connection_link = format!(
            "postgresql://{}:{}@{}:{}/pomegranate",
            configuration.db_user,
            configuration.db_passwd,
            configuration.db_url,
            configuration.db_port
        );

        let (client, connection) = tokio_postgres::connect(&connection_link, NoTls)
            .await
            .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Database { client: client }
    }

    pub async fn create_table_accounts(&mut self) -> Result<(), Error> {
        self.client
            .query(
                "CREATE TABLE IF NOT EXISTS ACCOUNTS (
            user_id serial PRIMARY KEY,
            username VARCHAR ( 50 ) UNIQUE NOT NULL,
            password VARCHAR ( 50 ) NOT NULL,
            email VARCHAR ( 255 ) UNIQUE NOT NULL
        );",
                &[],
            )
            .await?;

        return Ok(());
    }
}
