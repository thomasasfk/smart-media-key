#![windows_subsystem = "windows"]
use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tray_item::TrayItem;

#[derive(Serialize, Deserialize)]
struct Config {
    #[cfg(feature = "wooting")]
    media_key: HIDCodes,
    #[cfg(not(feature = "wooting"))]
    media_key: u16, // Raw keycode
    play_pause_key: Key,
    next_track_key: Key,
    prev_track_key: Key,
    long_press_key: Key,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            #[cfg(feature = "wooting")]
            media_key: HIDCodes::F13,
            #[cfg(not(feature = "wooting"))]
            media_key: 0x68, // F13
            play_pause_key: Key::MediaPlayPause,
            next_track_key: Key::MediaNextTrack,
            prev_track_key: Key::MediaPrevTrack,
            long_press_key: Key::F14,
        }
    }
}

impl Config {
    fn load() -> Self {
        let config_path = get_config_path();
        match fs::read_to_string(&config_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                let config = Config::default();
                let _ = config.save();
                config
            }
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let config_path = get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(config_path, contents)
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("smart_media_key")
        .join("config.json")
}

#[cfg(feature = "wooting")]
use wooting_analog_wrapper::{
    initialise, read_analog, set_device_event_cb, set_keycode_mode, DeviceEventType,
    DeviceInfo_FFI, HIDCodes, KeycodeType,
};

const POLLING_INTERVAL: Duration = Duration::from_millis(5);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    #[cfg(feature = "wooting")]
    Wooting(HIDCodes),
    #[cfg(not(feature = "wooting"))]
    Raw(u16),
}

#[cfg(not(feature = "wooting"))]
impl From<u16> for KeyCode {
    fn from(code: u16) -> Self {
        KeyCode::Raw(code)
    }
}

#[cfg(feature = "wooting")]
impl From<HIDCodes> for KeyCode {
    fn from(code: HIDCodes) -> Self {
        KeyCode::Wooting(code)
    }
}

pub trait KeyboardProvider: Send + Sync {
    fn read_key_pressure(&self, key: KeyCode) -> Result<f32, String>;
    fn initialize(&self) -> Result<(), String>;
}

#[cfg(not(feature = "wooting"))]
pub struct DefaultKeyboardProvider;

#[cfg(not(feature = "wooting"))]
impl KeyboardProvider for DefaultKeyboardProvider {
    fn read_key_pressure(&self, key: KeyCode) -> Result<f32, String> {
        // Implementation would depend on the OS and available APIs
        // For now, we'll just return a basic pressed/not pressed state
        match key {
            KeyCode::Raw(code) => {
                // Here you would implement actual key detection logic
                // This is a placeholder that always returns "not pressed"
                Ok(0.0)
            }
            _ => Err("Invalid key type for keyboard".to_string()),
        }
    }

    fn initialize(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(feature = "wooting")]
pub struct WootingKeyboardProvider;

#[cfg(feature = "wooting")]
impl KeyboardProvider for WootingKeyboardProvider {
    fn read_key_pressure(&self, key: KeyCode) -> Result<f32, String> {
        match key {
            KeyCode::Wooting(code) => read_analog(code as u16)
                .0
                .map_err(|e| format!("Failed to read analog value: {:?}", e)),
            _ => Err("Invalid key type for Wooting keyboard".to_string()),
        }
    }

