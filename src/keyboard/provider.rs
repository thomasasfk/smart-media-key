use wooting_analog_wrapper::HIDCodes;

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
