use std::time::Duration;

pub trait TimeDurationExt {
    fn as_minutes(&self) -> u64;
    fn as_seconds(&self) -> u64;
}

impl TimeDurationExt for Duration {
    fn as_minutes(&self) -> u64 {
        self.as_secs() / 60
    }

    fn as_seconds(&self) -> u64 {
        self.as_secs() % 60
    }
}