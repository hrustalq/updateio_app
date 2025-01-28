use crate::error::{Result, Error};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

pub mod steam;
pub mod epic;

pub trait RegistryReader {
    fn get_install_path(&self) -> Result<PathBuf>;
    fn get_library_folders(&self) -> Result<Vec<PathBuf>>;
    fn get_installed_games(&self) -> Result<Vec<(String, PathBuf)>>;
}

pub struct WindowsRegistry {
    hklm: RegKey,
}

impl WindowsRegistry {
    pub fn new() -> Self {
        Self {
            hklm: RegKey::predef(HKEY_LOCAL_MACHINE),
        }
    }

    pub fn open_key(&self, path: &str) -> Result<RegKey> {
        self.hklm
            .open_subkey(path)
            .map_err(|e| Error::Registry(e.to_string()))
    }

    pub fn get_value(&self, key: &RegKey, name: &str) -> Result<String> {
        key.get_value(name)
            .map_err(|e| Error::Registry(e.to_string()))
    }

    pub fn get_value_from_path(&self, path: &str, name: &str) -> Result<String> {
        let key = self.open_key(path)?;
        self.get_value(&key, name)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct InstalledProgram {
    pub name: String,
    pub install_location: PathBuf,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub uninstall_string: Option<String>,
}

impl RegistryReader for WindowsRegistry {
    fn get_install_path(&self) -> Result<PathBuf> {
        // Сначала пробуем 64-битный раздел
        let result = self.open_key("SOFTWARE\\Valve\\Steam")
            .and_then(|key| self.get_value(&key, "InstallPath"));

        if let Ok(path) = result {
            return Ok(PathBuf::from(path));
        }

        // Если не нашли, пробуем 32-битный раздел
        let result = self.open_key("SOFTWARE\\WOW6432Node\\Valve\\Steam")
            .and_then(|key| self.get_value(&key, "InstallPath"));

        match result {
            Ok(path) => Ok(PathBuf::from(path)),
            Err(e) => Err(Error::Registry(format!("Steam не найден ни в 32-битном, ни в 64-битном реестре: {}", e)))
        }
    }

    fn get_library_folders(&self) -> Result<Vec<PathBuf>> {
        // Implementation needed
        Ok(Vec::new())
    }

    fn get_installed_games(&self) -> Result<Vec<(String, PathBuf)>> {
        // Implementation needed
        Ok(Vec::new())
    }
}

// Реализации для конкретных платформ
// pub mod battlenet;
// pub mod origin;
