// La base de code contient légitimement des fonctions à many paramètres
// (commandes Tauri, exécuteurs d'outils avec contexte riche). Les refactorer
// en structs serait risqué et hors-sujet. On désactive le lint globalement.
#![allow(clippy::too_many_arguments)]
// Plusieurs modules de tests compagnons portent le même nom que leur module
// parent (convention *_tests.rs). C'est intentionnel et documenté.
#![allow(clippy::module_inception)]

mod app_events;
mod commands;
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
use tokio_util::sync::CancellationToken;

pub struct ActiveStreams(pub Mutex<HashMap<String, (CancellationToken, u64, String)>>);

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
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_advanced_settings,
            commands::set_advanced_settings,
            commands::patch_advanced_settings,
            commands::get_effective_context_length,
            commands::list_wakeups,
            commands::create_wakeup,
            commands::update_wakeup,
            commands::delete_wakeup,
            commands::set_wakeup_active,
            commands::set_global_paused,
            commands::get_heartbeat_config,
            commands::list_wakeup_runs,
            commands::get_wakeup_status_summaries,
            commands::list_personality_files,
            commands::read_personality_file,
            commands::open_in_editor,
            commands::get_injection_state,
            commands::set_injection_state,
            // Agent Local — Ollama
            commands::list_ollama_models,
            commands::show_ollama_model,
            commands::is_ollama_running,
            commands::search_ollama_models,
            commands::get_registry_model_details,
            commands::list_registry_tags,
            commands::translate_description,
            commands::delete_ollama_model,
            commands::get_modelfile,
            commands::update_modelfile,
            commands::update_system_prompt,
            commands::update_parameters,
            // Agent Local — Chat + Sessions
            commands::chat_stream,
            commands::estimate_context_hidden_usage,
            commands::cancel_agent_request,
            commands::list_agent_sessions,
            commands::list_archived_agent_sessions,
            commands::get_agent_session,
            commands::save_agent_session,
            commands::get_session_permission_state,
            commands::set_session_permission_mode,
            commands::prepare_agent_send,
            commands::resolve_missing_session_directory,
            commands::add_messages_to_session,
            commands::create_agent_session,
            commands::rename_agent_session,
            commands::update_session_model,
            commands::update_session_reasoning,
            commands::set_session_plan_mode,
            commands::delete_agent_session,
            commands::archive_agent_session,
            commands::restore_agent_session,
            commands::clone_agent_session,
            commands::cancel_clone_summary,
            commands::list_session_tabs,
            commands::save_session_tabs,
            commands::close_session_tab,
            commands::rename_session_tab,
            commands::create_clone_git_branch,
            commands::unlink_clone_git_branch,
            commands::link_clone_git_branch,
            commands::close_session_tab_and_cleanup_git_branch,
            commands::export_agent_session_markdown,
            commands::truncate_and_replace_at,
            commands::get_agent_settings,
            commands::set_permission_mode,
            commands::list_agent_tool_catalog,
            commands::list_agent_tool_groups,
            commands::set_agent_tool_enabled,
            commands::set_agent_tool_group_enabled,
            commands::respond_to_permission,
            commands::respond_to_interactive_choice,
            // Agent Local — Subagents
            commands::list_subagents,
            commands::cancel_subagent,
            // Agent Local — Tools
            commands::list_skills,
            commands::load_skill,
            // API Keys (multi-provider)
            commands::set_api_key,
            commands::delete_api_key,
            commands::has_api_key,
            commands::list_configured_providers,
            commands::test_api_key,
            commands::test_api_key_with_value,
            // MCP OAuth
            commands::start_mcp_oauth,
            commands::cancel_mcp_oauth,
            commands::has_mcp_oauth_token,
            commands::delete_mcp_oauth_token,
            commands::set_mcp_env_token,
            commands::delete_mcp_env_token,
            commands::list_mcp_connectors,
            commands::add_mcp_connector,
            commands::remove_mcp_connector,
            commands::set_mcp_connector_status,
            commands::set_mcp_connector_chat_enabled,
            commands::test_mcp_connector,
            // LLM providers
            commands::list_llm_providers_catalog,
            commands::list_llm_models,
            commands::test_llm_connection,
            commands::supports_tool_use,
            commands::get_provider_quota,
            // Search providers
            commands::list_search_providers_catalog,
            commands::test_search_connection,
            // LLM Registry
            commands::search_registry,
            commands::get_registry_model,
            commands::list_registry_families,
            commands::list_family_models,
            // Favorite models
            commands::list_favorite_models,
            commands::add_favorite_model,
            commands::remove_favorite_model,
            // File tree
            commands::list_directory,
            commands::watch_project_directory,
            commands::unwatch_project_directory,
            // File preview
            commands::read_file_preview,
            commands::check_preview_files_exist,
            commands::detect_editors_for_file,
            commands::open_preview_file,
            commands::open_preview_with_editor,
            commands::read_spreadsheet_preview,
            commands::read_binary_preview,
            // Projects
            commands::list_projects,
            commands::add_project,
            commands::rename_project,
            commands::delete_project,
            commands::reorder_projects,
            commands::open_project_folder,
            // Agent MD
            commands::read_agent_md,
            commands::write_agent_md,
            // Terminal PTY
            commands::pty_spawn,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_kill,
            // Updates
            commands::check_app_update,
            commands::download_app_update,
            commands::check_ollama_updates,
            commands::check_ollama_binary_update,
            // Ollama setup
            commands::is_ollama_installed,
            commands::download_ollama,
            commands::cancel_ollama_setup,
            commands::update_ollama_binary,
            commands::restart_ollama_sidecar,
            commands::check_model_fits_vram,
            commands::start_model_download,
            commands::list_model_downloads,
            commands::cancel_model_download,
            // Link preview
            commands::fetch_link_preview,
            // Git operations
            commands::start_git_watcher,
            commands::list_git_branches,
            commands::get_git_context,
            commands::checkout_git_branch,
            commands::create_git_branch,
            commands::commit_and_checkout_git_branch,
            commands::list_git_dirty_files,
            commands::list_git_worktrees,
            // Gateway
            commands::gateway_status,
            commands::gateway_start,
            commands::gateway_stop,
            commands::gateway_get_config,
            commands::gateway_set_config,
            commands::gateway_set_token,
            commands::gateway_delete_token,
            commands::gateway_has_token,
            // Codex OAuth (dev-only)
            commands::codex_login,
            commands::codex_logout,
            commands::codex_status,
            commands::codex_models,
            // Forecast
            commands::run_forecast,
            commands::list_forecast_analyses,
            commands::get_forecast_analysis,
            commands::export_forecast_analysis,
            commands::create_forecast_scenario,
            commands::update_forecast_scenario,
            commands::delete_forecast_scenario,
            commands::delete_forecast_analysis,
            commands::rename_forecast_analysis,
            commands::list_forecast_notes,
            commands::create_forecast_note,
            commands::update_forecast_note,
            commands::delete_forecast_note,
            commands::open_forecast_note,
            commands::list_forecast_models,
            commands::get_selected_forecast_model,
            commands::set_selected_forecast_model,
            commands::get_forecast_model_config,
            commands::set_forecast_model_config,
            commands::get_forecast_model_details,
            commands::uninstall_forecast_model,
            commands::list_forecast_providers_catalog,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(app_events::handle_run_event);
}
