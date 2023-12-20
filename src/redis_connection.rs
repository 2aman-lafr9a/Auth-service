pub(crate) fn redis_connect() -> redis::Connection {
    let redis_host_name = "localhost:6379";
    let redis_password = "";

    let redis_conn_url = format!("{}://:{}@{}", "redis", redis_password, redis_host_name);

    let client = redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL");

    match client.get_connection() {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("Failed to connect to Redis: {}", err);
            std::process::exit(1);
        }
    }
}
