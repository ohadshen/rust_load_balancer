use axum::async_trait;
use std::{
    cmp::min,
    sync::{Arc, Mutex},
};

use super::traits::IRateLimiter;

static MAX_DAILY_TOKENS: i32 = 10;
static REFRESH_BUCKET_TIME: u64 = 10;
static TOKENS_ADDED_ON_REFILL: i32 = 2;

pub struct TokenBucketLimiter {
    tokens: Arc<Mutex<i32>>,
}

#[async_trait]
impl IRateLimiter for TokenBucketLimiter {
    async fn validate(&self) -> bool {
        let tokens_value = *self.tokens.lock().unwrap();

        if tokens_value > 0 {
            self.set_tokens(tokens_value - 1);
            return true;
        } else {
            return false;
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
        let tokens_clone = Arc::clone(&tokens);

        tokio::spawn(async move {
            Self::token_bucket_refill(tokens_clone).await;
        });

        TokenBucketLimiter { tokens }
    }

    async fn token_bucket_refill(tokens: Arc<Mutex<i32>>) {
        loop {
            let duration = tokio::time::Duration::from_secs(REFRESH_BUCKET_TIME);
            tokio::time::sleep(duration).await;

            let mut tokens_locked = tokens.lock().unwrap();

            println!("{}, refill to {}", *tokens_locked, MAX_DAILY_TOKENS);
            if *tokens_locked < MAX_DAILY_TOKENS {
                //max on rust

                *tokens_locked += TOKENS_ADDED_ON_REFILL;
                *tokens_locked = min(*tokens_locked, MAX_DAILY_TOKENS);
            }
        }
    }

    pub fn set_tokens(&self, value: i32) {
        let mut tokens = self.tokens.lock().unwrap();
        println!("set tokens from {} to {}", *tokens, value);
        *tokens = value;
    }
}
