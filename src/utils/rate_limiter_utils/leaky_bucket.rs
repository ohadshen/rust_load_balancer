use super::traits::IRateLimiter;
use axum::async_trait;
use leaky_bucket::RateLimiter;
use std::collections::HashMap;
use std::time::Duration;

struct LeakyBucketLimiterByIpModel {
    limiter_for_ip: RateLimiter,
}

impl LeakyBucketLimiterByIpModel {
    pub fn new() -> Self {
        LeakyBucketLimiterByIpModel {
            limiter_for_ip: RateLimiter::builder()
                .initial(2)
                .max(10)
                .interval(Duration::from_millis(10000))
                .build(),
        }
    }
}

struct LeakyBucketLimiterModel {
    map_by_ip: HashMap<String, LeakyBucketLimiterByIpModel>,
}

impl LeakyBucketLimiterModel {
    pub fn new() -> Self {
        LeakyBucketLimiterModel {
            map_by_ip: HashMap::new(),
        }
    }

    pub fn get_value(&self, ip: &String) -> &LeakyBucketLimiterByIpModel {
        self.map_by_ip.get(ip).unwrap()
    }
}

pub struct LeakyBucketLimiter {
    limiter_model: LeakyBucketLimiterModel,
}

#[async_trait]
impl IRateLimiter for LeakyBucketLimiter {
    async fn validate(&mut self, ip: &String) -> bool {
        self.init_ip_in_map_if_needed(&ip);

        self.limiter_model
            .get_value(ip)
            .limiter_for_ip
            .acquire(1)
            .await;
        return true;
    }

    async fn limiter_status(&self, ip: &String) -> String {
        format!(
            "ip: {}, balance: {}",
            ip,
            self.limiter_model.get_value(ip).limiter_for_ip.balance(),
        )
    }
}

impl LeakyBucketLimiter {
    pub fn new() -> Self {
        LeakyBucketLimiter {
            limiter_model: LeakyBucketLimiterModel::new(),
        }
    }

    fn init_ip_in_map_if_needed(&mut self, ip: &String) -> () {
        if (self.limiter_model.map_by_ip.get(ip).is_none()) {
            println!("Inserting new ip: {}", ip);
            self.limiter_model
                .map_by_ip
                .insert(ip.clone(), LeakyBucketLimiterByIpModel::new());
        }
    }
}
