use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager};
use tokio::process::Command;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::cache::{Cache, TimedCacheEntry};
use crate::error::{Result, Error};
use crate::registry::steam::SteamRegistry;
use crate::registry::RegistryReader;
use super::{Game, Platform, UpdateProgress, UpdateStatus};

// Структуры для десериализации JSON
#[derive(Debug, Serialize, Deserialize, Clone)]
struct SteamApp {
    appid: u32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SteamAppList {
    apps: Vec<SteamApp>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SteamAppListRoot {
    applist: SteamAppList,
}

// Глобальный кэш для списка игр
lazy_static::lazy_static! {
    static ref STEAM_APPS_CACHE: Cache<String, TimedCacheEntry<SteamAppListRoot>> = Cache::new(1);
}

fn get_steamcmd_path(app: &AppHandle) -> std::result::Result<PathBuf, tauri::Error> {
    app.path().resolve("resources/bin/steamcmd/steamcmd.exe", BaseDirectory::Resource)
}

fn get_steam_games_list_path(app: &AppHandle) -> std::result::Result<PathBuf, tauri::Error> {
    app.path().resolve("resources/bin/steamcmd/games_list.json", BaseDirectory::Resource)
}

pub(crate) async fn get_installed_games(app: &AppHandle) -> Result<Vec<Game>> {
    let registry = SteamRegistry::new()?;
    let games = registry.get_installed_games()?;
    
    let mut result = Vec::new();
    for (name, path) in games {
        // Получаем Steam ID игры
        if let Some(app_id) = get_steam_app_id(&name, app).await? {
            result.push(Game {
                id: app_id.to_string(),
                name,
                platform: Platform::Steam,
                install_path: path,
                last_update: None,
                update_status: None,
            });
        }
    }
    
    Ok(result)
}

pub(crate) async fn update_game(game_id: &str, app: &AppHandle) -> Result<()> {
    let steamcmd_path = get_steamcmd_path(app)?;

    if !steamcmd_path.exists() {
        return Err(Error::SteamCmdNotFound);
    }
    
    // Эмитим начало обновления
    app.emit("update-progress", UpdateProgress {
        game_id: game_id.to_string(),
        progress: 0.0,
        status: UpdateStatus {
            is_updating: true,
            progress: Some(0.0),
            error: None,
        },
        message: Some("Начало обновления...".to_string()),
    })?;

    // Запускаем обновление через SteamCMD
    let output = Command::new(&steamcmd_path)
        .args(&[
            "+login", "anonymous",
            "+app_update", game_id,
            "validate",
            "+quit"
        ])
        .output()
        .await
        .map_err(|e| Error::ProcessError(e.to_string()))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        app.emit("update-progress", UpdateProgress {
            game_id: game_id.to_string(),
            progress: 0.0,
            status: UpdateStatus {
                is_updating: false,
                progress: None,
                error: Some(error_msg.to_string()),
            },
            message: Some("Ошибка обновления".to_string()),
        })?;
        return Err(Error::UpdateFailed(error_msg.to_string()));
    }

    // Эмитим завершение обновления
    app.emit("update-progress", UpdateProgress {
        game_id: game_id.to_string(),
        progress: 100.0,
        status: UpdateStatus {
            is_updating: false,
            progress: Some(100.0),
            error: None,
        },
        message: Some("Обновление завершено".to_string()),
    })?;

    Ok(())
}

pub(crate) async fn check_updates(game_id: &str, app: &AppHandle) -> Result<bool> {
    let steamcmd_path = get_steamcmd_path(app)?;

    if !steamcmd_path.exists() {
        return Err(Error::SteamCmdNotFound);
    }

    let output = Command::new(&steamcmd_path)
        .args(&[
            "+login", "anonymous",
            "+app_info_update", "1",
            "+app_status", game_id,
            "+quit"
        ])
        .output()
        .await
        .map_err(|e| Error::ProcessError(e.to_string()))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Проверяем наличие обновлений по выводу steamcmd
    // Если в выводе есть "update available", значит есть обновление
    Ok(output_str.contains("update available"))
}

async fn get_steam_app_id(name: &str, app: &AppHandle) -> Result<Option<u32>> {
    // Проверяем кэш
    if let Some(cached) = STEAM_APPS_CACHE.get(&"steam_apps".to_string()) {
        if cached.timestamp > chrono::Utc::now() {
            // Ищем игру в кэшированном списке
            if let Some(app) = cached.value.applist.apps.iter().find(|app| app.name.to_lowercase() == name.to_lowercase()) {
                return Ok(Some(app.appid));
            }
            return Ok(None);
        }
    }

    // Если кэш устарел или пуст, читаем файл
    let games_list_path = get_steam_games_list_path(app)?;
    let content = fs::read_to_string(games_list_path)
        .map_err(|e| Error::FileError(format!("Failed to read games list: {}", e)))?;

    let app_list: SteamAppListRoot = serde_json::from_str(&content)
        .map_err(|e| Error::ParseError(format!("Failed to parse games list: {}", e)))?;

    // Кэшируем результат на 24 часа
    STEAM_APPS_CACHE.set(
        "steam_apps".to_string(),
        TimedCacheEntry::new(app_list.clone(), 24 * 60)
    );

    // Ищем игру в списке
    if let Some(app) = app_list.applist.apps.iter().find(|app| app.name.to_lowercase() == name.to_lowercase()) {
        Ok(Some(app.appid))
    } else {
        Ok(None)
    }
} 