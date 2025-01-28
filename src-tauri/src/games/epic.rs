use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use crate::error::{Result, Error};
use crate::registry::epic::EpicRegistry;
use crate::registry::RegistryReader;
use super::{Game, Platform, UpdateProgress, UpdateStatus};

pub(crate) async fn get_installed_games(_epic_path: &PathBuf) -> Result<Vec<Game>> {
    let registry = EpicRegistry::new()?;
    let games = registry.get_installed_games()?;
    
    Ok(games.into_iter().map(|(name, path)| Game {
        id: name.clone(),
        name,
        platform: Platform::Epic,
        install_path: path,
        last_update: None,
        update_status: None,
    }).collect())
}

pub(crate) async fn update_game(game_id: &str, app: &AppHandle) -> Result<()> {
    let registry = EpicRegistry::new()?;
    let launcher_path = registry.get_install_path()?;
    let epic_launcher = launcher_path.join("Launcher/Portal/Binaries/Win32/EpicGamesLauncher.exe");

    if !epic_launcher.exists() {
        return Err(Error::ConfigError("Epic Games Launcher not found".to_string()));
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

    // Запускаем обновление через Epic Games Launcher
    // Используем параметры командной строки для автоматического обновления
    let output = Command::new(&epic_launcher)
        .args(&[
            "-opengl", // Используем OpenGL рендеринг
            "-silent", // Тихий режим
            "-installupdate",
            game_id,
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

pub(crate) async fn check_updates(game_id: &str, _app: &AppHandle) -> Result<bool> {
    let registry = EpicRegistry::new()?;
    let launcher_path = registry.get_install_path()?;
    let epic_launcher = launcher_path.join("Launcher/Portal/Binaries/Win32/EpicGamesLauncher.exe");

    if !epic_launcher.exists() {
        return Err(Error::ConfigError("Epic Games Launcher not found".to_string()));
    }

    // Проверяем обновления через Epic Games Launcher
    let output = Command::new(&epic_launcher)
        .args(&[
            "-opengl",
            "-silent",
            "-checkforupdates",
            game_id,
        ])
        .output()
        .await
        .map_err(|e| Error::ProcessError(e.to_string()))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    Ok(output_str.contains("update available"))
} 