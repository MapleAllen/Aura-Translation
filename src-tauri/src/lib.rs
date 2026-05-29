mod config;
mod hotkey;
mod translate;

use config::{AppConfig, Provider};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State, WebviewWindow, WebviewWindowBuilder,
};
use tokio::sync::Mutex;
use translate::CancellationRegistry;

/// Managed config - the single source of truth, shared between commands.
type ConfigState = Arc<RwLock<AppConfig>>;
type UiReadyState = Arc<RwLock<bool>>;

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

#[tauri::command]
fn mark_ui_ready(state: State<'_, UiReadyState>) {
    *state.write().unwrap() = true;
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

fn attach_main_window_blur_listener(window: &WebviewWindow) {
    let emitted_window = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = emitted_window.emit("window-blur", ());
        }
    });
}

fn apply_window_preferences(window: &WebviewWindow, config: &AppConfig) -> Result<(), String> {
    window
        .set_always_on_top(config.window_pinned)
        .map_err(|e| format!("Failed to apply window pin state: {}", e))
}

fn sync_existing_main_window(app: &AppHandle, config: &AppConfig) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(err) = apply_window_preferences(&window, config) {
            emit_daemon_error(app, "main-window-sync-failed", err, true);
        }
    }
}

fn ensure_main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("main") {
        return Ok(window);
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .ok_or_else(|| "Main window config is missing.".to_string())?;

    *app.state::<UiReadyState>().write().unwrap() = false;

    let window = WebviewWindowBuilder::from_config(app, config)
        .map_err(|e| format!("Failed to prepare main window from config: {}", e))?
        .build()
        .map_err(|e| format!("Failed to build main window: {}", e))?;
    attach_main_window_blur_listener(&window);

    Ok(window)
}

async fn wait_for_ui_ready(app: &AppHandle, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if *app.state::<UiReadyState>().read().unwrap() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    Err("Timed out waiting for the main UI window to finish initializing.".to_string())
}

fn position_main_window(window: &WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return;
    };

    let Ok(window_size) = window.inner_size() else {
        return;
    };

    let scale = monitor.scale_factor();
    let work_area = monitor.work_area();
    let work_x = work_area.position.x as f64 / scale;
    let work_y = work_area.position.y as f64 / scale;
    let work_w = work_area.size.width as f64 / scale;
    let work_h = work_area.size.height as f64 / scale;
    let win_w = window_size.width as f64 / scale;
    let win_h = window_size.height as f64 / scale;

    let x = work_x + (work_w - win_w - 18.0).max(0.0);
    let y = work_y + (work_h - win_h - 18.0).max(0.0);

    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
}

fn current_config(app: &AppHandle) -> AppConfig {
    app.state::<ConfigState>().read().unwrap().clone()
}

fn prepare_main_window(app: &AppHandle, window: &WebviewWindow) {
    let config = current_config(app);
    if let Err(err) = apply_window_preferences(window, &config) {
        emit_daemon_error(app, "main-window-pin-apply-failed", err, true);
    }
    position_main_window(window);
}

async fn show_translation_window(app: AppHandle, clipboard_text: String) {
    let window = match ensure_main_window(&app) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "main-window-create-failed", err, false);
            return;
        }
    };

    prepare_main_window(&app, &window);
    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) = wait_for_ui_ready(&app, Duration::from_secs(5)).await {
        emit_daemon_error(&app, "main-window-ui-timeout", err, true);
        return;
    }

    let _ = app.emit("trigger-translate", clipboard_text);
}

async fn show_settings_window(app: AppHandle) {
    let window = match ensure_main_window(&app) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "main-window-create-failed", err, false);
            return;
        }
    };

    prepare_main_window(&app, &window);
    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) = wait_for_ui_ready(&app, Duration::from_secs(5)).await {
        emit_daemon_error(&app, "main-window-ui-timeout", err, true);
        return;
    }

    let _ = app.emit("show-settings", ());
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
        sync_existing_main_window(&app, &config);
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
    sync_existing_main_window(&app, &config);
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
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    show_settings_window(app_handle).await;
                });
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
    let ui_ready_state: UiReadyState = Arc::new(RwLock::new(false));

    let mut builder = tauri::Builder::default()
        .manage(http_client)
        .manage(cancel_registry)
        .manage(config_state.clone())
        .manage(ui_ready_state)
        .plugin(tauri_plugin_clipboard_manager::init());

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

                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            show_translation_window(app_handle, clipboard_text).await;
                        });
                    }
                })
                .build(),
        );
    }

    builder
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_provider_defaults,
            mark_ui_ready,
            save_config,
            translate::translate_text,
            translate::cancel_translate
        ])
        .setup(|app| {
            if let Err(err) = build_tray(app.app_handle()) {
                emit_daemon_error(app.app_handle(), "tray-build-failed", err, false);
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
