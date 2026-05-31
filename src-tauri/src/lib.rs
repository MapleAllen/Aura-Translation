mod config;
mod hotkey;
mod translate;

use config::{AppConfig, Provider, WindowPlacement};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, PhysicalPosition, State,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tokio::sync::Mutex;
use translate::CancellationRegistry;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg(windows)]
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

const TRANSLATION_WINDOW_LABEL: &str = "translation";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const DEFAULT_TRANSLATION_WIDTH: f64 = 360.0;
const DEFAULT_TRANSLATION_HEIGHT: f64 = 164.0;
const SETTINGS_EDGE_MARGIN: f64 = 18.0;
const TRANSLATION_EDGE_MARGIN: f64 = 14.0;
const TRANSLATION_CURSOR_GAP: f64 = 18.0;

type ConfigState = Arc<RwLock<AppConfig>>;
type UiReadyState = Arc<RwLock<HashSet<String>>>;
type RuntimeState = Arc<Mutex<AppRuntimeState>>;

#[derive(Default)]
struct AppRuntimeState {
    clipboard: ClipboardWatcherState,
    translation: TranslationRuntimeState,
}

#[derive(Default)]
struct ClipboardWatcherState {
    last_sequence: Option<u32>,
    last_dispatched_text: Option<String>,
    suppressed_text: Option<String>,
}

#[derive(Default)]
struct TranslationRuntimeState {
    last_anchor: Option<CursorAnchor>,
    last_requested_text: Option<String>,
}

#[derive(Clone, Copy)]
struct CursorAnchor {
    x: f64,
    y: f64,
}

#[derive(Clone, Serialize)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WindowPlacementKind {
    Settings,
    PinnedTranslation,
}

struct LogicalWorkArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[tauri::command]
fn get_config(state: State<'_, ConfigState>) -> AppConfig {
    state.read().unwrap().clone()
}

#[tauri::command]
fn get_provider_defaults(provider: Provider) -> ProviderDefaults {
    ProviderDefaults {
        base_url: provider.default_base_url().to_string(),
        models: provider.default_models(),
    }
}

#[tauri::command]
fn mark_ui_ready(window: WebviewWindow, state: State<'_, UiReadyState>) {
    state.write().unwrap().insert(window.label().to_string());
}

#[tauri::command]
fn copy_result_to_clipboard(
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    text: String,
) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    app.clipboard()
        .write_text(text.clone())
        .map_err(|e| format!("Failed to write clipboard text: {}", e))?;

    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn(async move {
        runtime.lock().await.clipboard.suppressed_text = Some(text);
    });

    Ok(())
}

#[tauri::command]
fn save_window_placement(
    window: WebviewWindow,
    state: State<'_, ConfigState>,
    kind: WindowPlacementKind,
    mut placement: WindowPlacement,
) -> Result<(), String> {
    placement.monitor = window
        .current_monitor()
        .map_err(|e| format!("Failed to inspect current monitor: {}", e))?
        .and_then(|monitor| monitor.name().cloned());

    let mut next = state.read().unwrap().clone();
    match kind {
        WindowPlacementKind::Settings => next.settings_window_placement = Some(placement),
        WindowPlacementKind::PinnedTranslation => next.pinned_translation_placement = Some(placement),
    }
    next.save()?;
    *state.write().unwrap() = next;
    Ok(())
}

#[tauri::command]
fn realign_translation_window(
    app: AppHandle,
    state: State<'_, ConfigState>,
    runtime: State<'_, RuntimeState>,
) -> Result<(), String> {
    let Some(window) = app.get_webview_window(TRANSLATION_WINDOW_LABEL) else {
        return Ok(());
    };

    let config = state.read().unwrap().clone();
    if config.window_pinned {
        return Ok(());
    }

    position_translation_near_anchor(&window, runtime.inner(), &config)
}

#[tauri::command]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn save_config(
    app: AppHandle,
    state: State<'_, ConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    let old_config = state.read().unwrap().clone();
    let old_hotkey = old_config.hotkey.clone();

    if old_hotkey == config.hotkey {
        persist_config_and_sync(&app, &state, &config)?;
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
    sync_existing_windows(&app, &config);
    emit_config_updated(&app, &config);
    emit_hotkey_registered(&app, &config.hotkey);
    Ok(())
}

