pub struct DatabaseConfig {
    pub db_user: String,
    pub db_passwd: String,
    pub db_url: String,
    pub db_port: String,
}

impl DatabaseConfig {
    pub fn new() -> DatabaseConfig {
        // Setting up the env variables for postgres db
        let postgres_db_user =
            std::env::var("POMEGRANATE_DB_USER").expect("POMEGRANATE_DB_USER must be set.");
        let postgres_db_pass =
            std::env::var("POMEGRANATE_DB_PWD").expect("POMEGRANATE_DB_PWD must be set.");
        let postgres_db_url =
            std::env::var("POMEGRANATE_DB_URL").expect("POMEGRANATE_DB_URL must be set.");
        let postgres_db_port =
            std::env::var("POMEGRANATE_DB_PORT").expect("POMEGRANATE_DB_PORT must be set.");

        return DatabaseConfig {
            db_user: postgres_db_user,
            db_passwd: postgres_db_pass,
            db_url: postgres_db_url,
            db_port: postgres_db_port,
        };
    }
}
