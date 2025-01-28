use super::{RegistryReader, WindowsRegistry};
use crate::error::{Result, Error};
use std::path::PathBuf;

pub struct SteamRegistry {
    registry: WindowsRegistry,
}

impl SteamRegistry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            registry: WindowsRegistry::new(),
        })
    }

    fn parse_vdf_file(&self, path: &PathBuf) -> Result<Vec<PathBuf>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ProcessError(e.to_string()))?;
        
        let mut folders = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("\"path\"") {
                if let Some(path) = line.split('"').nth(3) {
                    folders.push(PathBuf::from(path).join("steamapps"));
                }
            }
        }
        Ok(folders)
    }

    fn parse_acf_file(&self, path: &PathBuf) -> Result<Option<(String, String)>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ProcessError(e.to_string()))?;

        let mut name = None;
        let mut install_dir = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("\"name\"") {
                if let Some(n) = line.split('"').nth(3) {
                    name = Some(n.to_string());
                }
            } else if line.starts_with("\"installdir\"") {
                if let Some(dir) = line.split('"').nth(3) {
                    install_dir = Some(dir.to_string());
                }
            }
        }

        match (name, install_dir) {
            (Some(name), Some(dir)) => Ok(Some((name, dir))),
            _ => Ok(None),
        }
    }
}

impl RegistryReader for SteamRegistry {
    fn get_install_path(&self) -> Result<PathBuf> {
        self.registry.get_install_path()
    }

    fn get_library_folders(&self) -> Result<Vec<PathBuf>> {
        let steam_path = self.get_install_path()?;
        let mut folders = vec![steam_path.join("steamapps")];
        
        let vdf_path = steam_path.join("steamapps/libraryfolders.vdf");
        if vdf_path.exists() {
            folders.extend(self.parse_vdf_file(&vdf_path)?);
        }

        Ok(folders)
    }

    fn get_installed_games(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut games = Vec::new();
        let libraries = self.get_library_folders()?;

        for library in libraries {
            for entry in std::fs::read_dir(&library)
                .map_err(|e| Error::ProcessError(e.to_string()))?
            {
                let entry = entry.map_err(|e| Error::ProcessError(e.to_string()))?;
                let path = entry.path();
                
                if let Some(ext) = path.extension() {
                    if ext == "acf" {
                        if let Ok(Some((name, install_dir))) = self.parse_acf_file(&path) {
                            games.push((
                                name,
                                library.join("common").join(install_dir)
                            ));
                        }
                    }
                }
            }
        }

        Ok(games)
    }
}
