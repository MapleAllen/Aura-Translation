mod config;
mod translate;

use config::AppConfig;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Tauri command: get current config for the frontend.
#[tauri::command]
fn get_config() -> AppConfig {
    AppConfig::load()
}

/// Tauri command: save config from the frontend.
#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    config.save()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
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
            translate::translate_text
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

            // ── Register Global Shortcut ─────────────────────────────
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyT);
                app.global_shortcut().register(shortcut).map_err(|e| {
                    eprintln!("Failed to register global shortcut Ctrl+T: {}", e);
                    e
                })?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
