use crate::error::{Result, Error};
use crate::steam::{SteamGame, UpdateState, UpdateStatus};
use std::path::PathBuf;

pub fn parse_installed_games(output: &str) -> Result<Vec<SteamGame>> {
    let mut games = Vec::new();
    let mut current_app_id = None;
    let mut current_name = None;
    let mut current_path = None;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("AppID") {
            if let Some(id) = line.split(':').nth(1) {
                current_app_id = Some(id.trim().parse::<u32>().unwrap_or(0));
            }
        } else if line.starts_with("Name") {
            if let Some(name) = line.split(':').nth(1) {
                current_name = Some(name.trim().to_string());
            }
        } else if line.starts_with("InstallDir") {
            if let Some(path) = line.split(':').nth(1) {
                current_path = Some(PathBuf::from(path.trim()));
            }
        }

        if let (Some(app_id), Some(name), Some(install_dir)) =
            (current_app_id, current_name.clone(), current_path.clone())
        {
            games.push(SteamGame {
                app_id,
                name,
                install_dir,
            });
            current_app_id = None;
            current_name = None;
            current_path = None;
        }
    }

    Ok(games)
}

pub fn check_update_needed(output: &str) -> Result<bool> {
    for line in output.lines() {
        if line.contains("Update needed") {
            return Ok(true);
        }
        if line.contains("Up to date") {
            return Ok(false);
        }
    }
    Err(Error::Unknown(
        "Could not determine update status".to_string(),
    ))
}

pub fn parse_download_progress(line: &str) -> Option<(u64, u64)> {
    if line.contains("Update:") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let progress = parts[1].split('/').collect::<Vec<&str>>();
            if progress.len() == 2 {
                if let (Ok(current), Ok(total)) =
                    (progress[0].parse::<u64>(), progress[1].parse::<u64>())
                {
                    return Some((current, total));
                }
            }
        }
    }
    None
}

pub fn parse_update_state(line: &str) -> UpdateState {
    if line.contains("0x61") {
        UpdateState::Downloading
    } else if line.contains("0x81") {
        UpdateState::Installing
    } else if line.contains("0x101") {
        UpdateState::Complete
    } else if line.contains("0x11") {
        UpdateState::Starting
    } else {
        UpdateState::Unknown
    }
}

pub fn parse_update_status(line: &str) -> Result<UpdateStatus> {
    let state = parse_update_state(line);
    let progress = parse_download_progress(line)
        .map(|(current, total)| (current as f32 / total as f32) * 100.0)
        .unwrap_or(0.0);

    Ok(UpdateStatus {
        progress,
        status: line.to_string(),
        state,
        error: None,
    })
}
