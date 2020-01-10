use rocket_contrib::databases::redis::Connection as RedisConnection;

use std::time::{SystemTime, UNIX_EPOCH};

use super::cache;

pub enum RateLimitError {
    TooManyRequests,
    Unknown(String),
}

use RateLimitError::*;

pub fn rate_limit(cache: &mut RedisConnection, ip: &str) -> Result<(), RateLimitError> {
    let minute = current_minute();
    let key = format!("{}:{}", ip, minute);

    if let Ok(counter) = cache::ratelimit_get(cache, &key) {
        if counter > 10 {
            return Err(TooManyRequests);
        }
    }

    cache::ratelimit_increment(cache, &key)
        .map_err(|err| Unknown(format!("Failed to increment_and_get for {}: {}", ip, err)))
}

fn current_minute() -> u64 {
    let now = SystemTime::now();
    let elapsed = now
        .duration_since(UNIX_EPOCH)
        .expect(&format!("Clock went backwards? Now={:?}", now));
    (elapsed.as_secs() / 60) % 60
}
