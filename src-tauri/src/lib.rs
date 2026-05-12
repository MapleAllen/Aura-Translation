mod config;
mod hotkey;
mod translate;

use config::AppConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use tokio::sync::Mutex;
use translate::CancellationRegistry;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Tauri command: get current config for the frontend.
#[tauri::command]
fn get_config() -> AppConfig {
    AppConfig::load()
}

/// Tauri command: save config from the frontend and re-register the hotkey.
///
/// On hotkey re-registration failure, emits a `hotkey-conflict` event and
/// keeps the previously registered shortcut active (does NOT return an error
/// to the frontend — a conflict is a user-fixable warning, not a fatal failure).
#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config.save()?;

    // Re-register the global shortcut to reflect any hotkey change
    match hotkey::parse_hotkey(&config.hotkey) {
        Ok(new_shortcut) => {
            // Unregister all previously registered shortcuts first
            if let Err(e) = app.global_shortcut().unregister_all() {
                eprintln!("Failed to unregister shortcuts during re-registration: {}", e);
                // TODO: emit daemon-error when Phase 5 error surface is implemented
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
                    // Note: the shortcut was unregistered above but failed to re-register.
                    // Re-register the default as a safe fallback.
                    if let Ok(fallback) = hotkey::parse_hotkey("CmdOrCtrl+T") {
                        let _ = app.global_shortcut().register(fallback);
                    }
                }
            }
        }
        Err(parse_err) => {
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

    let mut builder = tauri::Builder::default()
        .manage(http_client)
        .manage(cancel_registry)
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
            // Reads AppConfig.hotkey at startup so the binding is always
            // in sync with user preferences — no hardcoded key combination.
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let config = AppConfig::load();
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
