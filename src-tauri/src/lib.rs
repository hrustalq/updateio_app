// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod cache;
mod error;
pub mod registry;
pub mod games;
pub mod settings;
pub mod commands;

use parking_lot::Mutex;
use crate::settings::manager::SettingsManager;
use crate::games::manager::GameManager;
use tauri::Manager;
use tracing::info;
use std::sync::Arc;

pub use error::{Error, Result};

// Глобальное состояние
pub struct AppState {
    pub settings: Arc<Mutex<Option<SettingsManager>>>,
    pub game_manager: Arc<Mutex<GameManager>>,
    pub runtime: Arc<tokio::runtime::Runtime>,
}

impl AppState {
    pub fn new(
        settings_manager: SettingsManager,
        game_manager: GameManager,
        runtime: Arc<tokio::runtime::Runtime>,
    ) -> Self {
        Self {
            settings: Arc::new(Mutex::new(Some(settings_manager))),
            game_manager: Arc::new(Mutex::new(game_manager)),
            runtime,
        }
    }
}

fn setup_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();
    info!("Starting UpdateIO application");

    let runtime = Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
    let runtime_clone = runtime.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(move |app| {
            let settings_manager = runtime_clone
                .block_on(async { SettingsManager::new().await })
                .expect("Failed to initialize settings manager");

            let settings = runtime_clone
                .block_on(async { settings_manager.load().await })
                .expect("Failed to load settings");

            let game_manager = GameManager::new(settings, app.handle().clone());

            app.manage(settings_manager.clone());
            app.manage(game_manager.clone());

            app.manage(AppState::new(
                settings_manager,
                game_manager,
                runtime_clone,
            ));

            #[cfg(debug_assertions)]
            app.get_webview_window("main").unwrap().open_devtools();
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_installed_games,
            commands::check_game_updates,
            commands::update_game,
            commands::select_directory,
            commands::refresh_games_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
