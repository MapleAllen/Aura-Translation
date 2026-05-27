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

/// Managed config - the single source of truth, shared between commands.
type ConfigState = Arc<RwLock<AppConfig>>;

#[derive(Serialize)]
struct ProviderDefaults {
    base_url: String,
    models: Vec<String>,
}

#[derive(Clone, Serialize)]
struct DaemonErrorPayload {
    code: String,
    message: String,
    recoverable: bool,
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

fn emit_daemon_error(
    app: &AppHandle,
    code: impl Into<String>,
    message: impl Into<String>,
    recoverable: bool,
) {
    let _ = app.emit(
        "daemon-error",
        DaemonErrorPayload {
            code: code.into(),
            message: message.into(),
            recoverable,
        },
    );
}

fn emit_hotkey_conflict(app: &AppHandle, hotkey: &str, error: impl Into<String>) {
    let _ = app.emit(
        "hotkey-conflict",
        serde_json::json!({
            "hotkey": hotkey,
            "error": error.into()
        }),
    );
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn restore_previous_hotkey(app: &AppHandle, hotkey: &str) {
    match hotkey::parse_hotkey(hotkey) {
        Ok(shortcut) => {
            if let Err(e) = app.global_shortcut().register(shortcut) {
                emit_daemon_error(
                    app,
                    "hotkey-restore-failed",
                    format!(
                        "Failed to restore the previous hotkey '{}' after a rejected save: {}",
                        hotkey, e
                    ),
                    false,
                );
                register_fallback_hotkey(app);
            }
        }
        Err(e) => {
            emit_daemon_error(
                app,
                "hotkey-restore-parse-failed",
                format!(
                    "Failed to parse the previous hotkey '{}' after a rejected save: {}",
                    hotkey, e
                ),
                false,
            );
            register_fallback_hotkey(app);
        }
    }
}

/// Tauri command: save config from the frontend and re-register the hotkey.
///
/// Saves the full config atomically from the user's perspective:
/// - if the hotkey is invalid or cannot be registered, no config changes persist
/// - if disk save fails after hotkey re-registration, the old hotkey is restored
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn save_config(
    app: AppHandle,
    state: State<'_, ConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    let old_config = state.read().unwrap().clone();
    let old_hotkey = old_config.hotkey.clone();

    // Hotkey unchanged - skip re-registration entirely
    if old_hotkey == config.hotkey {
        config.save()?;
        *state.write().unwrap() = config.clone();
        return Ok(());
    }

    let new_shortcut = match hotkey::parse_hotkey(&config.hotkey) {
        Ok(shortcut) => shortcut,
        Err(parse_err) => {
            emit_hotkey_conflict(&app, &config.hotkey, parse_err.clone());
            return Err(format!("Hotkey save rejected: {}", parse_err));
        }
    };

    if let Err(e) = app.global_shortcut().unregister_all() {
        emit_daemon_error(
            &app,
            "hotkey-unregister-failed",
            format!(
                "Failed to unregister the previous hotkey before saving '{}': {}",
                config.hotkey, e
            ),
            true,
        );
        return Err(format!("Failed to prepare hotkey update: {}", e));
    }

    if let Err(e) = app.global_shortcut().register(new_shortcut) {
        emit_hotkey_conflict(&app, &config.hotkey, e.to_string());
        restore_previous_hotkey(&app, &old_hotkey);
        return Err(format!("Hotkey save rejected: {}", e));
    }

    if let Err(e) = config.save() {
        emit_daemon_error(
            &app,
            "config-save-failed",
            format!(
                "Failed to persist config after registering hotkey '{}': {}",
                config.hotkey, e
            ),
            false,
        );

        if let Err(unregister_err) = app.global_shortcut().unregister_all() {
            emit_daemon_error(
                &app,
                "hotkey-unregister-after-save-failure",
                format!(
                    "Failed to unregister the new hotkey '{}' after a config save failure: {}",
                    config.hotkey, unregister_err
                ),
                false,
            );
        }

        restore_previous_hotkey(&app, &old_hotkey);
        return Err(format!("Failed to save config: {}", e));
    }

    *state.write().unwrap() = config.clone();
    let _ = app.emit("hotkey-registered", &config.hotkey);

    Ok(())
}

/// Mobile stub - no global shortcuts on mobile.
#[tauri::command]
#[cfg(any(target_os = "android", target_os = "ios"))]
fn save_config(_app: AppHandle, config: AppConfig) -> Result<(), String> {
    config.save()
}

fn build_tray(app: &AppHandle) -> Result<(), String> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| format!("Failed to build tray Settings item: {}", e))?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to build tray separator: {}", e))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| format!("Failed to build tray Quit item: {}", e))?;

    let menu = Menu::with_items(app, &[&settings_item, &separator, &quit_item])
        .map_err(|e| format!("Failed to build tray menu: {}", e))?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "Default window icon is missing.".to_string())?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Aura Translation")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = app.emit("show-settings", ());
                    let _ = window.show();
                    let _ = window.set_focus();
                } else {
                    emit_daemon_error(
                        &app,
                        "settings-window-missing",
                        "The main window was not available when opening Settings from the tray.",
                        false,
                    );
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|e| format!("Failed to build tray icon: {}", e))?;

    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn register_startup_hotkey(app: &AppHandle) {
    let state = app.state::<ConfigState>();
    let config = state.read().unwrap();

    match hotkey::parse_hotkey(&config.hotkey) {
        Ok(shortcut) => {
            if let Err(e) = app.global_shortcut().register(shortcut) {
                emit_daemon_error(
                    app,
                    "hotkey-register-failed",
                    format!(
                        "Failed to register startup hotkey '{}': {}. Falling back to CmdOrCtrl+T.",
                        config.hotkey, e
                    ),
                    true,
                );
                register_fallback_hotkey(app);
            }
        }
        Err(e) => {
            emit_daemon_error(
                app,
                "hotkey-parse-failed",
                format!(
                    "Failed to parse startup hotkey '{}': {}. Falling back to CmdOrCtrl+T.",
                    config.hotkey, e
                ),
                true,
            );
            register_fallback_hotkey(app);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn register_fallback_hotkey(app: &AppHandle) {
    match hotkey::parse_hotkey("CmdOrCtrl+T") {
        Ok(fallback) => {
            if let Err(e) = app.global_shortcut().register(fallback) {
                emit_daemon_error(
                    app,
                    "hotkey-fallback-register-failed",
                    format!("Failed to register CmdOrCtrl+T fallback hotkey: {}", e),
                    false,
                );
            }
        }
        Err(e) => {
            emit_daemon_error(
                app,
                "hotkey-fallback-parse-failed",
                format!("Failed to parse CmdOrCtrl+T fallback hotkey: {}", e),
                false,
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Shared HTTP client - reuses connection pool across all translation requests
    let http_client = reqwest::Client::new();
    // Cancellation registry - allows in-flight translations to be cancelled
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
                        let clipboard_text = app.clipboard().read_text().unwrap_or_default();

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

                                let _ = window.set_position(tauri::LogicalPosition::new(x, y));
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
            if let Err(err) = build_tray(app.app_handle()) {
                emit_daemon_error(app.app_handle(), "tray-build-failed", err, false);
            }

            // Main Window - Focus Loss Handler
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = w.emit("window-blur", ());
                    }
                });
            } else {
                emit_daemon_error(
                    app.app_handle(),
                    "main-window-missing",
                    "The main window was not available during setup.",
                    false,
                );
            }

            // Register Global Shortcut from Config
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                register_startup_hotkey(app.app_handle());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
