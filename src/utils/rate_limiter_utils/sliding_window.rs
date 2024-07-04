use axum::async_trait;
use std::sync::{Arc, Mutex};

use super::traits::IRateLimiter;

use chrono::{Datelike, Timelike, Utc};

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
        let current_requests = *self.current_requests.lock().unwrap();
        let last_requests = *self.last_requests.lock().unwrap();
        let window_start_time = *self.window_start_time.lock().unwrap();

        let current_time = get_current_time();

        let time_in_current_window = current_time - window_start_time;
        let last_window_time_precentage: f64 =
            (WINDOW_DURATION as f64 - time_in_current_window as f64) / WINDOW_DURATION as f64;

        let requests_in_sliding_window: i32 =
            (last_window_time_precentage * last_requests as f64 + current_requests as f64) as i32;

        println!("last_requests:{}, current_requests:{}, last_window_time_precentage:{}, requests_in_sliding_window: {}",
                    last_requests,
                    current_requests,
                    last_window_time_precentage,
                    requests_in_sliding_window);

        if requests_in_sliding_window < MAX_WINDOW_REQUESTS {
            self.set_requests(current_requests + 1);
            return true;
        } else {
            return false;
        }
    }

    async fn limiter_status(&self) -> String {
        let current_requests = *self.current_requests.lock().unwrap();
        let last_requests = *self.last_requests.lock().unwrap();
        let window_start_time = *self.window_start_time.lock().unwrap();

        let current_time = get_current_time();

        let time_in_current_window: i64 = current_time - window_start_time;
        let last_window_time_precentage: f64 =
            (WINDOW_DURATION as f64 - time_in_current_window as f64) / WINDOW_DURATION as f64;

        let requests_in_sliding_window: i32 =
            (last_window_time_precentage * last_requests as f64 + current_requests as f64) as i32;

        // print all variables include requests_in_sliding_window
        format!("time_in_current_window:{},last_requests:{}, current_requests:{}, last_window_time_precentage:{}, requests_in_sliding_window: {}",
                    time_in_current_window,
                    last_requests,
                    current_requests,
                    last_window_time_precentage,
                    requests_in_sliding_window)
    }
}

fn get_current_time() -> i64 {
    let now = Utc::now();
    let seconds = now.timestamp();
    seconds
}

impl SlidingWindowRateLimiter {
    pub fn new() -> Self {
        let current_requests = Arc::new(Mutex::new(0));
        let last_requests = Arc::new(Mutex::new(0));
        let window_start_time = Arc::new(Mutex::new(get_current_time()));

        let instance: SlidingWindowRateLimiter = SlidingWindowRateLimiter {
            current_requests,
            last_requests,
            window_start_time,
        };

        let clone: SlidingWindowRateLimiter = SlidingWindowRateLimiter::get_clone(&instance);

        tokio::spawn(async move {
            Self::move_to_next_window(clone).await;
        });

        instance
    }

    fn get_clone(&self) -> SlidingWindowRateLimiter {
        let current_requests = Arc::clone(&self.current_requests);
        let last_requests = Arc::clone(&self.last_requests);
        let window_start_time = Arc::clone(&self.window_start_time);

        SlidingWindowRateLimiter {
            current_requests,
            last_requests,
            window_start_time,
        }
    }

    async fn move_to_next_window(limiter: SlidingWindowRateLimiter) {
        loop {
            let duration = tokio::time::Duration::from_secs(WINDOW_DURATION as u64);
            tokio::time::sleep(duration).await;

            let mut requests_locked = limiter.current_requests.lock().unwrap();
            let mut last_requests_locked = limiter.last_requests.lock().unwrap();
            let mut window_start_time_locked = limiter.window_start_time.lock().unwrap();

            println!(
                "window passed, last_requests: {}, request limit: {}",
                *last_requests_locked, MAX_WINDOW_REQUESTS
            );

            *window_start_time_locked = get_current_time();
            *last_requests_locked = *requests_locked;
            *requests_locked = 0;
        }
    }

    pub fn set_requests(&self, value: i32) {
        let mut tokens = self.current_requests.lock().unwrap();
        println!("set requests from {} to {}", *tokens, value);
        *tokens = value;
    }
}
