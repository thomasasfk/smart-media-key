#![windows_subsystem = "windows"]

use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
use keyboard::WootingKeyboardProvider;
use parking_lot::Mutex;
use std::{sync::Arc, thread, time::Duration};
use tap::{KeyConfig, PatternDetector, TapDuration, TapSequence};
use tray_item::{IconSource, TrayItem};

#[cfg(feature = "wooting")]
use wooting_analog_wrapper::HIDCodes;

mod config;
mod keyboard;
mod tap;


fn setup_tray() -> TrayItem {
    let mut tray = TrayItem::new("Smart Media Key", IconSource::Resource("tray-icon"))
        .expect("Failed to create tray item");

    tray.add_menu_item("Quit", || std::process::exit(0))
        .expect("Failed to add quit menu item");

    tray
}

fn create_media_key_config(enigo: Arc<Mutex<Enigo>>) -> KeyConfig {
    #[cfg(feature = "wooting")]
    let mut config = KeyConfig::new(HIDCodes::F13);
    #[cfg(not(feature = "wooting"))]
    let mut config = KeyConfig::new(KeyCode::Raw(0x68)); // F13

    config.add_pattern(TapSequence::new(vec![TapDuration::Short]), {
        let enigo = enigo.clone();
        move || {
            let _ = enigo.lock().key(Key::MediaPlayPause, Click);
        }
    });

    config.add_pattern(TapSequence::new(vec![TapDuration::Short; 2]), {
        let enigo = enigo.clone();
        move || {
            let _ = enigo.lock().key(Key::MediaNextTrack, Click);
        }
    });

    config.add_pattern(TapSequence::new(vec![TapDuration::Short; 3]), {
        let enigo = enigo.clone();
        move || {
            let _ = enigo.lock().key(Key::MediaPrevTrack, Click);
        }
    });

    config.add_pattern(TapSequence::new(vec![TapDuration::Long]), move || {
        let _ = enigo.lock().key(Key::F14, Click);
    });

    config
}

fn main() {
    let _tray = setup_tray();

    #[cfg(feature = "wooting")]
    let keyboard_provider = WootingKeyboardProvider;
    #[cfg(not(feature = "wooting"))]
    let keyboard_provider = DefaultKeyboardProvider;

    let detector =
        PatternDetector::new(keyboard_provider).expect("Failed to create pattern detector");

    let enigo = Arc::new(Mutex::new(
        Enigo::new(&Settings::default()).expect("Failed to create Enigo"),
    ));

    let media_key_config = create_media_key_config(enigo);
    detector.add_key_config(media_key_config);

    detector.start();

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
