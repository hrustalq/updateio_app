use sqlx::{Pool, Sqlite, Row};
use directories::ProjectDirs;
use crate::error::{Result, Error};
use crate::logging::LogRotation;
use crate::log_info;
use super::Settings;

pub async fn init_database() -> Result<Pool<Sqlite>> {
    let project_dirs = ProjectDirs::from("com", "updateio", "app")
        .ok_or_else(|| Error::Database("Failed to get app data directory".to_string()))?;

    let db_dir = project_dirs.data_local_dir();
    std::fs::create_dir_all(db_dir)
        .map_err(|e| Error::Database(format!("Failed to create data directory: {}", e)))?;

    let db_path = db_dir.join("settings.db");
    
    // Создаем пустой файл базы данных, если его нет
    if !db_path.exists() {
        std::fs::File::create(&db_path)
            .map_err(|e| Error::Database(format!("Failed to create database file: {}", e)))?;
    }

    log_info!("Database path: {}", db_path.display());
    
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&format!("sqlite:{}", db_path.to_string_lossy()))
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    // Создаем таблицу, если её нет
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;

    Ok(pool)
}

pub async fn load_settings(pool: &Pool<Sqlite>) -> Result<Settings> {
    let mut settings = Settings::default();

    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    for row in rows {
        let key: String = row.get(0);
        let value: String = row.get(1);
        match key.as_str() {
            "auto_update" => settings.auto_update = value.parse().unwrap_or(false),
            "notifications" => settings.notifications = value.parse().unwrap_or(true),
            "check_interval" => settings.check_interval = value.parse().unwrap_or(30),
            "steam_path" => settings.paths.steam = Some(value),
            "epic_path" => settings.paths.epic = Some(value),
            "cache_ttl_minutes" => settings.cache_ttl_minutes = value.parse().unwrap_or(30),
            "cache_size" => settings.cache_size = value.parse().unwrap_or(1000),
            "log_level" => settings.logging.level = value,
            "log_file_name" => settings.logging.file_name = value,
            "log_rotation" => {
                settings.logging.rotation = match value.as_str() {
                    "minutely" => LogRotation::Minutely,
                    "hourly" => LogRotation::Hourly,
                    "daily" => LogRotation::Daily,
                    "never" => LogRotation::Never,
                    _ => LogRotation::Daily,
                }
            },
            "log_custom_path" => settings.logging.custom_path = Some(value.into()),
            "custom_steamcmd_path" => settings.custom_steamcmd_path = Some(value.into()),
            "steam_username" => settings.steam_username = Some(value),
            "steam_password" => settings.steam_password = Some(value),
            _ => {}
        }
    }

    Ok(settings)
}

pub async fn save_settings(pool: &Pool<Sqlite>, settings: &Settings) -> Result<()> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    // Очищаем старые настройки
    sqlx::query("DELETE FROM settings")
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    // Сохраняем базовые настройки
    let settings_to_save = [
        ("auto_update", settings.auto_update.to_string()),
        ("notifications", settings.notifications.to_string()),
        ("check_interval", settings.check_interval.to_string()),
        ("cache_ttl_minutes", settings.cache_ttl_minutes.to_string()),
        ("cache_size", settings.cache_size.to_string()),
        ("log_level", settings.logging.level.clone()),
        ("log_file_name", settings.logging.file_name.clone()),
        (
            "log_rotation",
            match settings.logging.rotation {
                LogRotation::Minutely => "minutely",
                LogRotation::Hourly => "hourly",
                LogRotation::Daily => "daily",
                LogRotation::Never => "never",
            }
            .to_string(),
        ),
    ];

    for (key, value) in settings_to_save {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    // Сохраняем пути
    if let Some(path) = &settings.paths.steam {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("steam_path")
            .bind(path)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    if let Some(path) = &settings.paths.epic {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("epic_path")
            .bind(path)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    // Сохраняем настройки логирования
    if let Some(path) = &settings.logging.custom_path {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("log_custom_path")
            .bind(path.to_string_lossy().to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    // Сохраняем учетные данные Steam
    if let Some(username) = &settings.steam_username {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("steam_username")
            .bind(username)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    if let Some(password) = &settings.steam_password {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("steam_password")
            .bind(password)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    // Сохраняем путь к SteamCMD
    if let Some(path) = &settings.custom_steamcmd_path {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("custom_steamcmd_path")
            .bind(path.to_string_lossy().to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

    Ok(())
} 