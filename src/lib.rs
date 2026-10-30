pub mod config;
pub mod replay_buffer;
pub mod encoder;
pub mod tray;
pub mod recorder;
pub mod capture;
pub mod settings_window;

pub type DefaultError = Box<dyn std::error::Error + Send + Sync>;