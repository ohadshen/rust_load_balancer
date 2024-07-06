use axum::async_trait;
use std::sync::{Arc, Mutex};

use crate::utils::time_utils::get_current_time;

use super::traits::IRateLimiter;

static MAX_WINDOW_REQUESTS: i32 = 10;
static WINDOW_DURATION: i64 = 10;

pub struct FixedWindowRateLimiter {
    current_requests: Arc<Mutex<i32>>,
    last_refill_time: Arc<Mutex<i64>>,
}

#[async_trait]
impl IRateLimiter for FixedWindowRateLimiter {
    async fn validate(&self) -> bool {
        if self.should_refill_window() {
            self.perform_window_refill();
        }

        let requests_value = self.get_requests();

        if requests_value < MAX_WINDOW_REQUESTS {
            self.set_requests(requests_value + 1);
            true
        } else {
            false
        }
    }

    async fn limiter_status(&self) -> String {
        let current_time = get_current_time();
        let last_refill_time = *self.last_refill_time.lock().unwrap();
        let time_since_last_refill = current_time - last_refill_time;

        format!(
            "Requests: {}, Of: {}, Time since last refill: {}",
            self.get_requests(),
            MAX_WINDOW_REQUESTS,
            time_since_last_refill
        )
    }
}

impl FixedWindowRateLimiter {
    pub fn new() -> Self {
        let current_requests = Arc::new(Mutex::new(0));
        let last_refill_time = Arc::new(Mutex::new(get_current_time()));

        FixedWindowRateLimiter {
            current_requests,
            last_refill_time,
        }
    }

    pub fn set_requests(&self, value: i32) {
        let mut requests = self.current_requests.lock().unwrap();
        *requests = value;
    }

    fn should_refill_window(&self) -> bool {
        let current_time = get_current_time();
        let time_since_last_refill = current_time - *self.last_refill_time.lock().unwrap();
        time_since_last_refill >= WINDOW_DURATION
    }

    fn perform_window_refill(&self) {
        let current_time = get_current_time();

        *self.current_requests.lock().unwrap() = 0;
        *self.last_refill_time.lock().unwrap() = current_time;

        println!(
            "Window reset. Requests reset to 0. Last refill time updated to {}.",
            current_time
        );
    }

    fn get_requests(&self) -> i32 {
        *self.current_requests.lock().unwrap()
    }
}
