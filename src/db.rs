use postgres::{Client, Error, NoTls};

pub struct Database {
    user: String,
    pass: String,
    url: String,
    port: String,
}

impl Database {
    pub fn new(user: String, pass: String, url: String, port: String) -> Database {
        Database {
            user,
            pass,
            url,
            port,
        }
    }
    pub fn connect(&self) -> Result<(), Error> {
        // Connect to postgresql client
        let connection_link = format!(
            "postgresql://{}:{}@{}:{}/pomegranate",
            self.user, self.pass, self.url, self.port
        );
        println!("{:?}", connection_link);

        let mut client = Client::connect(&connection_link, NoTls)?;

        // Create table as an example
        client.batch_execute(
            "
            CREATE TABLE IF NOT EXISTS accounts (
                user_id serial PRIMARY KEY,
                username VARCHAR ( 50 ) UNIQUE NOT NULL,
                password VARCHAR ( 50 ) NOT NULL,
                email VARCHAR ( 255 ) UNIQUE NOT NULL
            );",
        )?;

        Ok(())
    }
}
