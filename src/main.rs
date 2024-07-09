use std::fmt::format;
use std::sync::Arc;

mod dal;
mod models;
mod utils;
use axum::response::IntoResponse;
use futures::future::BoxFuture;
// For returning a boxed future

use axum::handler::{self, Handler};
use axum::{routing::get, Router};
use tokio::sync::Mutex;
use utils::rate_limiter_utils::fixed_window::FixedWindowLimiter;
use utils::rate_limiter_utils::leaky_bucket::LeakyBucketLimiter;
use utils::rate_limiter_utils::sliding_window::{self, SlidingWindowLimiter};
use utils::rate_limiter_utils::token_bucket::TokenBucketLimiter;
use utils::rate_limiter_utils::{
    // fixed_window::FixedWindowRateLimiter,
    //  leaking_bucket::LeakyBucketRateLimiter,
    // sliding_window::SlidingWindowRateLimiter,
    traits::IRateLimiter,
};

use dal::redis::get as redis_get;
use dal::redis::set as redis_set;
use utils::time_utils::await_5_seconds;

#[tokio::main]
async fn main() {
    // let key = "ohad7";
    // let res1 = redis_get(key);
    // redis_set(key, "helloa");
    // let res2 = redis_get(key);

    // match res1 {
    //     Ok(ref v) => println!("Redis GET successful, value: {}", v),
    //     Err(ref e) => println!("Nill"),
    // }

    // match res2 {
    //     Ok(ref v) => println!("Redis GET successful, value: {}", v),
    //     Err(ref e) => println!("Redis GET error: {}", e),
    // }

    // let limiter = Arc::new(LeakyBucketRateLimiter::new());
    // let limiter = Arc::new(FixedWindowRateLimiter::new());
    // let limiter = Arc::new(SlidingWindowRateLimiter::new());
    // let limiter: TokenBucketLimiter = TokenBucketLimiter::new();
    use axum::{extract::Path, routing::get, Router};
    async fn verify_limiter<T>(path: Path<String>, limiter: Arc<Mutex<T>>) -> String
    where
        T: IRateLimiter,
    {
        let ip = &path.0;
        if (limiter.lock().await.validate(ip).await) {
            format!(
                "Hello world! limiter_status: {}",
                limiter.lock().await.limiter_status(ip).await
            )
        } else {
            format!(
                "OOOO NOOOO! limiter_status: {}",
                limiter.lock().await.limiter_status(ip).await
            )
        }
    }

    let limiter = Arc::new(Mutex::new(FixedWindowLimiter::new()));

    let app = Router::new().route(
        "/test/:ip",
        get(move |ip: Path<String>| verify_limiter(ip, limiter)),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
