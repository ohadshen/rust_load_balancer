use std::sync::Arc;

mod models;
mod utils;

use axum::{routing::get, Router};
use utils::rate_limiter_utils::{
    fixed_window::FixedWindowRateLimiter, sliding_window::SlidingWindowRateLimiter,
    traits::IRateLimiter,
};

#[tokio::main]
async fn main() {
    // let limiter = Arc::new(LeakyBucketRateLimiter::new());
    // let limiter = Arc::new(TokenBucketLimiter::new());
    // let limiter = Arc::new(FixedWindowRateLimiter::new());
    let limiter = Arc::new(SlidingWindowRateLimiter::new());

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
