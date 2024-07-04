use axum::async_trait;

#[async_trait]
pub trait IRateLimiter {
    async fn validate(&self) -> bool;
    async fn limiter_status(&self) -> String;
}
