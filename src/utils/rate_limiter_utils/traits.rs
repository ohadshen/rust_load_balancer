use axum::async_trait;

#[async_trait]
pub trait IRateLimiter {
    async fn validate(&mut self, ip: &String) -> bool;
    async fn limiter_status(&self, ip: &String) -> String;
}
