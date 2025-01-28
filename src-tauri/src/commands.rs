use tauri::command;
use tauri_plugin_dialog::DialogExt;
use crate::games::manager::GameManager;
use crate::games::Game;
use crate::settings::manager::SettingsManager;
use crate::settings::Settings;
use crate::registry::{steam::SteamRegistry, epic::EpicRegistry, RegistryReader};
use crate::error::Result;
use serde::Serialize;

#[command]
pub async fn get_installed_games(game_manager: tauri::State<'_, GameManager>) -> std::result::Result<Vec<Game>, String> {
    game_manager.get_installed_games().await.map_err(|e| e.to_string())
}

#[command]
pub async fn refresh_games_list(game_manager: tauri::State<'_, GameManager>) -> std::result::Result<Vec<Game>, String> {
    game_manager.refresh_games_list().await.map_err(|e| e.to_string())
}

#[command]
pub async fn update_game(game_id: String, game_manager: tauri::State<'_, GameManager>) -> std::result::Result<(), String> {
    game_manager.update_game(&game_id).await.map_err(|e| e.to_string())
}

#[command]
pub async fn check_game_updates(game_id: String, game_manager: tauri::State<'_, GameManager>) -> std::result::Result<bool, String> {
    game_manager.check_game_updates(&game_id).await.map_err(|e| e.to_string())
}

#[command]
pub async fn get_settings(settings_manager: tauri::State<'_, SettingsManager>) -> std::result::Result<Settings, String> {
    settings_manager.load().await.map_err(|e| e.to_string())
}

#[command]
pub async fn save_settings(settings: Settings, settings_manager: tauri::State<'_, SettingsManager>) -> std::result::Result<(), String> {
    settings_manager.save(&settings).await.map_err(|e| e.to_string())
}

#[command]
pub async fn select_directory(app: tauri::AppHandle) -> std::result::Result<String, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path);
    });

    match rx.await.unwrap_or(None) {
        Some(path) => match path.into_path() {
            Ok(path_buf) => Ok(path_buf.to_string_lossy().to_string()),
            Err(e) => Err(e.to_string()),
        },
        None => Err("No directory selected".to_string()),
    }
}

#[derive(Debug, Serialize)]
pub struct PathResponse {
    path: String,
}

#[derive(Debug, Serialize)]
pub struct PathsResponse {
    paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GameInfo {
    id: String,
    path: String,
}

#[derive(Debug, Serialize)]
pub struct GamesResponse {
    games: Vec<GameInfo>,
}

#[tauri::command]
pub async fn get_steam_install_path() -> Result<PathResponse> {
    let registry = SteamRegistry::new()?;
    let path = registry.get_install_path()?;
    Ok(PathResponse {
        path: path.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub async fn get_steam_library_folders() -> Result<PathsResponse> {
    let registry = SteamRegistry::new()?;
    let paths = registry.get_library_folders()?
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    Ok(PathsResponse { paths })
}

#[tauri::command]
pub async fn get_steam_installed_games() -> Result<GamesResponse> {
    let registry = SteamRegistry::new()?;
    let games = registry.get_installed_games()?
        .into_iter()
        .map(|(id, path)| GameInfo {
            id,
            path: path.to_string_lossy().into_owned(),
        })
        .collect();
    Ok(GamesResponse { games })
}

#[tauri::command]
pub async fn get_epic_install_path() -> Result<PathResponse> {
    let registry = EpicRegistry::new()?;
    let path = registry.get_install_path()?;
    Ok(PathResponse {
        path: path.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub async fn get_epic_library_folders() -> Result<PathsResponse> {
    let registry = EpicRegistry::new()?;
    let paths = registry.get_library_folders()?
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    Ok(PathsResponse { paths })
}

#[tauri::command]
pub async fn get_epic_installed_games() -> Result<GamesResponse> {
    let registry = EpicRegistry::new()?;
    let games = registry.get_installed_games()?
        .into_iter()
        .map(|(id, path)| GameInfo {
            id,
            path: path.to_string_lossy().into_owned(),
        })
        .collect();
    Ok(GamesResponse { games })
}

#[derive(Debug, Serialize)]
pub struct InstalledGame {
    id: String,
    path: String,
    platform: String,
}

#[derive(Debug, Serialize)]
pub struct AllGamesResponse {
    games: Vec<InstalledGame>,
}

#[tauri::command]
pub async fn get_all_installed_games() -> Result<AllGamesResponse> {
    let mut games = Vec::new();

    if let Ok(steam_registry) = SteamRegistry::new() {
        if let Ok(steam_games) = steam_registry.get_installed_games() {
            for (id, path) in steam_games {
                games.push(InstalledGame {
                    id,
                    path: path.to_string_lossy().into_owned(),
                    platform: "steam".to_string(),
                });
            }
        }
    }

    if let Ok(epic_registry) = EpicRegistry::new() {
        if let Ok(epic_games) = epic_registry.get_installed_games() {
            for (id, path) in epic_games {
                games.push(InstalledGame {
                    id,
                    path: path.to_string_lossy().into_owned(),
                    platform: "epic".to_string(),
                });
            }
        }
    }

    Ok(AllGamesResponse { games })
} 