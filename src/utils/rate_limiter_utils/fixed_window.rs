use axum::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::utils::time_utils::get_current_time;

use super::traits::IRateLimiter;

static WINDOW_SIZE: i64 = 10;
struct FixedWindowLimiterByIpModel {
    requests: i64,
    last_refill_time: i64,
}

impl FixedWindowLimiterByIpModel {
    fn clone(&self) -> Self {
        FixedWindowLimiterByIpModel {
            requests: self.requests,
            last_refill_time: self.last_refill_time,
        }
    }

    pub fn new() -> Self {
        FixedWindowLimiterByIpModel {
            requests: 0,
            last_refill_time: get_current_time(),
        }
    }
}

struct FixedWindowLimiterModel {
    map_by_ip: HashMap<String, FixedWindowLimiterByIpModel>,
}

impl FixedWindowLimiterModel {
    pub fn new() -> Self {
        FixedWindowLimiterModel {
            map_by_ip: HashMap::new(),
        }
    }

    pub fn get_value(&self, ip: &String) -> &FixedWindowLimiterByIpModel {
        self.map_by_ip.get(ip).unwrap()
    }

    pub fn get_requests(&self, ip: &String) -> i64 {
        self.map_by_ip.get(ip).unwrap().requests
    }

    fn add_request(&mut self, ip: &String) {
        let mut value = self.map_by_ip.get(ip).unwrap().clone();
        value.requests += 1;
        self.map_by_ip.insert(ip.clone(), value);
    }

    fn set_requests(&mut self, ip: &String, requests_to_set: i64) {
        let mut value = self.map_by_ip.get(ip).unwrap().clone();

        println!(
            "Setting requests from {} to {}",
            value.requests, requests_to_set
        );
        value.requests = requests_to_set;
        self.map_by_ip.insert(ip.clone(), value);
    }

    pub fn get_last_refill_time(&self, ip: String) -> i64 {
        self.map_by_ip.get(&ip).unwrap().last_refill_time
    }

    pub fn set_last_refill_time(&mut self, ip: String, last_refill_time: i64) {
        let mut value = self.map_by_ip.get(&ip).unwrap().clone();

        println!(
            "Setting last_refill_time from {} to {}",
            value.last_refill_time, last_refill_time
        );
        value.last_refill_time = last_refill_time;
        self.map_by_ip.insert(ip, value);
    }
}

pub struct FixedWindowLimiter {
    limiter_model: FixedWindowLimiterModel,
}

#[async_trait]
impl IRateLimiter for FixedWindowLimiter {
    async fn validate(&mut self, ip: &String) -> bool {
        self.init_ip_in_map_if_needed(&ip);

        self.move_window_if_needed(&ip);

        if self.limiter_model.get_requests(&ip) < WINDOW_SIZE {
            self.limiter_model.add_request(&ip);
            true
        } else {
            false
        }
    }

    async fn limiter_status(&self, ip: &String) -> String {
        let requests_value = self.limiter_model.get_requests(ip);
        format!("ip: {},requests: {}", ip, requests_value)
    }
}

impl FixedWindowLimiter {
    fn init_ip_in_map_if_needed(&mut self, ip: &String) -> () {
        if (self.limiter_model.map_by_ip.get(ip).is_none()) {
            println!("Inserting new ip: {}", ip);
            self.limiter_model
                .map_by_ip
                .insert(ip.clone(), FixedWindowLimiterByIpModel::new());
        }
    }

    fn move_window_if_needed(&mut self, ip: &String) -> () {
        let current_time = get_current_time();
        let last_refill_time = self.limiter_model.get_last_refill_time(ip.clone());
        let time_since_last_refill = current_time - last_refill_time;
        let windows_passed: i64 = time_since_last_refill / WINDOW_SIZE;

        if windows_passed > 0 {
            self.limiter_model.set_requests(&ip, 0);

            let refill_unused_delta = time_since_last_refill % WINDOW_SIZE;
            self.limiter_model
                .set_last_refill_time(ip.clone(), current_time - refill_unused_delta);
        }
    }

    pub fn new() -> Self {
        FixedWindowLimiter {
            limiter_model: FixedWindowLimiterModel::new(),
        }
    }
}
