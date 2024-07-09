use axum::async_trait;
use std::collections::HashMap;

use crate::utils::time_utils::get_current_time;

use super::traits::IRateLimiter;

static WINDOW_SIZE: i64 = 10;

struct SlidingWindowLimiterByIpModel {
    current_requests: i64,
    last_requests: i64,
    last_refill_time: i64,
}

impl SlidingWindowLimiterByIpModel {
    fn clone(&self) -> Self {
        SlidingWindowLimiterByIpModel {
            current_requests: self.current_requests,
            last_requests: self.last_requests,
            last_refill_time: self.last_refill_time,
        }
    }

    pub fn new() -> Self {
        SlidingWindowLimiterByIpModel {
            current_requests: 0,
            last_requests: 0,
            last_refill_time: get_current_time(),
        }
    }
}

struct SlidingWindowLimiterModel {
    map_by_ip: HashMap<String, SlidingWindowLimiterByIpModel>,
}

impl SlidingWindowLimiterModel {
    pub fn new() -> Self {
        SlidingWindowLimiterModel {
            map_by_ip: HashMap::new(),
        }
    }

    pub fn get_value(&self, ip: &String) -> &SlidingWindowLimiterByIpModel {
        self.map_by_ip.get(ip).unwrap()
    }

    fn add_request(&mut self, ip: &String) {
        let mut value = self.map_by_ip.get(ip).unwrap().clone();
        value.current_requests += 1;
        self.map_by_ip.insert(ip.clone(), value);
    }

    fn set_requests(&mut self, ip: &String, requests_to_set: i64) {
        let mut value = self.map_by_ip.get(ip).unwrap().clone();

        value.current_requests = requests_to_set;
        self.map_by_ip.insert(ip.clone(), value);
    }

    fn set_last_requests(&mut self, ip: &String, last_requests_to_set: i64) {
        let mut value = self.map_by_ip.get(ip).unwrap().clone();
        value.last_requests = last_requests_to_set;
        self.map_by_ip.insert(ip.clone(), value);
    }

    pub fn set_last_refill_time(&mut self, ip: String, last_refill_time: i64) {
        let mut value = self.map_by_ip.get(&ip).unwrap().clone();

        value.last_refill_time = last_refill_time;
        self.map_by_ip.insert(ip, value);
    }
}

pub struct SlidingWindowLimiter {
    limiter_model: SlidingWindowLimiterModel,
}

#[async_trait]
impl IRateLimiter for SlidingWindowLimiter {
    async fn validate(&mut self, ip: &String) -> bool {
        self.init_ip_in_map_if_needed(&ip);

        self.move_window_if_needed(&ip);

        let requests_in_sliding_window = self.calculate_requests_in_sliding_window(ip);

        if requests_in_sliding_window < WINDOW_SIZE {
            self.limiter_model.add_request(&ip);
            true
        } else {
            false
        }
    }

    async fn limiter_status(&self, ip: &String) -> String {
        let requests_value = self.limiter_model.get_value(ip).current_requests;
        let last_requests_value = self.limiter_model.get_value(ip).last_requests;
        let window_start_time = self.limiter_model.get_value(ip).last_refill_time;
        format!(
            "ip: {}, Requests: {}, Last Requests: {}, Window Start Time: {}",
            ip, requests_value, last_requests_value, window_start_time
        )
    }
}

impl SlidingWindowLimiter {
    fn calculate_requests_in_sliding_window(&self, ip: &String) -> i64 {
        let current_requests = self.limiter_model.get_value(ip).current_requests;
        let last_requests = self.limiter_model.get_value(ip).last_requests;
        let window_start_time = self.limiter_model.get_value(ip).last_refill_time;
        let current_time = get_current_time();

        let time_in_current_window = current_time - window_start_time;

        let last_window_time_percentage =
            (WINDOW_SIZE as f64 - time_in_current_window as f64) / WINDOW_SIZE as f64;

        (last_window_time_percentage * last_requests as f64 + current_requests as f64) as i64
    }

    fn init_ip_in_map_if_needed(&mut self, ip: &String) -> () {
        if (self.limiter_model.map_by_ip.get(ip).is_none()) {
            println!("Inserting new ip: {}", ip);
            self.limiter_model
                .map_by_ip
                .insert(ip.clone(), SlidingWindowLimiterByIpModel::new());
        }
    }

    fn move_window_if_needed(&mut self, ip: &String) -> () {
        let current_time: i64 = get_current_time();
        let current_values = self.limiter_model.get_value(ip);

        let time_since_last_refill = current_time - current_values.last_refill_time;
        let windows_passed = time_since_last_refill / WINDOW_SIZE;

        if windows_passed > 0 {
            self.limiter_model
                .set_last_requests(&ip, current_values.current_requests);
            self.limiter_model.set_requests(&ip, 0);

            let refill_unused_delta = time_since_last_refill % WINDOW_SIZE;
            self.limiter_model
                .set_last_refill_time(ip.clone(), current_time - refill_unused_delta);
        }
    }

    pub fn new() -> Self {
        SlidingWindowLimiter {
            limiter_model: SlidingWindowLimiterModel::new(),
        }
    }
}
