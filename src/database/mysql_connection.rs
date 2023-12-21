use std::env;
use diesel::{Connection, MysqlConnection};
use dotenv::dotenv;

pub fn mysql_connect() -> MysqlConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}