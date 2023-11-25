use std::time::Duration;

const TIME_SHORT: u64 = 1000;

pub fn sleep_short() {
    std::thread::sleep(Duration::from_millis(TIME_SHORT));
}

pub fn sleep_time(time: u64) {
    std::thread::sleep(Duration::from_millis(time));
}
