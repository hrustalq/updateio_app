use serde::{Serialize, Serializer};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

pub mod manager;
pub mod steam;
pub mod epic;

pub use manager::GameManager;

#[derive(Debug, Clone, Serialize)]
pub enum Platform {
    Steam,
    Epic,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatus {
    pub is_updating: bool,
    pub progress: Option<f32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgress {
    pub game_id: String,
    pub progress: f32,
    pub status: UpdateStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub platform: Platform,
    pub install_path: PathBuf,
    pub last_update: Option<DateTime<Utc>>,
    pub update_status: Option<UpdateStatus>,
}

impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Game", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("platform", &self.platform)?;
        state.serialize_field("install_path", &self.install_path.to_string_lossy())?;
        state.serialize_field("last_update", &self.last_update)?;
        state.serialize_field("update_status", &self.update_status)?;
        state.end()
    }
} 