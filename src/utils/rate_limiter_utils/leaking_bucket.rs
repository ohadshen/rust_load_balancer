use axum::async_trait;
use lazy_static::lazy_static;
use std::time::{Duration, SystemTime};

use leaky_bucket::RateLimiter;

use super::traits::IRateLimiter;

lazy_static! {
    static ref LIMITER: RateLimiter = RateLimiter::builder()
        .initial(2)
        .max(10)
        .interval(Duration::from_millis(10000))
        .build();
}

pub struct LeakyBucketRateLimiter;

impl LeakyBucketRateLimiter {
    pub fn new() -> Self {
        // Perform any initialization logic if necessary
        LeakyBucketRateLimiter
    }
}

#[async_trait]
impl IRateLimiter for LeakyBucketRateLimiter {
    async fn validate(&self) -> bool {
        println!("{}", LIMITER.balance());
        LIMITER.acquire(1).await;
        return true;
    }

    async fn limiter_status(&self) -> String {
        format!("Requests: {}", LIMITER.balance())
    }
}
