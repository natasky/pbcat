use std::{cell::LazyCell, sync::Mutex, time::Instant};

static FIRST_CHECK_INSTANT: Mutex<LazyCell<Instant>> = Mutex::new(LazyCell::new(|| Instant::now()));

pub fn get_update_count() -> u64 {
    // Fallback just checks contents every second.
    FIRST_CHECK_INSTANT.lock().unwrap().elapsed().as_secs()
}
