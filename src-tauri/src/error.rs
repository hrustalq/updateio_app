use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Steam CMD not found")]
    SteamCmdNotFound,

    #[error("Epic Games Launcher not found")]
    EpicLauncherNotFound,

    #[error("Update failed: {0}")]
    UpdateFailed(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("Event emission error: {0}")]
    EmitError(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Database(err.to_string())
    }
}

impl From<Error> for String {
    fn from(err: Error) -> String {
        err.to_string()
    }
}

impl From<tauri::Error> for Error {
    fn from(err: tauri::Error) -> Self {
        Error::Unknown(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
