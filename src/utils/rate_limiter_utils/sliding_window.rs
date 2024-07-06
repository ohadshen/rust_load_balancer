use axum::async_trait;
use std::sync::{Arc, Mutex};

use crate::utils::time_utils::get_current_time;

use super::traits::IRateLimiter;

static MAX_WINDOW_REQUESTS: i32 = 10;
static WINDOW_DURATION: i64 = 10;

pub struct SlidingWindowRateLimiter {
    current_requests: Arc<Mutex<i32>>,
    last_requests: Arc<Mutex<i32>>,
    window_start_time: Arc<Mutex<i64>>,
}

#[async_trait]
impl IRateLimiter for SlidingWindowRateLimiter {
    async fn validate(&self) -> bool {
        if self.should_refill_window() {
            self.perform_window_refill();
        }

        let requests_in_sliding_window = self.calculate_requests_in_sliding_window();

        let current_requests = *self.current_requests.lock().unwrap();

        if requests_in_sliding_window < MAX_WINDOW_REQUESTS {
            // Update current requests count
            self.set_requests(current_requests + 1);
            true
        } else {
            false
        }
    }

    async fn limiter_status(&self) -> String {
        let window_start_time = *self.window_start_time.lock().unwrap();
        let current_time = get_current_time();
        let time_in_current_window = current_time - window_start_time;

        // Calculate requests in the sliding window based on given formula for status
        let requests_in_sliding_window = self.calculate_requests_in_sliding_window();

        // print all struct values and requests in sliding window
        format!(
            "Requests: {}, Last Requests: {}, Time in Current Window: {}, Requests in Sliding Window: {}",
            *self.current_requests.lock().unwrap(),
            *self.last_requests.lock().unwrap(),
            time_in_current_window,
            requests_in_sliding_window
        )
    }
}

impl SlidingWindowRateLimiter {
    pub fn new() -> Self {
        let current_requests = Arc::new(Mutex::new(0));
        let last_requests = Arc::new(Mutex::new(0));
        let window_start_time = Arc::new(Mutex::new(get_current_time()));

        SlidingWindowRateLimiter {
            current_requests,
            last_requests,
            window_start_time,
        }
    }

    pub fn set_requests(&self, value: i32) {
        let mut requests = self.current_requests.lock().unwrap();
        *requests = value;
    }

    fn should_refill_window(&self) -> bool {
        let current_time = get_current_time();
        let time_since_last_refill = current_time - *self.window_start_time.lock().unwrap();
        time_since_last_refill >= WINDOW_DURATION
    }

    fn perform_window_refill(&self) {
        let mut window_start_time_locked = self.window_start_time.lock().unwrap();

        let current_time = get_current_time();

        let time_since_last_refill = current_time - *window_start_time_locked;
        let mut current_requests_locked = self.current_requests.lock().unwrap();
        let mut last_requests_locked = self.last_requests.lock().unwrap();

        if time_since_last_refill >= 2 * WINDOW_DURATION {
            *last_requests_locked = 0;
        } else {
            *last_requests_locked = *current_requests_locked;
        }

        *current_requests_locked = 0;
        let window_delta_to_reduce = time_since_last_refill % WINDOW_DURATION;
        *window_start_time_locked = current_time - window_delta_to_reduce;

        println!(
            "Window reset. Requests reset to 0. Last refill time updated to {}.",
            *window_start_time_locked
        );
    }

    fn calculate_requests_in_sliding_window(&self) -> i32 {
        let current_requests = *self.current_requests.lock().unwrap();
        let last_requests = *self.last_requests.lock().unwrap();
        let window_start_time = *self.window_start_time.lock().unwrap();
        let current_time = get_current_time();

        let time_in_current_window = current_time - window_start_time;

        let last_window_time_percentage =
            (WINDOW_DURATION as f64 - time_in_current_window as f64) / WINDOW_DURATION as f64;

        (last_window_time_percentage * last_requests as f64 + current_requests as f64) as i32
    }
}
