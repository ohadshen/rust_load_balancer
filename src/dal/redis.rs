use lazy_static::lazy_static;
use redis::Commands;
use std::sync::Mutex;

lazy_static! {
    static ref CLIENT: Mutex<redis::Client> =
        Mutex::new(redis::Client::open("redis://127.0.0.1/").unwrap());
}

pub fn get_connection() -> redis::RedisResult<redis::Connection> {
    let connection = CLIENT.lock().unwrap().get_connection()?;
    Ok(connection)
}

pub fn set(key: &str, value: &str) -> redis::RedisResult<()> {
    let connection = get_connection()?;
    connection.set(key, value)
}

pub fn get(key: &str) -> redis::RedisResult<String> {
    let connection = get_connection()?;
    return connection.get(key);
}
