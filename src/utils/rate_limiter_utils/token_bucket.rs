use axum::async_trait;
use std::sync::{Arc, Mutex};

use crate::utils::time_utils::get_current_time;

use super::traits::IRateLimiter;

static MAX_DAILY_TOKENS: i64 = 10;
static REFRESH_BUCKET_TIME: i64 = 10;
static TOKENS_ADDED_ON_REFILL: i64 = 2;

pub struct TokenBucketLimiter {
    tokens: Arc<Mutex<i64>>,
    last_refill_time: Arc<Mutex<i64>>,
}

#[async_trait]
impl IRateLimiter for TokenBucketLimiter {
    async fn validate(&self) -> bool {
        let current_time = get_current_time();

        // Acquire locks and get values
        let mut tokens_value = *self.tokens.lock().unwrap();
        let last_refill_time = *self.last_refill_time.lock().unwrap();

        let time_since_last_refill = current_time - last_refill_time;
        let tokens_to_add = time_since_last_refill / REFRESH_BUCKET_TIME * TOKENS_ADDED_ON_REFILL;
        let refill_unused_delta = time_since_last_refill % REFRESH_BUCKET_TIME;

        if tokens_to_add > 0 {
            tokens_value += tokens_to_add;
            tokens_value = tokens_value.min(MAX_DAILY_TOKENS);

            // Update token bucket state
            self.set_tokens(tokens_value);
            *self.last_refill_time.lock().unwrap() = current_time - refill_unused_delta;
        }

        // Consume one token if available
        if tokens_value > 0 {
            self.set_tokens(tokens_value - 1);
            true
        } else {
            false
        }
    }

    async fn limiter_status(&self) -> String {
        let tokens_value = *self.tokens.lock().unwrap();
        format!("Tokens: {}", tokens_value)
    }
}

impl TokenBucketLimiter {
    pub fn new() -> Self {
        let tokens = Arc::new(Mutex::new(MAX_DAILY_TOKENS));
        let last_refill_time = Arc::new(Mutex::new(get_current_time()));

        TokenBucketLimiter {
            tokens,
            last_refill_time,
        }
    }

    pub fn set_tokens(&self, value: i64) {
        let mut tokens = self.tokens.lock().unwrap();
        println!("set tokens from {} to {}", *tokens, value);
        *tokens = value;
    }
}
