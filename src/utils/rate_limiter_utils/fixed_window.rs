use axum::async_trait;
use std::sync::{Arc, Mutex};

use super::traits::IRateLimiter;

static MAX_WINDOW_REQUESTS: i32 = 10;
static WINDOW_DURATION: u64 = 10;

pub struct FixedWindowRateLimiter {
    current_requests: Arc<Mutex<i32>>,
}

#[async_trait]
impl IRateLimiter for FixedWindowRateLimiter {
    async fn validate(&self) -> bool {
        let requests_value = *self.current_requests.lock().unwrap();

        if requests_value < MAX_WINDOW_REQUESTS {
            self.set_requests(requests_value + 1);
            return true;
        } else {
            return false;
        }
    }

    async fn limiter_status(&self) -> String {
        let requests_value = *self.current_requests.lock().unwrap();
        format!("Requests: {}", requests_value)
    }
}

impl FixedWindowRateLimiter {
    pub fn new() -> Self {
        let requests = Arc::new(Mutex::new(0));
        let requests_clone = Arc::clone(&requests);

        tokio::spawn(async move {
            Self::move_to_next_window(requests_clone).await;
        });

        FixedWindowRateLimiter {
            current_requests: requests,
        }
    }

    async fn move_to_next_window(requests: Arc<Mutex<i32>>) {
        loop {
            let duration = tokio::time::Duration::from_secs(WINDOW_DURATION);
            tokio::time::sleep(duration).await;

            let mut requests_locked = requests.lock().unwrap();

            println!("window passed, request limit: {}", MAX_WINDOW_REQUESTS);
            if *requests_locked < MAX_WINDOW_REQUESTS {
                *requests_locked = MAX_WINDOW_REQUESTS;
            }
        }
    }

    pub fn set_requests(&self, value: i32) {
        let mut requests = self.current_requests.lock().unwrap();
        println!("set requests from {} to {}", *requests, value);
        *requests = value;
    }
}
