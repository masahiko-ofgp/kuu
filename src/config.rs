use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::app::KeyBindMode;


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub key_bind_mode: KeyBindMode,
    pub show_line_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_bind_mode: KeyBindMode::Vim,
            show_line_numbers: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = dirs::config_dir()
            .map(|p| p.join("kuu").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"));

        if !config_path.exists() {
            return Self::default();
        }
        
        match fs::read_to_string(&config_path) {
                Ok(content) => toml::from_str(&content)
                    .unwrap_or_else(|_| { Self::default() }),
                Err(_) => Self::default(),
            }
        }
}