#[tauri::command]
#[cfg(any(target_os = "android", target_os = "ios"))]
fn save_config(_app: AppHandle, config: AppConfig) -> Result<(), String> {
    config.save()
}

fn persist_config_and_sync(
    app: &AppHandle,
    state: &State<'_, ConfigState>,
    config: &AppConfig,
) -> Result<(), String> {
    config.save()?;
    *state.write().unwrap() = config.clone();
    sync_existing_windows(app, config);
    emit_config_updated(app, config);
    Ok(())
}

fn current_config(app: &AppHandle) -> AppConfig {
    app.state::<ConfigState>().read().unwrap().clone()
}

fn emit_config_updated(app: &AppHandle, config: &AppConfig) {
    let _ = app.emit("config-updated", config.clone());
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
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let _ = window.emit(
            "hotkey-conflict",
            serde_json::json!({
                "hotkey": hotkey,
                "error": error.into()
            }),
        );
    }
}

fn emit_hotkey_registered(app: &AppHandle, hotkey: &str) {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let _ = window.emit("hotkey-registered", hotkey);
    }
}

fn sync_existing_windows(app: &AppHandle, config: &AppConfig) {
    if let Some(window) = app.get_webview_window(TRANSLATION_WINDOW_LABEL) {
        if let Err(err) = apply_translation_window_preferences(&window, config) {
            emit_daemon_error(app, "translation-window-sync-failed", err, true);
        }
    }
}

fn apply_translation_window_preferences(
    window: &WebviewWindow,
    config: &AppConfig,
) -> Result<(), String> {
    window
        .set_always_on_top(config.window_pinned)
        .map_err(|e| format!("Failed to apply translation pin state: {}", e))
}

fn ensure_window(app: &AppHandle, label: &str) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(label) {
        return Ok(window);
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == label)
        .ok_or_else(|| format!("Window config is missing for '{}'.", label))?;

    app.state::<UiReadyState>().write().unwrap().remove(label);

    let window = WebviewWindowBuilder::from_config(app, config)
        .map_err(|e| format!("Failed to prepare '{}' from config: {}", label, e))?
        .build()
        .map_err(|e| format!("Failed to build '{}': {}", label, e))?;

    attach_window_event_forwarders(app, &window);
    Ok(window)
}

fn attach_window_event_forwarders(app: &AppHandle, window: &WebviewWindow) {
    let label = window.label().to_string();
    let forwarded = window.clone();
    let app_handle = app.clone();

    window.on_window_event(move |event| match event {
        WindowEvent::Focused(false) if label == TRANSLATION_WINDOW_LABEL => {
            let _ = forwarded.emit("window-blur", ());
        }
        WindowEvent::Destroyed => {
            app_handle
                .state::<UiReadyState>()
                .write()
                .unwrap()
                .remove(&label);
        }
        _ => {}
    });
}

async fn wait_for_ui_ready(
    app: &AppHandle,
    label: &str,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if app.state::<UiReadyState>().read().unwrap().contains(label) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    Err(format!(
        "Timed out waiting for '{}' to finish initializing.",
        label
    ))
}

fn logical_work_area(monitor: &tauri::Monitor) -> LogicalWorkArea {
    let scale = monitor.scale_factor();
    let work_area = monitor.work_area();
    LogicalWorkArea {
        x: work_area.position.x as f64 / scale,
        y: work_area.position.y as f64 / scale,
        width: work_area.size.width as f64 / scale,
        height: work_area.size.height as f64 / scale,
    }
}

fn clamp_logical_origin(
    work_area: &LogicalWorkArea,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    margin: f64,
) -> (f64, f64) {
    let min_x = work_area.x + margin;
    let min_y = work_area.y + margin;
    let max_x = (work_area.x + work_area.width - width - margin).max(min_x);
    let max_y = (work_area.y + work_area.height - height - margin).max(min_y);
    (x.clamp(min_x, max_x), y.clamp(min_y, max_y))
}

fn set_window_logical_bounds(
    window: &WebviewWindow,
    width: Option<f64>,
    height: Option<f64>,
    x: f64,
    y: f64,
) {
    if let (Some(width), Some(height)) = (width, height) {
        let _ = window.set_size(LogicalSize::new(width, height));
    }
    let _ = window.set_position(LogicalPosition::new(x, y));
}

