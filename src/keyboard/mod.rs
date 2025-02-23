mod provider;
#[cfg(feature = "wooting")]
mod wooting;
#[cfg(not(feature = "wooting"))]
pub use provider::DefaultKeyboardProvider;
pub use provider::{KeyCode, KeyboardProvider};
#[cfg(feature = "wooting")]
pub use wooting::WootingKeyboardProvider;
