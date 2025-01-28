use std::path::PathBuf;
use std::sync::Arc;
use chrono::Local;
use directories::ProjectDirs;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer, Registry,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub file_name: String,
    pub rotation: LogRotation,
    pub custom_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogRotation {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_name: "app.log".to_string(),
            rotation: LogRotation::Daily,
            custom_path: None,
        }
    }
}

struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

#[derive(Clone)]
pub struct Logger {
    config: Arc<Mutex<LogConfig>>,
}

impl Logger {
    pub fn new(config: LogConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.lock();
        
        // Get log directory
        let log_dir = if let Some(custom_path) = &config.custom_path {
            custom_path.clone()
        } else {
            let project_dirs = ProjectDirs::from("com", "updateio", "app")
                .ok_or("Failed to get project directories")?;
            project_dirs.data_local_dir().join("logs")
        };

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&log_dir)?;

        // Set up file appender with rotation
        let rotation = match config.rotation {
            LogRotation::Minutely => Rotation::MINUTELY,
            LogRotation::Hourly => Rotation::HOURLY,
            LogRotation::Daily => Rotation::DAILY,
            LogRotation::Never => Rotation::NEVER,
        };

        let file_appender = RollingFileAppender::new(
            rotation,
            log_dir,
            &config.file_name,
        );

        // Parse log level
        let level = match config.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };

        // Create file layer
        let file_layer = fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(true)
            .with_timer(LocalTimer)
            .with_writer(file_appender)
            .with_filter(tracing_subscriber::filter::LevelFilter::from_level(level));

        // Create console layer
        let console_layer = fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(true)
            .with_timer(LocalTimer)
            .with_filter(tracing_subscriber::filter::LevelFilter::from_level(level));

        // Combine layers and set as global default
        Registry::default()
            .with(console_layer)
            .with(file_layer)
            .init();

        Ok(())
    }

    pub fn update_config(&self, new_config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.config.lock();
        *config = new_config;
        drop(config);
        self.init()
    }

    pub fn get_log_path(&self) -> Option<PathBuf> {
        let config = self.config.lock();
        config.custom_path.clone().or_else(|| {
            ProjectDirs::from("com", "updateio", "app")
                .map(|dirs| dirs.data_local_dir().join("logs").join(&config.file_name))
        })
    }
}

// Convenience macros for logging
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)+) => {
        tracing::error!($($arg)+)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)+) => {
        tracing::warn!($($arg)+)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)+) => {
        tracing::info!($($arg)+)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)+) => {
        tracing::debug!($($arg)+)
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)+) => {
        tracing::trace!($($arg)+)
    };
} 