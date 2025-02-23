use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapDuration {
    Short,
    Long,
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TapRange {
    pub(crate) min_duration: Duration,
    pub(crate) max_duration: Duration,
    pub(crate) tap_type: TapDuration,
}

impl TapRange {
    pub fn new(min_duration: Duration, max_duration: Duration, tap_type: TapDuration) -> Self {
        Self {
            min_duration,
            max_duration,
            tap_type,
        }
    }

    pub fn short() -> Self {
        Self::new(
            Duration::ZERO,
            Duration::from_millis(300),
            TapDuration::Short,
        )
    }

    pub fn long() -> Self {
        Self::new(
            Duration::from_millis(301),
            Duration::from_millis(u64::MAX),
            TapDuration::Long,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tap {
    pub(crate) duration: Duration,
    pub(crate) pressure: f32,
    pub(crate) tap_type: TapDuration,
}

impl PartialEq for Tap {
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration
            && self.pressure == other.pressure
            && self.tap_type == other.tap_type
    }
}
