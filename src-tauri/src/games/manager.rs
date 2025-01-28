use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use crate::error::{Result, Error};
use crate::settings::Settings;
use std::path::PathBuf;
use super::{Game, UpdateProgress, steam, epic};

#[derive(Clone)]
pub struct GameManager {
    settings: Arc<Settings>,
    app: Arc<AppHandle>,
}

impl GameManager {
    pub fn new(settings: Settings, app: AppHandle) -> Self {
        Self {
            settings: Arc::new(settings),
            app: Arc::new(app),
        }
    }

    pub async fn get_installed_games(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();

        // Получаем игры Steam
        if let Some(_path) = &self.settings.paths.steam {
            let steam_games = steam::get_installed_games(&self.app).await?;
            games.extend(steam_games);
        }

        // Получаем игры Epic
        if let Some(path) = &self.settings.paths.epic {
            let epic_games = epic::get_installed_games(&PathBuf::from(path)).await?;
            games.extend(epic_games);
        }

        Ok(games)
    }

    pub async fn refresh_games_list(&self) -> Result<Vec<Game>> {
        self.get_installed_games().await
    }

    pub async fn update_game(&self, game_id: &str) -> Result<()> {
        let games = self.get_installed_games().await?;
        let game = games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

        match game.platform {
            super::Platform::Steam => {
                steam::update_game(game_id, &self.app.as_ref()).await?;
            }
            super::Platform::Epic => {
                epic::update_game(game_id, &self.app.as_ref()).await?;
            }
        }

        Ok(())
    }

    pub async fn check_game_updates(&self, game_id: &str) -> Result<bool> {
        let games = self.get_installed_games().await?;
        let game = games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

        match game.platform {
            super::Platform::Steam => {
                steam::check_updates(game_id, &self.app).await
            }
            super::Platform::Epic => {
                epic::check_updates(game_id, &self.app.as_ref()).await
            }
        }
    }

    pub fn emit_update_progress(&self, progress: UpdateProgress) -> Result<()> {
        self.app.as_ref()
            .emit("update-progress", &progress)
            .map_err(|e| Error::EmitError(e.to_string()))
    }
} 