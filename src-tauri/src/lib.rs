// Public so tools/examples can sample challenges from a database without
// launching the app; the game modules stay private.
pub mod dataset;
mod game;
mod persistence;
pub mod randomizer;
pub mod types;
mod wikipedia;

use dataset::Dataset;
use game::GameManager;
use persistence::Persistence;
use std::{path::PathBuf, sync::Arc};
use tauri::{Manager, State, WindowEvent};
use types::{Challenge, Settings, StatsSnapshot};

struct DatasetState(Result<Dataset, String>);

#[tauri::command]
fn generate_challenge(
    dataset: State<'_, DatasetState>,
    persistence: State<'_, Arc<Persistence>>,
    previous: Option<Challenge>,
) -> Result<Challenge, String> {
    let db = dataset.0.as_ref().map_err(Clone::clone)?;
    randomizer::generate(db, previous.as_ref(), &persistence.settings())
}

#[tauri::command]
fn get_settings(persistence: State<'_, Arc<Persistence>>) -> Settings {
    persistence.settings()
}

#[tauri::command]
fn update_settings(
    persistence: State<'_, Arc<Persistence>>,
    settings: Settings,
) -> Result<Settings, String> {
    persistence.update_settings(settings)
}

#[tauri::command]
fn get_stats(persistence: State<'_, Arc<Persistence>>) -> StatsSnapshot {
    persistence.stats()
}

#[tauri::command]
fn clear_run_history(persistence: State<'_, Arc<Persistence>>) -> Result<(), String> {
    persistence.clear_history()
}

#[tauri::command]
fn reset_personal_bests(persistence: State<'_, Arc<Persistence>>) -> Result<(), String> {
    persistence.reset_personal_bests()
}

#[tauri::command]
fn reset_streak(persistence: State<'_, Arc<Persistence>>) -> Result<(), String> {
    persistence.reset_streak()
}

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
fn dataset_version(dataset: State<'_, DatasetState>) -> String {
    dataset
        .0
        .as_ref()
        .map(Dataset::version)
        .unwrap_or_else(|_| "unavailable".into())
}

#[tauri::command]
async fn start_run(
    app: tauri::AppHandle,
    manager: State<'_, GameManager>,
    persistence: State<'_, Arc<Persistence>>,
    challenge: Challenge,
) -> Result<String, String> {
    wikipedia::close(&app);
    let id = manager.prepare(challenge)?;
    if let Err(error) = wikipedia::open(
        app.clone(),
        manager.inner().clone(),
        persistence.inner().clone(),
    )
    .await
    {
        manager.cancel();
        return Err(error);
    }
    Ok(id)
}

#[tauri::command]
async fn replay_run(
    app: tauri::AppHandle,
    manager: State<'_, GameManager>,
    persistence: State<'_, Arc<Persistence>>,
) -> Result<String, String> {
    let challenge = manager
        .last_challenge
        .lock()
        .map_err(|_| "Game state lock was poisoned")?
        .clone()
        .ok_or("There is no challenge to replay")?;
    wikipedia::close(&app);
    let id = manager.prepare(challenge)?;
    if let Err(error) = wikipedia::open(
        app.clone(),
        manager.inner().clone(),
        persistence.inner().clone(),
    )
    .await
    {
        manager.cancel();
        return Err(error);
    }
    Ok(id)
}

#[tauri::command]
fn cancel_run(app: tauri::AppHandle, manager: State<'_, GameManager>) {
    manager.cancel();
    wikipedia::close(&app);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Release CI injects the public key and a complete updater config.
            // Local/dev builds stay runnable without pretending to have a key.
            if option_env!("TAURI_UPDATER_PUBKEY").is_some() {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            let resource = app
                .path()
                .resolve("articles.sqlite", tauri::path::BaseDirectory::Resource)
                .ok();
            let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/articles.sqlite");
            let path = resource.filter(|p| p.exists()).unwrap_or(dev);
            app.manage(DatasetState(Dataset::open(&path)));
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("BlueLink"));
            app.manage(Arc::new(Persistence::new(app_data.join("runs.json"))));
            app.manage(GameManager::default());
            // Install one resize listener for the lifetime of the window. Runs can
            // be replayed indefinitely without accumulating native callbacks.
            if let Some(window) = app.get_window("main") {
                let resize_app = app.handle().clone();
                let resize_window = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Resized(physical) = event {
                        let scale = resize_window.scale_factor().unwrap_or(1.0);
                        wikipedia::resize_child(
                            &resize_app,
                            physical.width as f64,
                            physical.height as f64,
                            scale,
                        );
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_challenge,
            get_settings,
            update_settings,
            get_stats,
            clear_run_history,
            reset_personal_bests,
            reset_streak,
            app_version,
            dataset_version,
            start_run,
            replay_run,
            cancel_run
        ])
        .run(tauri::generate_context!())
        .expect("BlueLink could not start");
}
