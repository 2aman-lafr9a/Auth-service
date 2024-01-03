use std::env;

pub(crate) fn redis_connect() -> redis::Connection {
    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set");


    let client = redis::Client::open(redis_url)
        .expect("Invalid connection URL");

    match client.get_connection() {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("Failed to connect to Redis: {}", err);
            std::process::exit(1);
        }
    }
}