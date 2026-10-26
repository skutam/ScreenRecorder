use std::fs;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct Config {
    recording: RecordingConfig,
    hotkeys: HotkeyConfig,
    ui: UiConfig,
}

#[derive(Debug, Deserialize)]
pub struct RecordingConfig {
    enabled: bool,
    duration_seconds: u64,
    fps: u32,
    bitrate_mbps: u32,
    output_directory: PathBuf,
    format: VideoFormat,
}

#[derive(Debug, Deserialize)]
pub struct HotkeyConfig {
    save_replay: String,
}

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    tray_color_idle: String,
    tray_color_saving: String,
    tray_color_error: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    Mp4,
    Mkv,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.recording.duration_seconds == 0 {
            return Err("recording.duration_seconds must be greater than 0".to_string());
        }

        if !(1..=120).contains(&self.recording.fps) {
            return Err("recording.fps must be between 1 and 120 inclusive".to_string());
        }

        if self.recording.bitrate_mbps == 0 {
            return Err("recording.bitrate_mbps must be greater than 0".to_string());
        }

        if self.recording.output_directory.as_os_str().is_empty() {
            return Err("recording.output_directory must not be empty".to_string());
        }

        Ok(())
    }

    pub fn recording_enabled(&self) -> bool {
        self.recording.enabled
    }

    pub fn replay_duration(&self) -> Duration {
        Duration::from_secs(self.recording.duration_seconds)
    }

    pub fn output_directory(&self) -> &Path {
        &self.recording.output_directory
    }
}

type ConfigError = Box<dyn std::error::Error + Send + Sync>;

pub struct ConfigHolder {
    config: Config,
}

impl ConfigHolder {
    pub fn new() -> Result<Self, ConfigError> {
        Ok(Self {
            config: load_config()?
        })
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn reload_config(&mut self) -> Result<(), ConfigError> {
        let config = load_config()?;
        self.config = config;
        Ok(())
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;

    config.validate()?;

    Ok(config)
}