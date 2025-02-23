use super::sequence::{PatternAction, TapSequence};
use super::types::{Tap, TapRange};
use crate::keyboard::KeyCode;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct KeyConfig {
    pub(crate) key_code: KeyCode,
    tap_ranges: Vec<TapRange>,
    pub(crate) patterns: Vec<PatternAction>,
    pub(crate) debounce_duration: Duration,
    pub(crate) pressure_threshold: f32,
}

#[derive(Clone)]
pub struct TapEvent {
    pub(crate) tap: Tap,
}

#[derive(Clone)]
pub struct KeyState {
    pub(crate) tap_events: VecDeque<TapEvent>,
    pub(crate) last_pressure: f32,
    pub(crate) last_pressed: Option<Instant>,
    pub(crate) last_released: Option<Instant>,
    pub(crate) last_valid_pattern_index: Option<usize>,
}

impl KeyState {
    pub(crate) fn new() -> Self {
        Self {
            tap_events: VecDeque::new(),
            last_pressure: 0.0,
            last_pressed: None,
            last_released: None,
            last_valid_pattern_index: None,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.tap_events.clear();
        self.last_pressure = 0.0;
        self.last_pressed = None;
        self.last_released = None;
        self.last_valid_pattern_index = None;
    }
}

impl KeyConfig {
    pub fn new(key_code: impl Into<KeyCode>) -> Self {
        let default_ranges = vec![TapRange::short(), TapRange::long()];
        Self {
            key_code: key_code.into(),
            tap_ranges: default_ranges,
            patterns: Vec::new(),
            debounce_duration: Duration::from_millis(300),
            pressure_threshold: 0.0,
        }
    }

    pub fn with_tap_ranges(mut self, ranges: Vec<TapRange>) -> Self {
        self.tap_ranges = ranges;
        self
    }

    pub fn add_pattern<F>(&mut self, sequence: TapSequence, action: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.patterns.push(PatternAction {
            sequence,
            action: Arc::new(action),
        });
    }

    pub(crate) fn get_tap(&self, duration: Duration, pressure: f32) -> Tap {
        let range = self
            .tap_ranges
            .iter()
            .find(|r| duration >= r.min_duration && duration <= r.max_duration)
            .unwrap_or(&self.tap_ranges[0]);

        Tap {
            duration,
            pressure,
            tap_type: range.tap_type,
        }
    }

    pub(crate) fn find_matching_pattern(
        &self,
        events: &VecDeque<TapEvent>,
    ) -> Option<&PatternAction> {
        self.patterns
            .iter()
            .find(|pattern_action| pattern_action.sequence.matches(events))
    }

    pub(crate) fn has_longer_patterns(&self, current_events: &VecDeque<TapEvent>) -> bool {
        self.patterns.iter().any(|pattern_action| {
            pattern_action.sequence.sequence.len() > current_events.len()
                && pattern_action
                    .sequence
                    .sequence
                    .iter()
                    .zip(current_events.iter())
                    .all(|(expected, event)| *expected == event.tap.tap_type)
        })
    }
}
