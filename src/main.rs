use std::sync::Arc;

mod dal;
mod models;
mod utils;

use axum::{routing::get, Router};
use utils::rate_limiter_utils::{
    fixed_window::FixedWindowRateLimiter, leaking_bucket::LeakyBucketRateLimiter,
    sliding_window::SlidingWindowRateLimiter, token_bucket::TokenBucketLimiter,
    traits::IRateLimiter,
};

use dal::redis::get as redis_get;
use dal::redis::set as redis_set;

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
    // let limiter = Arc::new(TokenBucketLimiter::new());
    let limiter = Arc::new(FixedWindowRateLimiter::new());
    // let limiter = Arc::new(SlidingWindowRateLimiter::new());

    let app = Router::new().route(
        "/",
        get(move || async move {
            let limiter = Arc::clone(&limiter);
            if (limiter.validate().await) {
                "Welcome Ohad! \n Limiter Status: ".to_string() + &limiter.limiter_status().await
            } else {
                "Rate limit exceeded! \n Limiter Status: ".to_string()
                    + &limiter.limiter_status().await
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
