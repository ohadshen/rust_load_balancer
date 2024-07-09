use chrono::Utc;
use tokio::time::{sleep, Duration};

pub async fn await_5_seconds() {
    let duration = Duration::from_secs(5);
    sleep(duration).await;
    println!("5 seconds have passed!");
}

pub fn get_current_time() -> i64 {
    let now = Utc::now();
    let seconds = now.timestamp() as i64;
    seconds
}