fn default_bottom_right_position(window: &WebviewWindow, width: f64, height: f64) -> Option<(f64, f64)> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())?;

    let work_area = logical_work_area(&monitor);
    Some(clamp_logical_origin(
        &work_area,
        width,
        height,
        work_area.x + work_area.width - width - SETTINGS_EDGE_MARGIN,
        work_area.y + work_area.height - height - SETTINGS_EDGE_MARGIN,
        SETTINGS_EDGE_MARGIN,
    ))
}

fn restore_window_from_saved_placement(
    window: &WebviewWindow,
    placement: &WindowPlacement,
    fallback_width: f64,
    fallback_height: f64,
    margin: f64,
) {
    let target_monitor = window
        .available_monitors()
        .ok()
        .and_then(|monitors| {
            placement.monitor.as_ref().and_then(|name| {
                monitors
                    .into_iter()
                    .find(|monitor| monitor.name() == Some(name))
            })
        })
        .or_else(|| {
            window
                .monitor_from_point(placement.x, placement.y)
                .ok()
                .flatten()
        })
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = target_monitor else {
        set_window_logical_bounds(
            window,
            Some(placement.width.unwrap_or(fallback_width)),
            Some(placement.height.unwrap_or(fallback_height)),
            placement.x,
            placement.y,
        );
        return;
    };

    let work_area = logical_work_area(&monitor);
    let width = placement.width.unwrap_or(fallback_width);
    let height = placement.height.unwrap_or(fallback_height);
    let (x, y) = clamp_logical_origin(&work_area, width, height, placement.x, placement.y, margin);
    set_window_logical_bounds(window, Some(width), Some(height), x, y);
}

fn restore_settings_window(window: &WebviewWindow, config: &AppConfig) {
    if let Some(placement) = &config.settings_window_placement {
        restore_window_from_saved_placement(window, placement, 440.0, 640.0, SETTINGS_EDGE_MARGIN);
        return;
    }

    if let Some((x, y)) = default_bottom_right_position(window, 440.0, 640.0) {
        set_window_logical_bounds(window, Some(440.0), Some(640.0), x, y);
    }
}

fn capture_cursor_anchor(window: &WebviewWindow, runtime: &RuntimeState) -> Result<(), String> {
    let position = window
        .cursor_position()
        .map_err(|e| format!("Failed to read cursor position: {}", e))?;

    let runtime = runtime.clone();
    tauri::async_runtime::spawn(async move {
        let mut state = runtime.lock().await;
        state.translation.last_anchor = Some(CursorAnchor {
            x: position.x,
            y: position.y,
        });
    });

    Ok(())
}

fn position_translation_near_anchor(
    window: &WebviewWindow,
    runtime: &RuntimeState,
    config: &AppConfig,
) -> Result<(), String> {
    if config.window_pinned {
        if let Some(placement) = &config.pinned_translation_placement {
            restore_window_from_saved_placement(
                window,
                placement,
                DEFAULT_TRANSLATION_WIDTH,
                DEFAULT_TRANSLATION_HEIGHT,
                SETTINGS_EDGE_MARGIN,
            );
        }
        return Ok(());
    }

    let anchor = {
        let runtime = runtime.blocking_lock();
        runtime.translation.last_anchor
    }
    .ok_or_else(|| "Missing translation anchor point.".to_string())?;

    let monitor = window
        .monitor_from_point(anchor.x, anchor.y)
        .map_err(|e| format!("Failed to locate monitor for cursor anchor: {}", e))?
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "Unable to locate a monitor for the translation window.".to_string())?;

    let outer_size = window
        .outer_size()
        .map_err(|e| format!("Failed to read translation window size: {}", e))?;
    let work_area = monitor.work_area();

    let width = outer_size.width as f64;
    let height = outer_size.height as f64;
    let min_x = work_area.position.x as f64 + TRANSLATION_EDGE_MARGIN;
    let max_x = (work_area.position.x + work_area.size.width as i32) as f64 - width - TRANSLATION_EDGE_MARGIN;
    let min_y = work_area.position.y as f64 + TRANSLATION_EDGE_MARGIN;
    let max_y = (work_area.position.y + work_area.size.height as i32) as f64 - height - TRANSLATION_EDGE_MARGIN;
    let desired_x = anchor.x - (width / 2.0);
    let desired_y = anchor.y - height - TRANSLATION_CURSOR_GAP;

    let x = desired_x.clamp(min_x, max_x.max(min_x));
    let y = desired_y.clamp(min_y, max_y.max(min_y));

    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| format!("Failed to position translation window: {}", e))
}

