use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use crate::error::Result;
use super::{Settings, db};

#[derive(Clone)]
pub struct SettingsManager {
    pool: Arc<Pool<Sqlite>>,
}

impl SettingsManager {
    pub async fn new() -> Result<Self> {
        let pool = db::init_database().await?;
        Ok(Self { pool: Arc::new(pool) })
    }

    pub async fn load(&self) -> Result<Settings> {
        db::load_settings(&self.pool).await
    }

    pub async fn save(&self, settings: &Settings) -> Result<()> {
        db::save_settings(&self.pool, settings).await
    }
} 