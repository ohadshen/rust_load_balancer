use axum::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::utils::time_utils::get_current_time;

use super::traits::IRateLimiter;

static MAX_DAILY_TOKENS: i64 = 10;
static REFRESH_BUCKET_TIME: i64 = 10;
static TOKENS_ADDED_ON_REFILL: i64 = 2;

struct TokenBucketLimiterByIpModel {
    tokens: i64,
    last_refill_time: i64,
}

impl TokenBucketLimiterByIpModel {
    fn clone(&self) -> Self {
        TokenBucketLimiterByIpModel {
            tokens: self.tokens,
            last_refill_time: self.last_refill_time,
        }
    }

    pub fn new() -> Self {
        TokenBucketLimiterByIpModel {
            tokens: MAX_DAILY_TOKENS,
            last_refill_time: get_current_time(),
        }
    }
}

struct TokenBucketLimiterModel {
    map_by_ip: HashMap<String, TokenBucketLimiterByIpModel>,
}

impl TokenBucketLimiterModel {
    pub fn new() -> Self {
        TokenBucketLimiterModel {
            map_by_ip: HashMap::new(),
        }
    }

    pub fn get_tokens(&self, ip: &String) -> i64 {
        self.map_by_ip.get(ip).unwrap().tokens
    }

    pub fn set_tokens(&mut self, ip: &String, tokens: i64) {
        let mut value: TokenBucketLimiterByIpModel = self.map_by_ip.get(ip).unwrap().clone();

        println!("Setting tokens from {} to {}", value.tokens, tokens);
        value.tokens = tokens;
        self.map_by_ip.insert(ip.clone(), value);
    }

    pub fn get_last_refill_time(&self, ip: String) -> i64 {
        self.map_by_ip.get(&ip).unwrap().last_refill_time
    }

    pub fn set_last_refill_time(&mut self, ip: String, last_refill_time: i64) {
        let mut value: TokenBucketLimiterByIpModel = self.map_by_ip.get(&ip).unwrap().clone();

        println!(
            "Setting last_refill_time from {} to {}",
            value.last_refill_time, last_refill_time
        );
        value.last_refill_time = last_refill_time;
        self.map_by_ip.insert(ip, value);
    }
}

pub struct TokenBucketLimiter {
    token_bucket_limiter_model: TokenBucketLimiterModel,
}

#[async_trait]
impl IRateLimiter for TokenBucketLimiter {
    async fn validate(&mut self, ip: &String) -> bool {
        let current_time = get_current_time();

        // Acquire locks and get values
        if (self.token_bucket_limiter_model.map_by_ip.get(ip).is_none()) {
            println!("Inserting new ip: {}", ip);
            self.token_bucket_limiter_model
                .map_by_ip
                .insert(ip.clone(), TokenBucketLimiterByIpModel::new());
        }

        let mut tokens_value = self.token_bucket_limiter_model.get_tokens(&ip);
        let last_refill_time = self
            .token_bucket_limiter_model
            .get_last_refill_time(ip.clone());

        let time_since_last_refill = current_time - last_refill_time;
        let tokens_to_add = time_since_last_refill / REFRESH_BUCKET_TIME * TOKENS_ADDED_ON_REFILL;
        let refill_unused_delta = time_since_last_refill % REFRESH_BUCKET_TIME;

        println!("tokens to add:{}", tokens_to_add);
        if tokens_to_add > 0 {
            tokens_value += tokens_to_add;
            tokens_value = tokens_value.min(MAX_DAILY_TOKENS);

            // Update token bucket state
            self.token_bucket_limiter_model
                .set_tokens(&ip, tokens_value);
            self.token_bucket_limiter_model
                .set_last_refill_time(ip.clone(), current_time - refill_unused_delta);
        }

        // Consume one token if available
        if self.token_bucket_limiter_model.get_tokens(&ip) > 0 {
            self.token_bucket_limiter_model
                .set_tokens(&ip, tokens_value - 1);
            true
        } else {
            false
        }
    }

    async fn limiter_status(&self, ip: &String) -> String {
        let tokens_value = self.token_bucket_limiter_model.get_tokens(ip);
        format!("ip: {}, Tokens: {}", ip, tokens_value)
    }
}

impl TokenBucketLimiter {
    pub fn new() -> Self {
        TokenBucketLimiter {
            token_bucket_limiter_model: TokenBucketLimiterModel::new(),
        }
    }
}
