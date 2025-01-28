#[allow(dead_code)]
mod parser;

use self::parser::parse_update_status;
use crate::cache::{Cache, TimedCacheEntry};
use crate::error::{Result, Error};
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

const CACHE_SIZE: usize = 1000;
const CACHE_TTL_MINUTES: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    pub app_id: u32,
    pub name: String,
    pub install_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateStatus {
    pub progress: f32,
    pub status: String,
    pub state: UpdateState,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[allow(dead_code)]
pub enum UpdateState {
    Unknown,
    Starting,
    Downloading,
    Verifying,
    Extracting,
    Installing,
    Complete,
    Error,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone)]
pub struct SteamManager {
    steamcmd_path: PathBuf,
    update_cache: Arc<Cache<u32, TimedCacheEntry<bool>>>,
    settings: Arc<Settings>,
}

impl SteamManager {
    pub fn new(settings: Settings) -> Result<Self> {
        let steamcmd_path = if let Some(custom_path) = settings.custom_steamcmd_path.clone() {
            if std::fs::metadata(&custom_path).is_ok() {
                custom_path
            } else {
                Self::find_steamcmd()?
            }
        } else {
            Self::find_steamcmd()?
        };

        Ok(Self {
            steamcmd_path,
            update_cache: Arc::new(Cache::new(CACHE_SIZE)),
            settings: Arc::new(settings),
        })
    }

    fn find_steamcmd() -> Result<PathBuf> {
        let exe_dir = std::env::current_exe()
            .map_err(|_e| Error::SteamCmdNotFound)?
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or(Error::SteamCmdNotFound)?;

        let resource_path = exe_dir
            .join("resources")
            .join("bin")
            .join("steamcmd")
            .join("steamcmd.exe");

        if std::fs::metadata(&resource_path).is_ok() {
            info!("Found steamcmd at: {:?}", resource_path);
            Ok(resource_path)
        } else {
            error!("steamcmd not found at: {:?}", resource_path);
            Err(Error::SteamCmdNotFound)
        }
    }

    fn build_steam_command(&self) -> Command {
        let mut cmd = Command::new(&self.steamcmd_path);

        // Если есть учетные данные Steam, используем их
        if let Some(username) = &self.settings.steam_username {
            cmd.arg("+login");
            cmd.arg(username);
            if let Some(password) = &self.settings.steam_password {
                cmd.arg(password);
            }
        } else {
            // Иначе используем анонимный вход
            cmd.arg("+login");
            cmd.arg("anonymous");
        }

        cmd
    }

    pub fn get_installed_games(&self) -> Result<Vec<SteamGame>> {
        let mut cmd = self.build_steam_command();
        let output = cmd
            .arg("+apps_installed")
            .arg("+quit")
            .output()
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        parser::parse_installed_games(&String::from_utf8_lossy(&output.stdout))
    }

    pub fn check_for_updates(&self, app_id: u32) -> Result<bool> {
        // Сначала проверяем кэш
        if let Some(entry) = self.update_cache.get(&app_id) {
            if entry.timestamp > chrono::Utc::now() {
                info!("Using cached update status for app_id: {}", app_id);
                return Ok(entry.value);
            }
        }

        info!("Checking updates for app_id: {}", app_id);
        let mut cmd = self.build_steam_command();
        let output = cmd
            .args(&["+app_status", &app_id.to_string()])
            .arg("+quit")
            .output()
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        let needs_update = parser::check_update_needed(&String::from_utf8_lossy(&output.stdout))?;

        // Кэшируем результат
        info!("Caching update status for app_id: {}", app_id);
        self.update_cache.set(app_id, TimedCacheEntry::new(needs_update, CACHE_TTL_MINUTES));

        Ok(needs_update)
    }

