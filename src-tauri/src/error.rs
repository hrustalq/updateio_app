use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Steam error: {0}")]
    SteamError(String),

    #[error("SteamCmd not found")]
    SteamCmdNotFound,

    #[error("Update failed: {0}")]
    UpdateFailed(String),

    #[error("Logging error: {0}")]
    LoggingError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Other error: {0}")]
    Other(String),
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
        Error::Other(err.to_string())
    }
}
