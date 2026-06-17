use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub speed_ms: u64,
    pub board_size: String,
    pub theme: String,
    pub high_score: usize,
    #[serde(default)]
    pub num_apples: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed_ms: 26,
            board_size: "Auto".to_string(),
            theme: "Classic".to_string(),
            high_score: 0,
            num_apples: 1,
        }
    }
}

impl Config {
    fn default_config() -> Self {
        Self {
            speed_ms: 26,
            board_size: "Auto".to_string(),
            theme: "Classic".to_string(),
            high_score: 0,
            num_apples: 1,
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        Self::load_from_path(&path)
    }

    /// Parse config from a TOML string. Returns defaults on parse error.
    pub fn load_from_str(s: &str) -> Self {
        toml::from_str::<Self>(s).unwrap_or_else(|_| Self::default_config())
    }

    fn load_from_path(path: &Path) -> Self {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str::<Self>(&contents) {
                return config;
            }
        }
        Self::default_config()
    }

    pub fn save(&self) {
        self.save_to_path(&Self::config_path())
    }

    /// Save config to an arbitrary path. Creates parent directories if needed.
    pub fn save_to_path(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(contents) = toml::to_string(self) {
            let _ = fs::write(path, contents);
        }
    }

    /// Save config from individual values.
    pub fn save_values(speed_ms: u64, board_size: &str, theme: &str, high_score: usize, num_apples: usize) {
        Self {
            speed_ms,
            board_size: board_size.to_string(),
            theme: theme.to_string(),
            high_score,
            num_apples,
        }
        .save();
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".config");
        path.push("snek");
        path.push("config.toml");
        path
    }
}
