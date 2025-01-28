use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::logging::LogConfig;

mod db;
pub mod manager;

pub use db::{load_settings, save_settings};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub steam_path: Option<String>,
    pub epic_path: Option<String>,
    pub auto_update: bool,
    pub update_interval: i64, // в часах
    pub notifications: bool,
    pub check_interval: i64,
    pub paths: GamePaths,
    // Steam авторизация
    pub steam_username: Option<String>,
    pub steam_password: Option<String>,
    // Системные настройки
    pub cache_ttl_minutes: i64,
    pub cache_size: usize,
    pub logging: LogConfig,
    pub custom_steamcmd_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePaths {
    pub steam: Option<String>,
    pub epic: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            steam_path: None,
            epic_path: None,
            auto_update: false,
            update_interval: 24,
            notifications: true,
            check_interval: 30,
            paths: GamePaths {
                steam: None,
                epic: None,
            },
            steam_username: None,
            steam_password: None,
            cache_ttl_minutes: 30,
            cache_size: 1000,
            logging: LogConfig::default(),
            custom_steamcmd_path: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SettingsUpdate {
    pub key: String,
    pub value: serde_json::Value,
}