    pub async fn update_game(
        &self,
        app_id: u32,
        progress_callback: impl Fn(UpdateStatus) + Send + 'static,
    ) -> Result<()> {
        let mut cmd = self.build_steam_command();
        let mut child = cmd
            .args(&["+app_update", &app_id.to_string(), "+quit"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        let stdout = child.stdout.take().unwrap();
        let reader = std::io::BufReader::new(stdout);
        let lines = std::io::BufRead::lines(reader);

        for line in lines {
            if let Ok(line) = line {
                if let Some((current, total)) = parser::parse_download_progress(&line) {
                    let progress = if total > 0 {
                        (current as f32 / total as f32) * 100.0
                    } else {
                        0.0
                    };

                    progress_callback(UpdateStatus {
                        progress,
                        status: line,
                        state: UpdateState::Downloading,
                        error: None,
                    });
                }
            }
        }

        let status = child
            .wait()
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        if status.success() {
            // Инвалидируем кэш после успешного обновления
            self.update_cache.invalidate(&app_id);
            progress_callback(UpdateStatus {
                progress: 100.0,
                status: "Update completed".to_string(),
                state: UpdateState::Complete,
                error: None,
            });
            Ok(())
        } else {
            Err(Error::ProcessError("Update failed".to_string()))
        }
    }

    pub async fn update_game_with_progress(
        &self,
        app_id: u32,
        progress_callback: impl Fn(UpdateStatus) + Send + 'static,
    ) -> Result<()> {
        info!("Starting update for app_id: {}", app_id);

        progress_callback(UpdateStatus {
            progress: 0.0,
            status: "Starting update...".to_string(),
            state: UpdateState::Starting,
            error: None,
        });

        let mut cmd = self.build_steam_command();
        let mut child = cmd
            .args(&["+app_update", &app_id.to_string(), "validate"])
            .arg("+quit")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let (tx, mut rx) = mpsc::channel(32);
        let tx2 = tx.clone();

        // Читаем stdout
        tokio::spawn(async move {
            let mut buffer = String::new();
            let mut reader = stdout_reader;
            while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
                if let Ok(status) = parse_update_status(&buffer) {
                    let _ = tx.send(status).await;
                }
                buffer.clear();
            }
        });

        // Читаем stderr
        tokio::spawn(async move {
            let mut buffer = String::new();
            let mut reader = stderr_reader;
            while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
                if buffer.contains("ERROR") {
                    let status = UpdateStatus {
                        state: UpdateState::Error,
                        error: Some(buffer.clone()),
                        ..Default::default()
                    };
                    let _ = tx2.send(status).await;
                }
                buffer.clear();
            }
        });

        // Получаем и обрабатываем статусы
        while let Some(status) = rx.recv().await {
            progress_callback(status.clone());

            if status.state == UpdateState::Error {
                error!(
                    "Error during update for app_id {}: {:?}",
                    app_id, status.error
                );
                return Err(Error::ProcessError(
                    status.error.unwrap_or_else(|| "Unknown error".into()),
                ));
            }

            if status.state == UpdateState::Complete {
                break;
            }
        }

        let status = child
            .wait()
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        if !status.success() {
            error!("Update process failed for app_id: {}", app_id);
            return Err(Error::ProcessError("Update process failed".into()));
        }

        // Инвалидируем кэш после успешного обновления
        self.update_cache.invalidate(&app_id);

        info!("Update completed successfully for app_id: {}", app_id);

        // Сообщаем о завершении
        progress_callback(UpdateStatus {
            progress: 100.0,
            status: "Update completed".to_string(),
            state: UpdateState::Complete,
            error: None,
        });

        Ok(())
    }

    pub fn prepare_update(
        &self,
        app_id: u32,
    ) -> Result<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Ok(Box::new(self.update_game_with_progress(app_id, |_| {})))
    }

    pub fn refresh_games_list(&self) -> Result<Vec<SteamGame>> {
        // Очищаем кэш перед обновлением списка
        // Note: The main Cache doesn't have clear() method, so we'll skip this for now
        // self.update_cache.clear();
        // Используем существующий метод для получения списка игр
        self.get_installed_games()
    }
}
