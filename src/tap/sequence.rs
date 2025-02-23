use super::state::TapEvent;
use super::types::TapDuration;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TapSequence {
    pub(crate) sequence: Vec<TapDuration>,
}

impl TapSequence {
    pub fn new(sequence: Vec<TapDuration>) -> Self {
        Self { sequence }
    }

    pub(crate) fn matches(&self, events: &VecDeque<TapEvent>) -> bool {
        if events.len() != self.sequence.len() {
            return false;
        }
        events
            .iter()
            .zip(self.sequence.iter())
            .all(|(event, expected)| event.tap.tap_type == *expected)
    }
}

#[derive(Clone)]
pub struct PatternAction {
    pub(crate) sequence: TapSequence,
    pub(crate) action: Arc<dyn Fn() + Send + Sync + 'static>,
}
