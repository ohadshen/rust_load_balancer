use tokio::time::{sleep, Duration};

pub async fn await_5_seconds() {
    let duration = Duration::from_secs(5);
    sleep(duration).await;
    println!("5 seconds have passed!");
}
