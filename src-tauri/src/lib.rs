mod config;
mod hotkey;
mod translate;

use config::{AppConfig, Provider};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tokio::sync::Mutex;
use translate::CancellationRegistry;

/// Managed config — the single source of truth, shared between commands.
type ConfigState = Arc<RwLock<AppConfig>>;

#[derive(Serialize)]
struct ProviderDefaults {
    base_url: String,
    models: Vec<String>,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Tauri command: get current config from managed state (no disk I/O).
#[tauri::command]
fn get_config(state: State<'_, ConfigState>) -> AppConfig {
    state.read().unwrap().clone()
}

/// Tauri command: get default base URL and model list for a provider.
/// Used by the frontend so provider defaults only live in Rust.
#[tauri::command]
fn get_provider_defaults(provider: Provider) -> ProviderDefaults {
    ProviderDefaults {
        base_url: provider.default_base_url().to_string(),
        models: provider.default_models(),
    }
}

/// Tauri command: save config from the frontend and re-register the hotkey.
///
/// On hotkey re-registration failure, emits a `hotkey-conflict` event and
/// re-registers the previously-working hotkey. Falls back to `CmdOrCtrl+T`
/// only if the old hotkey also fails to re-register (extreme edge case).
/// Does NOT return an error to the frontend — a conflict is a user-fixable
/// warning, not a fatal failure.
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn save_config(app: AppHandle, state: State<'_, ConfigState>, config: AppConfig) -> Result<(), String> {
    // Capture old hotkey from managed state before overwriting
    let old_hotkey = state.read().unwrap().hotkey.clone();
    config.save()?;

    // Update in-memory state
    *state.write().unwrap() = config.clone();

    // Hotkey unchanged — skip re-registration entirely
    if old_hotkey == config.hotkey {
        return Ok(());
    }

    match hotkey::parse_hotkey(&config.hotkey) {
        Ok(new_shortcut) => {
            if let Err(e) = app.global_shortcut().unregister_all() {
                eprintln!("Failed to unregister shortcuts during re-registration: {}", e);
            }
            match app.global_shortcut().register(new_shortcut) {
                Ok(_) => {
                    let _ = app.emit("hotkey-registered", &config.hotkey);
                }
                Err(e) => {
                    let _ = app.emit(
                        "hotkey-conflict",
                        serde_json::json!({
                            "hotkey": config.hotkey,
                            "error": e.to_string()
                        }),
                    );
                    // Re-register the old (previously-working) hotkey
                    let re_registered = hotkey::parse_hotkey(&old_hotkey)
                        .map(|old| app.global_shortcut().register(old).is_ok())
                        .unwrap_or(false);
                    // Last-resort fallback if even the old hotkey fails
                    if !re_registered {
                        if let Ok(fallback) = hotkey::parse_hotkey("CmdOrCtrl+T") {
                            let _ = app.global_shortcut().register(fallback);
                        }
                    }
                }
            }
        }
        Err(parse_err) => {
            // New hotkey string is malformed — old shortcut is still registered
            // (we never called unregister_all), so no fallback needed.
            let _ = app.emit(
                "hotkey-conflict",
                serde_json::json!({
                    "hotkey": config.hotkey,
                    "error": parse_err
                }),
            );
        }
    }

    Ok(())
}

/// Mobile stub — no global shortcuts on mobile.
#[tauri::command]
#[cfg(any(target_os = "android", target_os = "ios"))]
fn save_config(_app: AppHandle, config: AppConfig) -> Result<(), String> {
    config.save()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared HTTP client — reuses connection pool across all translation requests
    let http_client = reqwest::Client::new();
    // Cancellation registry — allows in-flight translations to be cancelled
    let cancel_registry: CancellationRegistry = Arc::new(Mutex::new(HashMap::new()));
    // Config loaded once at startup, kept in managed state for all commands
    let config_state: ConfigState = Arc::new(RwLock::new(AppConfig::load()));

    let mut builder = tauri::Builder::default()
        .manage(http_client)
        .manage(cancel_registry)
        .manage(config_state.clone())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init());

    // Register global shortcut plugin at the builder level (not in setup)
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        // Read clipboard via Tauri command
                        use tauri_plugin_clipboard_manager::ClipboardExt;
                        let clipboard_text = app
                            .clipboard()
                            .read_text()
                            .unwrap_or_default();

                        if clipboard_text.trim().is_empty() {
                            return;
                        }

                        // Position window near system tray (bottom-right of screen)
                        if let Some(window) = app.get_webview_window("main") {
                            // Get primary monitor dimensions
                            if let Ok(Some(monitor)) = window.primary_monitor() {
                                let screen_size = monitor.size();
                                let scale = monitor.scale_factor();
                                let screen_w = screen_size.width as f64 / scale;
                                let screen_h = screen_size.height as f64 / scale;

                                // Position near bottom-right (system tray area)
                                let win_w = 440.0;
                                let win_h = 360.0;
                                let x = screen_w - win_w - 16.0;
                                let y = screen_h - win_h - 60.0;

                                let _ =
                                    window.set_position(tauri::LogicalPosition::new(x, y));
                            }

                            let _ = window.show();
                            let _ = window.set_focus();

                            // Emit clipboard text to frontend
                            let _ = app.emit("trigger-translate", clipboard_text);
                        }
                    }
                })
                .build(),
        );
    }

    builder
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_provider_defaults,
            save_config,
            translate::translate_text,
            translate::cancel_translate
        ])
        .setup(|app| {
            // ── System Tray ──────────────────────────────────────────
            let settings_item =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&settings_item, &separator, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Aura Translation")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = app.emit("show-settings", ());
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // ── Main Window — Focus Loss Handler ─────────────────────
            let main_window = app.get_webview_window("main");
            if let Some(window) = main_window {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.emit("window-blur", ());
                    }
                });
            }

            // ── Register Global Shortcut from Config ─────────────────
            // Reads the hotkey from managed config state (loaded once at startup).
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let state = app.state::<ConfigState>();
                let config = state.read().unwrap();
                match hotkey::parse_hotkey(&config.hotkey) {
                    Ok(shortcut) => {
                        if let Err(e) = app.global_shortcut().register(shortcut) {
                            eprintln!(
                                "Failed to register hotkey '{}': {}. \
                                 Falling back to CmdOrCtrl+T.",
                                config.hotkey, e
                            );
                            // TODO: emit daemon-error when Phase 5 error surface is implemented
                            // Attempt fallback registration
                            if let Ok(fallback) = hotkey::parse_hotkey("CmdOrCtrl+T") {
                                let _ = app.global_shortcut().register(fallback);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to parse hotkey '{}': {}. \
                             Falling back to CmdOrCtrl+T.",
                            config.hotkey, e
                        );
                        if let Ok(fallback) = hotkey::parse_hotkey("CmdOrCtrl+T") {
                            let _ = app.global_shortcut().register(fallback);
                        }
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
