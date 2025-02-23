use enigo::Key;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use wooting_analog_wrapper::HIDCodes;

#[derive(Serialize, Deserialize)]
pub struct Config {
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
