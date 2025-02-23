use super::provider::KeyCode;
use super::KeyboardProvider;
use std::{format, println};
use wooting_analog_wrapper::{
    initialise, read_analog, set_device_event_cb, set_keycode_mode, DeviceEventType,
    DeviceInfo_FFI, KeycodeType,
};

pub struct WootingKeyboardProvider;

impl KeyboardProvider for WootingKeyboardProvider {
    fn read_key_pressure(&self, key: KeyCode) -> Result<f32, String> {
        let KeyCode::Wooting(code) = key;
        read_analog(code as u16)
            .0
            .map_err(|e| format!("Failed to read analog value: {:?}", e))
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
