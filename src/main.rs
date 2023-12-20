mod redis_connection;

use redis::{Commands, RedisResult};
use redis_connection::{redis_connect};

fn main() {
    // connect to redis database
    let mut connection = redis_connect();
    // SET operation
    let _: RedisResult<()> = connection.set("first_name", "sohaib");
    let _: RedisResult<()> = connection.set("last_name", "manah");
    let _: RedisResult<()> = connection.set("age", 20);

    println!("inserted !");

    // DEL operation
    let deleted: RedisResult<bool> = connection.del("last_name");
    match deleted {
        Ok(true) => println!("Key deleted"),
        Ok(false) => println!("Key not found"),
        Err(err) => eprintln!("Error: {}", err),
    }

    // EXISTS operation
    let exists: RedisResult<bool> = connection.exists("first_name");
    match exists {
        Ok(true) => println!("Key exists"),
        Ok(false) => println!("Key does not exist"),
        Err(err) => eprintln!("Error: {}", err),
    }
}

