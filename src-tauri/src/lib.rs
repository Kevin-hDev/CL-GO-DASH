// La base de code contient légitimement des fonctions à many paramètres
// (commandes Tauri, exécuteurs d'outils avec contexte riche). Les refactorer
// en structs serait risqué et hors-sujet. On désactive le lint globalement.
#![allow(clippy::too_many_arguments)]
// Plusieurs modules de tests compagnons portent le même nom que leur module
// parent (convention *_tests.rs). C'est intentionnel et documenté.
#![allow(clippy::module_inception)]

mod app_events;
mod commands;
mod invoke_handler;
mod models;
mod ollama_polling;
mod services;
mod storage_migration;
mod storage_migration_files;
mod tray;

use services::agent_local::ollama_client::OllamaClient;
use services::gateway::GatewayService;
use services::ollama_lifecycle::{self, OllamaSidecar};
use services::scheduler::Scheduler;
use std::collections::HashMap;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Mutex;

pub struct ActiveStreams(
    pub(crate) Mutex<HashMap<String, commands::agent_chat_streams::StreamEntry>>,
);

static STREAM_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![app_events::AUTOSTART_ARG]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .manage(OllamaClient::new())
        .manage(ActiveStreams(Mutex::new(HashMap::new())))
        .manage(OllamaSidecar::new())
        .manage(services::model_downloads::ModelDownloadManager::new())
        .manage(services::searxng::SearxngSidecar::new())
        .manage(services::terminal::PtyManager::new())
        .manage(GatewayService::new())
        .manage(commands::file_tree_watcher::FileTreeWatcher::new())
        .manage(services::forecast::sidecar::ChronosSidecar::new())
        .setup(|app| {
            let startup_cutoff = chrono::Utc::now();
            services::agent_local::app_handle_global::init(app.handle().clone());
            services::agent_local::subagent_spawn_channel::init();
            if let Err(e) = storage_migration::run(app.handle()) {
                eprintln!("[storage migration] {}", e);
            }
            services::private_store::repair_app_storage().map_err(std::io::Error::other)?;
            // Cleanup des sous-agents orphelins (crash précédent) : non bloquant.
            tauri::async_runtime::spawn(async move {
                services::agent_local::subagent_startup_cleanup::cleanup_orphans(startup_cutoff)
                    .await;
            });
            let _ = dotenvy::dotenv();
            if let Err(e) = services::api_keys::init() {
                eprintln!("[vault] init failed: {}", e);
                let handle = app.handle().clone();
                let msg = e.to_string();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let _ = handle.emit("vault-init-failed", msg);
                });
            }
            services::searxng::prepare_on_startup(app.handle().clone());
            if ollama_lifecycle::ollama_binary_path().is_ok() {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    match tokio::task::spawn_blocking(move || {
                        ollama_lifecycle::start_sidecar(&handle)
                    })
                    .await
                    {
                        Ok(Err(e)) => eprintln!("[ollama] sidecar start failed: {}", e),
                        Err(e) => eprintln!("[ollama] sidecar task failed: {}", e),
                        _ => {}
                    }
                });
            }

            let config = services::config::read_config().unwrap_or_default();

            // Autostart : synchronise l'état OS avec le setting
            app_events::sync_autostart(app.handle(), config.advanced.autostart);

            // Start hidden applies only to launches initiated by the autostart entry.
            if app_events::should_start_hidden(&config) {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            // Linux/Windows : désactiver les décorations natives, boutons custom React
            #[cfg(not(target_os = "macos"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_decorations(false);
                    let _ = win.set_shadow(false);
                }
            }

            // Tray icon
            if config.advanced.show_tray {
                let _ = tray::create_tray(app);
            }

            // Gateway : démarrage si configuré
            if config.gateway.enabled && config.gateway.start_with_app {
                let gw_config = config.gateway.clone();
                let gw_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let gw = gw_handle.state::<GatewayService>();
                    gw.start(gw_config, gw_handle.clone()).await;
                });
            }

            services::file_watcher::start(app.handle());
            let scheduler = Scheduler::spawn(app.handle().clone());
            app.manage(scheduler);
            ollama_polling::start(app.handle().clone());
            tauri::async_runtime::spawn(services::llm::model_registry::init());
            Ok(())
        })
        .on_window_event(|_window, _event| {
            #[cfg(target_os = "macos")]
            if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                if _window.label() == "main" {
                    let _ = _window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(invoke_handler::generate!())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(app_events::handle_run_event);
}
