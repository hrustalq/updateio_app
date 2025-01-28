use super::{RegistryReader, WindowsRegistry};
use crate::error::{Result, Error};
use std::path::PathBuf;

pub struct EpicRegistry {
    registry: WindowsRegistry,
}

impl EpicRegistry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            registry: WindowsRegistry::new(),
        })
    }
}

impl RegistryReader for EpicRegistry {
    fn get_install_path(&self) -> Result<PathBuf> {
        let path = self.registry.get_value_from_path(
            "SOFTWARE\\Epic Games\\EpicGamesLauncher",
            "AppDataPath"
        )?;
        Ok(PathBuf::from(path))
    }

    fn get_library_folders(&self) -> Result<Vec<PathBuf>> {
        let install_path = self.get_install_path()?;
        let manifest_path = install_path.join("Manifests");
        
        if manifest_path.exists() {
            Ok(vec![manifest_path])
        } else {
            Ok(Vec::new())
        }
    }

    fn get_installed_games(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut games = Vec::new();
        let manifest_path = self.get_install_path()?.join("Manifests");

        if manifest_path.exists() {
            for entry in std::fs::read_dir(&manifest_path)
                .map_err(|e| Error::ProcessError(e.to_string()))?
            {
                let entry = entry.map_err(|e| Error::ProcessError(e.to_string()))?;
                let path = entry.path();

                if let Some(ext) = path.extension() {
                    if ext == "item" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let (Some(id), Some(install_path)) = (
                                    manifest["CatalogItemId"].as_str(),
                                    manifest["InstallLocation"].as_str(),
                                ) {
                                    games.push((
                                        id.to_string(),
                                        PathBuf::from(install_path),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(games)
    }
}