fn prepare_translation_window_for_new_request(
    window: &WebviewWindow,
    config: &AppConfig,
    runtime: &RuntimeState,
) -> Result<(), String> {
    apply_translation_window_preferences(window, config)?;

    if config.window_pinned {
        if let Some(placement) = &config.pinned_translation_placement {
            restore_window_from_saved_placement(
                window,
                placement,
                DEFAULT_TRANSLATION_WIDTH,
                DEFAULT_TRANSLATION_HEIGHT,
                SETTINGS_EDGE_MARGIN,
            );
        }
        return Ok(());
    }

    let _ = window.set_size(LogicalSize::new(
        DEFAULT_TRANSLATION_WIDTH,
        DEFAULT_TRANSLATION_HEIGHT,
    ));
    capture_cursor_anchor(window, runtime)?;
    position_translation_near_anchor(window, runtime, config)
}

fn prepare_translation_window_for_recall(
    window: &WebviewWindow,
    config: &AppConfig,
    runtime: &RuntimeState,
) -> Result<(), String> {
    apply_translation_window_preferences(window, config)?;

    if config.window_pinned {
        if let Some(placement) = &config.pinned_translation_placement {
            restore_window_from_saved_placement(
                window,
                placement,
                DEFAULT_TRANSLATION_WIDTH,
                DEFAULT_TRANSLATION_HEIGHT,
                SETTINGS_EDGE_MARGIN,
            );
        }
        return Ok(());
    }

    capture_cursor_anchor(window, runtime)?;
    position_translation_near_anchor(window, runtime, config)
}

async fn trigger_translation(app: AppHandle, clipboard_text: String) {
    let window = match ensure_window(&app, TRANSLATION_WINDOW_LABEL) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "translation-window-create-failed", err, false);
            return;
        }
    };

    let config = current_config(&app);
    let runtime = app.state::<RuntimeState>().inner().clone();
    if let Err(err) = prepare_translation_window_for_new_request(&window, &config, &runtime) {
        emit_daemon_error(&app, "translation-window-prepare-failed", err, true);
    }

    {
        let mut state = runtime.lock().await;
        state.translation.last_requested_text = Some(clipboard_text.clone());
        state.clipboard.last_dispatched_text = Some(clipboard_text.clone());
    }

    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) = wait_for_ui_ready(&app, TRANSLATION_WINDOW_LABEL, Duration::from_secs(5)).await {
        emit_daemon_error(&app, "translation-window-ui-timeout", err, true);
        return;
    }

    let _ = window.emit("trigger-translate", clipboard_text);
}

async fn show_existing_translation_window(app: AppHandle) {
    let window = match ensure_window(&app, TRANSLATION_WINDOW_LABEL) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "translation-window-create-failed", err, false);
            return;
        }
    };

    let config = current_config(&app);
    let runtime = app.state::<RuntimeState>().inner().clone();
    if let Err(err) = prepare_translation_window_for_recall(&window, &config, &runtime) {
        emit_daemon_error(&app, "translation-window-recall-failed", err, true);
    }

    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) = wait_for_ui_ready(&app, TRANSLATION_WINDOW_LABEL, Duration::from_secs(5)).await {
        emit_daemon_error(&app, "translation-window-ui-timeout", err, true);
        return;
    }

    let _ = window.emit("show-existing-translation", ());
}

async fn show_settings_window(app: AppHandle) {
    let window = match ensure_window(&app, SETTINGS_WINDOW_LABEL) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "settings-window-create-failed", err, false);
            return;
        }
    };

    let config = current_config(&app);
    restore_settings_window(&window, &config);
    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) = wait_for_ui_ready(&app, SETTINGS_WINDOW_LABEL, Duration::from_secs(5)).await {
        emit_daemon_error(&app, "settings-window-ui-timeout", err, true);
    }
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

#[cfg(windows)]
fn clipboard_sequence_number() -> Option<u32> {
    let sequence = unsafe { GetClipboardSequenceNumber() };
    if sequence == 0 {
        None
    } else {
        Some(sequence)
    }
}

