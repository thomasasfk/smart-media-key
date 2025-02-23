use super::state::KeyState;
use super::state::{KeyConfig, TapEvent};
use crate::keyboard::KeyboardProvider;
use crate::keyboard::KeyCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const POLLING_INTERVAL: Duration = Duration::from_millis(5);

pub struct PatternDetector {
    configs: Arc<Mutex<Vec<KeyConfig>>>,
    states: Arc<Mutex<HashMap<KeyCode, KeyState>>>,
    is_running: Arc<Mutex<bool>>,
    keyboard_provider: Arc<dyn KeyboardProvider>,
}

impl Drop for PatternDetector {
    fn drop(&mut self) {
        self.stop();
    }
}

impl PatternDetector {
    pub fn new(keyboard_provider: impl KeyboardProvider + 'static) -> Result<Self, String> {
        let provider = Arc::new(keyboard_provider);
        provider.initialize()?;

        Ok(PatternDetector {
            configs: Arc::new(Mutex::new(Vec::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(true)),
            keyboard_provider: provider,
        })
    }

    fn check_and_execute_patterns(config: &KeyConfig, state: &mut KeyState) {
        if let Some(pattern_action) = config.find_matching_pattern(&state.tap_events) {
            state.last_valid_pattern_index = Some(
                config
                    .patterns
                    .iter()
                    .position(|p| p.sequence.matches(&state.tap_events))
                    .unwrap(),
            );

            if !config.has_longer_patterns(&state.tap_events) {
                (pattern_action.action)();
                state.reset();
            }
        }
    }

    pub fn start(&self) -> thread::JoinHandle<()> {
        let configs = Arc::clone(&self.configs);
        let states = Arc::clone(&self.states);
        let is_running = Arc::clone(&self.is_running);
        let keyboard_provider = Arc::clone(&self.keyboard_provider);

        thread::spawn(move || {
            while *is_running.lock().unwrap() {
                let mut states = states.lock().unwrap();
                let configs = configs.lock().unwrap();

                for config in configs.iter() {
                    if let Ok(pressure) =
                        keyboard_provider.read_key_pressure(config.key_code.clone())
                    {
                        let state = states
                            .entry(config.key_code.clone())
                            .or_insert_with(KeyState::new);

                        let was_pressed = state.last_pressure > config.pressure_threshold;
                        let is_pressed = pressure > config.pressure_threshold;

                        match (was_pressed, is_pressed) {
                            (false, true) => {
                                state.last_pressed = Some(Instant::now());
                            }
                            (true, false) => {
                                if let Some(press_start) = state.last_pressed {
                                    let duration = press_start.elapsed();
                                    let tap = config.get_tap(duration, pressure);

                                    state.tap_events.push_back(TapEvent { tap });

                                    state.last_released = Some(Instant::now());
                                    Self::check_and_execute_patterns(config, state);
                                }
                            }
                            _ => {}
                        }

                        state.last_pressure = pressure;

                        if let Some(last_release) = state.last_released {
                            if let Some(press_start) = state.last_pressed {
                                if press_start <= last_release
                                    && last_release.elapsed() >= config.debounce_duration
                                {
                                    if let Some(pattern_idx) = state.last_valid_pattern_index {
                                        if let Some(pattern_action) =
                                            config.patterns.get(pattern_idx)
                                        {
                                            (pattern_action.action)();
                                        }
                                    }
                                    state.reset();
                                }
                            }
                        }
                    }
                }

                thread::sleep(POLLING_INTERVAL);
            }
        })
    }

    pub fn add_key_config(&self, config: KeyConfig) {
        let mut configs = self.configs.lock().unwrap();
        configs.push(config);
    }

    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }
}