    fn initialize(&self) -> Result<(), String> {
        initialise()
            .0
            .map_err(|e| format!("Failed to initialize Wooting SDK: {:?}", e))?;
        set_keycode_mode(KeycodeType::HID)
            .0
            .map_err(|e| format!("Failed to set keycode mode: {:?}", e))?;

        extern "C" fn callback(event_type: DeviceEventType, _device_info: *mut DeviceInfo_FFI) {
            match event_type {
                DeviceEventType::Connected => println!("Wooting device connected"),
                DeviceEventType::Disconnected => println!("Wooting device disconnected"),
            }
        }

        set_device_event_cb(callback)
            .0
            .map_err(|e| format!("Failed to set device callback: {:?}", e))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapDuration {
    Short,
    Long,
    Custom(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TapRange {
    min_duration: Duration,
    max_duration: Duration,
    tap_type: TapDuration,
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
    duration: Duration,
    pressure: f32,
    tap_type: TapDuration,
}

impl PartialEq for Tap {
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration
            && self.pressure == other.pressure
            && self.tap_type == other.tap_type
    }
}

#[derive(Debug, Clone)]
pub struct TapSequence {
    sequence: Vec<TapDuration>,
}

impl TapSequence {
    pub fn new(sequence: Vec<TapDuration>) -> Self {
        Self { sequence }
    }

    fn matches(&self, events: &VecDeque<TapEvent>) -> bool {
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
struct PatternAction {
    sequence: TapSequence,
    action: Arc<dyn Fn() + Send + Sync + 'static>,
}

#[derive(Clone)]
pub struct KeyConfig {
    key_code: KeyCode,
    tap_ranges: Vec<TapRange>,
    patterns: Vec<PatternAction>,
    debounce_duration: Duration,
    pressure_threshold: f32,
}

#[derive(Debug, Clone)]
struct TapEvent {
    tap: Tap,
}

#[derive(Clone)]
struct KeyState {
    tap_events: VecDeque<TapEvent>,
    last_pressure: f32,
    last_pressed: Option<Instant>,
    last_released: Option<Instant>,
    last_valid_pattern_index: Option<usize>,
}

impl KeyState {
    fn new() -> Self {
        Self {
            tap_events: VecDeque::new(),
            last_pressure: 0.0,
            last_pressed: None,
            last_released: None,
            last_valid_pattern_index: None,
        }
    }

    fn reset(&mut self) {
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

    fn get_tap(&self, duration: Duration, pressure: f32) -> Tap {
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

    fn find_matching_pattern(&self, events: &VecDeque<TapEvent>) -> Option<&PatternAction> {
        self.patterns
            .iter()
            .find(|pattern_action| pattern_action.sequence.matches(events))
    }

    fn has_longer_patterns(&self, current_events: &VecDeque<TapEvent>) -> bool {
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

pub struct PatternDetector {
    configs: Arc<Mutex<Vec<KeyConfig>>>,
    states: Arc<Mutex<HashMap<KeyCode, KeyState>>>,
    is_running: Arc<Mutex<bool>>,
    keyboard_provider: Arc<dyn KeyboardProvider>,
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

impl Drop for PatternDetector {
    fn drop(&mut self) {
        self.stop();
    }
}

fn main() {
    let mut tray = TrayItem::new(
        "Smart Media Key",
        tray_item::IconSource::Resource("tray-icon"),
    )
    .unwrap();

    tray.add_menu_item("Quit", move || {
        std::process::exit(0);
    })
    .unwrap();

    #[cfg(feature = "wooting")]
    let keyboard_provider = WootingKeyboardProvider;
    #[cfg(not(feature = "wooting"))]
    let keyboard_provider = DefaultKeyboardProvider;

    let detector =
        PatternDetector::new(keyboard_provider).expect("Failed to create pattern detector");

    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));

    #[cfg(feature = "wooting")]
    let mut media_key_config = KeyConfig::new(HIDCodes::F13);
    #[cfg(not(feature = "wooting"))]
    let mut media_key_config = KeyConfig::new(KeyCode::Raw(0x68)); // F13

    let enigo_play = enigo.clone();
    media_key_config.add_pattern(TapSequence::new(vec![TapDuration::Short]), move || {
        let _ = enigo_play.lock().unwrap().key(Key::MediaPlayPause, Click);
    });

    let enigo_next = enigo.clone();
    media_key_config.add_pattern(
        TapSequence::new(vec![TapDuration::Short, TapDuration::Short]),
        move || {
            let _ = enigo_next.lock().unwrap().key(Key::MediaNextTrack, Click);
        },
    );

    let enigo_prev = enigo.clone();
    media_key_config.add_pattern(
        TapSequence::new(vec![
            TapDuration::Short,
            TapDuration::Short,
            TapDuration::Short,
        ]),
        move || {
            let _ = enigo_prev.lock().unwrap().key(Key::MediaPrevTrack, Click);
        },
    );

    let enigo_prev = enigo.clone();
    media_key_config.add_pattern(TapSequence::new(vec![TapDuration::Long]), move || {
        let _ = enigo_prev.lock().unwrap().key(Key::F14, Click);
    });

    detector.add_key_config(media_key_config);
    let _ = detector.start();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