#[cfg(not(windows))]
fn clipboard_sequence_number() -> Option<u32> {
    None
}

fn spawn_clipboard_monitor(app: &AppHandle) {
    #[cfg(not(windows))]
    {
        let _ = app;
    }

    #[cfg(windows)]
    {
        use tauri_plugin_clipboard_manager::ClipboardExt;

        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(275));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let runtime_state = app_handle.state::<RuntimeState>().inner().clone();

            loop {
                interval.tick().await;

                let sequence = clipboard_sequence_number();
                {
                    let mut runtime = runtime_state.lock().await;
                    if sequence.is_some() && runtime.clipboard.last_sequence == sequence {
                        continue;
                    }
                    if sequence.is_some() {
                        runtime.clipboard.last_sequence = sequence;
                    }
                }

                let config = current_config(&app_handle);
                if !config.aura_mode_enabled {
                    continue;
                }

                let text = app_handle.clipboard().read_text().unwrap_or_default();
                let trimmed = text.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                let mut runtime = runtime_state.lock().await;
                if let Some(suppressed) = runtime.clipboard.suppressed_text.as_deref() {
                    if suppressed == trimmed.as_str() {
                        runtime.clipboard.suppressed_text = None;
                        continue;
                    }

                    runtime.clipboard.suppressed_text = None;
                }

                if runtime.clipboard.last_dispatched_text.as_deref() == Some(trimmed.as_str()) {
                    continue;
                }

                runtime.clipboard.last_dispatched_text = Some(trimmed.clone());
                runtime.translation.last_requested_text = Some(trimmed.clone());
                drop(runtime);

                let app_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    trigger_translation(app_clone, trimmed).await;
                });
            }
        });
    }
}

async fn handle_hotkey_pressed(app: AppHandle) {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let config = current_config(&app);
    let existing_window = match ensure_window(&app, TRANSLATION_WINDOW_LABEL) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "translation-window-create-failed", err, false);
            return;
        }
    };

    if config.aura_mode_enabled {
        if matches!(existing_window.is_visible(), Ok(true)) {
            let _ = existing_window.hide();
            return;
        }

        let clipboard_text = app.clipboard().read_text().unwrap_or_default();
        let trimmed = clipboard_text.trim().to_string();
        let last_requested = {
            app.state::<RuntimeState>()
                .lock()
                .await
                .translation
                .last_requested_text
                .clone()
        };

        if !trimmed.is_empty() && last_requested.as_deref() != Some(trimmed.as_str()) {
            trigger_translation(app, trimmed).await;
        } else {
            show_existing_translation_window(app).await;
        }

        return;
    }

    let clipboard_text = app.clipboard().read_text().unwrap_or_default();
    let trimmed = clipboard_text.trim().to_string();
    if trimmed.is_empty() {
        return;
    }

    trigger_translation(app, trimmed).await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let http_client = reqwest::Client::new();
    let cancel_registry: CancellationRegistry = Arc::new(Mutex::new(HashMap::new()));
    let config_state: ConfigState = Arc::new(RwLock::new(AppConfig::load()));
    let ui_ready_state: UiReadyState = Arc::new(RwLock::new(HashSet::new()));
    let runtime_state: RuntimeState = Arc::new(Mutex::new(AppRuntimeState::default()));

    let mut builder = tauri::Builder::default()
        .manage(http_client)
        .manage(cancel_registry)
        .manage(config_state.clone())
        .manage(ui_ready_state)
        .manage(runtime_state)
        .plugin(tauri_plugin_clipboard_manager::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_hotkey_pressed(app_handle).await;
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
            save_window_placement,
            realign_translation_window,
            copy_result_to_clipboard,
            translate::translate_text,
            translate::cancel_translate
        ])
        .setup(|app| {
            if let Err(err) = build_tray(app.app_handle()) {
                emit_daemon_error(app.app_handle(), "tray-build-failed", err, false);
            }

            if let Err(err) = ensure_window(app.app_handle(), TRANSLATION_WINDOW_LABEL) {
                emit_daemon_error(
                    app.app_handle(),
                    "translation-window-startup-create-failed",
                    err,
                    false,
                );
            }

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                register_startup_hotkey(app.app_handle());
            }

            spawn_clipboard_monitor(app.app_handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
