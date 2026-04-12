use dotenv::dotenv;
use redis::aio::ConnectionManager;
use std::env;

const TTL_SECS: u64 = 300;

pub async fn create_connection_manager() -> ConnectionManager {
    dotenv().ok();
    let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(url).expect("Invalid Redis URL");
    client
        .get_connection_manager()
        .await
        .expect("Failed to connect to Redis")
}

pub async fn get(conn: &mut ConnectionManager, key: &str) -> Option<String> {
    redis::cmd("GET")
        .arg(key)
        .query_async::<String>(conn)
        .await
        .ok()
}

pub async fn set(conn: &mut ConnectionManager, key: &str, value: &str) {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(TTL_SECS)
        .query_async::<()>(conn)
        .await
        .ok();
}

pub async fn del(conn: &mut ConnectionManager, key: &str) {
    redis::cmd("DEL")
        .arg(key)
        .query_async::<()>(conn)
        .await
        .ok();
}
