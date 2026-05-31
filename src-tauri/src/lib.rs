mod aura_guard;
mod config;
mod history;
mod hotkey;
mod profiles;
mod readiness;
mod secrets;
mod translate;

use config::{AppConfig, Provider, WindowPlacement};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{
    menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, PhysicalPosition, State,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use tokio::sync::Mutex;
use translate::CancellationRegistry;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_notification::NotificationExt;

#[cfg(windows)]
use windows::Win32::{
    Foundation::HWND,
    System::DataExchange::GetClipboardSequenceNumber,
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
            VK_V,
        },
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow,
            SetForegroundWindow, ShowWindow, SW_RESTORE,
        },
    },
};

const TRANSLATION_WINDOW_LABEL: &str = "translation";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const TRAY_ID: &str = "main";
const DEFAULT_TRANSLATION_WIDTH: f64 = 360.0;
const DEFAULT_TRANSLATION_HEIGHT: f64 = 164.0;
const SETTINGS_EDGE_MARGIN: f64 = 18.0;
const TRANSLATION_EDGE_MARGIN: f64 = 14.0;
const TRANSLATION_CURSOR_GAP: f64 = 18.0;
const MAX_SUPPRESSED_CLIPBOARD_TEXTS: usize = 4;

type ConfigState = Arc<RwLock<AppConfig>>;
type HistoryState = Arc<Mutex<history::TranslationHistoryStore>>;
type ProfilesState = Arc<RwLock<profiles::TranslationProfilesStore>>;
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
    suppressed_texts: Vec<String>,
}

#[derive(Default)]
struct TranslationRuntimeState {
    last_anchor: Option<CursorAnchor>,
    last_requested_text: Option<String>,
    paste_back_window: Option<isize>,
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

#[derive(Clone, Serialize)]
struct AuraGuardBlockedPayload {
    reason: String,
}

#[derive(Clone, Serialize)]
struct PasteBackStatus {
    supported: bool,
    available: bool,
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
fn load_provider_api_key(provider: Provider) -> Result<String, String> {
    secrets::load_provider_api_key(&provider)
}

#[tauri::command]
async fn get_paste_back_status(state: State<'_, RuntimeState>) -> Result<PasteBackStatus, String> {
    let runtime_state = state.inner().clone();
    let mut runtime = runtime_state.lock().await;
    let available = runtime
        .translation
        .paste_back_window
        .map(is_valid_paste_back_window)
        .unwrap_or(false);

    if runtime.translation.paste_back_window.is_some() && !available {
        runtime.translation.paste_back_window = None;
    }

    Ok(PasteBackStatus {
        supported: paste_back_supported(),
        available,
    })
}

#[tauri::command]
fn get_translation_profiles(state: State<'_, ProfilesState>) -> profiles::TranslationProfilesStore {
    state.read().unwrap().snapshot()
}

#[tauri::command]
fn create_translation_profile(
    app: AppHandle,
    config_state: State<'_, ConfigState>,
    profiles_state: State<'_, ProfilesState>,
    mut config: AppConfig,
    name: String,
) -> Result<profiles::TranslationProfilesStore, String> {
    let old_config = config_state.read().unwrap().clone();
    profiles_state
        .write()
        .unwrap()
        .create_and_activate(name.trim(), &mut config)?;
    persist_config_and_sync(
        &app,
        &config_state,
        &profiles_state,
        &old_config,
        &mut config,
    )?;
    sync_tray_menu(&app)?;
    Ok(profiles_state.read().unwrap().snapshot())
}

#[tauri::command]
fn rename_translation_profile(
    app: AppHandle,
    profiles_state: State<'_, ProfilesState>,
    profile_id: String,
    name: String,
) -> Result<profiles::TranslationProfilesStore, String> {
    profiles_state
        .write()
        .unwrap()
        .rename(&profile_id, name.trim())?;
    sync_tray_menu(&app)?;
    Ok(profiles_state.read().unwrap().snapshot())
}

#[tauri::command]
fn activate_translation_profile(
    app: AppHandle,
    config_state: State<'_, ConfigState>,
    profiles_state: State<'_, ProfilesState>,
    profile_id: String,
) -> Result<profiles::TranslationProfilesStore, String> {
    let old_config = config_state.read().unwrap().clone();
    let mut next_config = old_config.clone();
    profiles_state
        .write()
        .unwrap()
        .activate(&profile_id, &mut next_config)?;
    secrets::hydrate_api_key(&mut next_config)?;
    persist_config_and_sync(
        &app,
        &config_state,
        &profiles_state,
        &old_config,
        &mut next_config,
    )?;
    sync_tray_menu(&app)?;
    Ok(profiles_state.read().unwrap().snapshot())
}

#[tauri::command]
fn delete_translation_profile(
    app: AppHandle,
    config_state: State<'_, ConfigState>,
    profiles_state: State<'_, ProfilesState>,
    profile_id: String,
) -> Result<profiles::TranslationProfilesStore, String> {
    let old_config = config_state.read().unwrap().clone();
    let mut next_config = old_config.clone();
    profiles_state
        .write()
        .unwrap()
        .delete(&profile_id, &mut next_config)?;
    secrets::hydrate_api_key(&mut next_config)?;
    persist_config_and_sync(
        &app,
        &config_state,
        &profiles_state,
        &old_config,
        &mut next_config,
    )?;
    sync_tray_menu(&app)?;
    Ok(profiles_state.read().unwrap().snapshot())
}

#[tauri::command]
async fn get_translation_history(
    state: State<'_, HistoryState>,
) -> Result<Vec<history::TranslationHistoryEntry>, String> {
    let history_state = state.inner().clone();
    let entries = history_state.lock().await.list();
    Ok(entries)
}

#[tauri::command]
async fn delete_translation_history_entry(
    state: State<'_, HistoryState>,
    entry_id: String,
) -> Result<Vec<history::TranslationHistoryEntry>, String> {
    let history_state = state.inner().clone();
    let entries = history_state.lock().await.delete(&entry_id)?;
    Ok(entries)
}

#[tauri::command]
async fn clear_translation_history(
    state: State<'_, HistoryState>,
) -> Result<Vec<history::TranslationHistoryEntry>, String> {
    let history_state = state.inner().clone();
    let entries = history_state.lock().await.clear()?;
    Ok(entries)
}

#[tauri::command]
async fn replay_translation_history_entry(
    app: AppHandle,
    state: State<'_, HistoryState>,
    entry_id: String,
) -> Result<(), String> {
    let history_state = state.inner().clone();
    let entry = history_state
        .lock()
        .await
        .find(&entry_id)
        .ok_or_else(|| format!("History entry '{}' was not found.", entry_id))?;
    trigger_translation(app, entry.source_text).await;
    Ok(())
}

#[tauri::command]
fn get_runtime_status(state: State<'_, ConfigState>) -> readiness::RuntimeStatus {
    readiness::build_runtime_status(&state.read().unwrap())
}

#[tauri::command]
async fn probe_provider(
    client: State<'_, reqwest::Client>,
    mut config: AppConfig,
) -> Result<readiness::ProviderProbeResult, String> {
    let client = client.inner().clone();
    secrets::hydrate_api_key(&mut config)?;
    Ok(readiness::probe_provider(&client, &config).await)
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
        let mut state = runtime.lock().await;
        queue_suppressed_clipboard_text(&mut state.clipboard, text);
    });

    Ok(())
}

#[tauri::command]
async fn paste_translation_back(
    app: AppHandle,
    runtime: State<'_, RuntimeState>,
    text: String,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use tauri_plugin_clipboard_manager::ClipboardExt;
        let runtime_state = runtime.inner().clone();

        let translated_text = text.trim().to_string();
        if translated_text.is_empty() {
            return Err("There is no translated text to paste back yet.".to_string());
        }

        let target_window = {
            let mut state = runtime_state.lock().await;
            let Some(target_window) = state.translation.paste_back_window else {
                return Err("Aura has not captured a source window for paste-back yet.".to_string());
            };

            if !is_valid_paste_back_window(target_window) {
                state.translation.paste_back_window = None;
                return Err("The original source window is no longer available for paste-back.".to_string());
            }

            target_window
        };

        let previous_clipboard = app.clipboard().read_text().ok();

        {
            let mut state = runtime_state.lock().await;
            queue_suppressed_clipboard_text(&mut state.clipboard, translated_text.clone());
        }

        app.clipboard()
            .write_text(translated_text.clone())
            .map_err(|e| format!("Failed to prepare the clipboard for paste-back: {}", e))?;

        tokio::time::sleep(Duration::from_millis(40)).await;
        let paste_result = focus_window_for_paste_back(target_window).and_then(|_| send_ctrl_v());
        tokio::time::sleep(Duration::from_millis(120)).await;

        if let Some(previous_clipboard) = previous_clipboard.filter(|value| value != &translated_text)
        {
            {
                let mut state = runtime_state.lock().await;
                queue_suppressed_clipboard_text(&mut state.clipboard, previous_clipboard.clone());
            }

            if let Err(err) = app.clipboard().write_text(previous_clipboard) {
                emit_daemon_error(
                    &app,
                    "pasteback-clipboard-restore-failed",
                    format!(
                        "Aura pasted the translation, but failed to restore the previous clipboard text: {}",
                        err
                    ),
                    true,
                );
            }
        }

        paste_result
    }

    #[cfg(not(windows))]
    {
        let _ = (app, runtime, text);
        Err("Paste-back is currently supported on Windows only.".to_string())
    }
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
        WindowPlacementKind::PinnedTranslation => {
            next.pinned_translation_placement = Some(placement)
        }
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
    profiles_state: State<'_, ProfilesState>,
    config: AppConfig,
) -> Result<(), String> {
    let old_config = state.read().unwrap().clone();
    let old_hotkey = old_config.hotkey.clone();
    let mut next_config = config;

    if old_hotkey == next_config.hotkey {
        persist_config_and_sync(&app, &state, &profiles_state, &old_config, &mut next_config)?;
        sync_tray_menu(&app)?;
        return Ok(());
    }

    let new_shortcut = match hotkey::parse_hotkey(&next_config.hotkey) {
        Ok(shortcut) => shortcut,
        Err(parse_err) => {
            emit_hotkey_conflict(&app, &next_config.hotkey, parse_err.clone());
            return Err(format!("Hotkey save rejected: {}", parse_err));
        }
    };

    if let Err(e) = app.global_shortcut().unregister_all() {
        emit_daemon_error(
            &app,
            "hotkey-unregister-failed",
            format!(
                "Failed to unregister the previous hotkey before saving '{}': {}",
                next_config.hotkey, e
            ),
            true,
        );
        return Err(format!("Failed to prepare hotkey update: {}", e));
    }

    if let Err(e) = app.global_shortcut().register(new_shortcut) {
        emit_hotkey_conflict(&app, &next_config.hotkey, e.to_string());
        restore_previous_hotkey(&app, &old_hotkey);
        return Err(format!("Hotkey save rejected: {}", e));
    }

    if let Err(e) =
        persist_config_and_sync(&app, &state, &profiles_state, &old_config, &mut next_config)
    {
        emit_daemon_error(
            &app,
            "config-or-secret-save-failed",
            format!(
                "Failed to persist config or API key state after registering hotkey '{}': {}",
                next_config.hotkey, e
            ),
            false,
        );

        if let Err(unregister_err) = app.global_shortcut().unregister_all() {
            emit_daemon_error(
                &app,
                "hotkey-unregister-after-save-failure",
                format!(
                    "Failed to unregister the new hotkey '{}' after a config save failure: {}",
                    next_config.hotkey, unregister_err
                ),
                false,
            );
        }

        restore_previous_hotkey(&app, &old_hotkey);
        return Err(format!("Failed to save config: {}", e));
    }

    sync_tray_menu(&app)?;
    emit_hotkey_registered(&app, &next_config.hotkey);
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
    profiles_state: &State<'_, ProfilesState>,
    old_config: &AppConfig,
    config: &mut AppConfig,
) -> Result<(), String> {
    secrets::persist_api_key(config, old_config)?;
    config.save()?;
    profiles_state
        .write()
        .unwrap()
        .sync_active_profile_from_config(config)?;
    *state.write().unwrap() = config.clone();
    sync_existing_windows(app, config);
    emit_config_updated(app, config);
    Ok(())
}

fn current_config(app: &AppHandle) -> AppConfig {
    app.state::<ConfigState>().read().unwrap().clone()
}

fn initialize_profiles_and_secrets(app: &AppHandle) -> Result<(), String> {
    let config_state = app.state::<ConfigState>();
    let profiles_state = app.state::<ProfilesState>();
    let mut config = config_state.read().unwrap().clone();

    profiles_state
        .write()
        .unwrap()
        .ensure_seeded_from_config(&mut config)?;

    let migrated = secrets::migrate_legacy_plaintext_key(&mut config)?;
    secrets::hydrate_api_key(&mut config)?;
    profiles_state
        .write()
        .unwrap()
        .sync_active_profile_from_config(&config)?;

    if migrated {
        config.save()?;
    }

    *config_state.write().unwrap() = config.clone();

    if migrated {
        emit_config_updated(app, &config);
    }

    Ok(())
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

fn emit_aura_guard_blocked(app: &AppHandle, reason: impl Into<String>) {
    let reason = reason.into();
    let _ = app.emit(
        "aura-guard-blocked",
        AuraGuardBlockedPayload {
            reason: reason.clone(),
        },
    );

    notify_shell_background(
        app,
        "Sensitive clipboard skipped",
        reason,
        &[TRANSLATION_WINDOW_LABEL, SETTINGS_WINDOW_LABEL],
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

fn queue_suppressed_clipboard_text(state: &mut ClipboardWatcherState, text: impl Into<String>) {
    let text = text.into().trim().to_string();
    if text.is_empty() || state.suppressed_texts.iter().any(|entry| entry == &text) {
        return;
    }

    state.suppressed_texts.push(text);
    if state.suppressed_texts.len() > MAX_SUPPRESSED_CLIPBOARD_TEXTS {
        let overflow = state.suppressed_texts.len() - MAX_SUPPRESSED_CLIPBOARD_TEXTS;
        state.suppressed_texts.drain(0..overflow);
    }
}

fn consume_suppressed_clipboard_text(state: &mut ClipboardWatcherState, text: &str) -> bool {
    if let Some(index) = state.suppressed_texts.iter().position(|entry| entry == text) {
        state.suppressed_texts.remove(index);
        true
    } else {
        false
    }
}

#[cfg(windows)]
const fn paste_back_supported() -> bool {
    true
}

#[cfg(not(windows))]
const fn paste_back_supported() -> bool {
    false
}

#[cfg(windows)]
fn current_foreground_window_handle() -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == std::process::id() {
            return None;
        }

        Some(hwnd.0 as isize)
    }
}

#[cfg(not(windows))]
fn current_foreground_window_handle() -> Option<isize> {
    None
}

#[cfg(windows)]
fn is_valid_paste_back_window(handle: isize) -> bool {
    unsafe { IsWindow(Some(HWND(handle as *mut _))).as_bool() }
}

#[cfg(not(windows))]
fn is_valid_paste_back_window(_handle: isize) -> bool {
    false
}

#[cfg(windows)]
fn focus_window_for_paste_back(handle: isize) -> Result<(), String> {
    unsafe {
        let hwnd = HWND(handle as *mut _);
        if !IsWindow(Some(hwnd)).as_bool() {
            return Err("The original source window is no longer available for paste-back.".to_string());
        }

        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }

        if GetForegroundWindow().0 != hwnd.0 && !SetForegroundWindow(hwnd).as_bool() {
            return Err("Failed to focus the original source window for paste-back.".to_string());
        }
    }

    std::thread::sleep(Duration::from_millis(40));
    Ok(())
}

#[cfg(windows)]
fn send_ctrl_v() -> Result<(), String> {
    let inputs = [
        keyboard_input(VK_CONTROL, Default::default()),
        keyboard_input(VK_V, Default::default()),
        keyboard_input(VK_V, KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ];

    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err("Failed to send Ctrl+V to the original source window.".to_string());
    }

    Ok(())
}

#[cfg(windows)]
fn keyboard_input(
    key: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS,
) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

fn is_window_visible(app: &AppHandle, label: &str) -> bool {
    app.get_webview_window(label)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn notify_shell_background(
    app: &AppHandle,
    title: &str,
    body: impl Into<String>,
    window_labels: &[&str],
) {
    if window_labels
        .iter()
        .any(|label| is_window_visible(app, label))
    {
        return;
    }

    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body.into())
        .show();
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn notify_shell_background(
    _app: &AppHandle,
    _title: &str,
    _body: impl Into<String>,
    _window_labels: &[&str],
) {
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

async fn wait_for_ui_ready(app: &AppHandle, label: &str, timeout: Duration) -> Result<(), String> {
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

fn default_bottom_right_position(
    window: &WebviewWindow,
    width: f64,
    height: f64,
) -> Option<(f64, f64)> {
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
    let max_x = (work_area.position.x + work_area.size.width as i32) as f64
        - width
        - TRANSLATION_EDGE_MARGIN;
    let min_y = work_area.position.y as f64 + TRANSLATION_EDGE_MARGIN;
    let max_y = (work_area.position.y + work_area.size.height as i32) as f64
        - height
        - TRANSLATION_EDGE_MARGIN;
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
    let paste_back_window = current_foreground_window_handle();
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
        state.translation.paste_back_window = paste_back_window;
        state.clipboard.last_dispatched_text = Some(clipboard_text.clone());
    }

    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) =
        wait_for_ui_ready(&app, TRANSLATION_WINDOW_LABEL, Duration::from_secs(5)).await
    {
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

    if let Err(err) =
        wait_for_ui_ready(&app, TRANSLATION_WINDOW_LABEL, Duration::from_secs(5)).await
    {
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

async fn handle_tray_primary_action(app: AppHandle) {
    if readiness::should_prompt_for_setup(&current_config(&app)) {
        show_settings_window(app).await;
        return;
    }

    show_existing_translation_window(app).await;
}

fn activate_profile_by_id(app: &AppHandle, profile_id: &str) -> Result<(), String> {
    let config_state = app.state::<ConfigState>();
    let profiles_state = app.state::<ProfilesState>();
    let old_config = config_state.read().unwrap().clone();
    let mut next_config = old_config.clone();
    profiles_state
        .write()
        .unwrap()
        .activate(profile_id, &mut next_config)?;
    secrets::hydrate_api_key(&mut next_config)?;
    persist_config_and_sync(
        app,
        &config_state,
        &profiles_state,
        &old_config,
        &mut next_config,
    )?;
    sync_tray_menu(app)?;
    Ok(())
}

fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let profiles_snapshot = app.state::<ProfilesState>().read().unwrap().snapshot();
    let mut profile_items = Vec::new();
    for profile in &profiles_snapshot.profiles {
        profile_items.push(
            CheckMenuItem::with_id(
                app,
                format!("profile:{}", profile.id),
                &profile.name,
                true,
                profile.id == profiles_snapshot.active_profile_id,
                None::<&str>,
            )
            .map_err(|e| {
                format!(
                    "Failed to build tray Profile item '{}': {}",
                    profile.name, e
                )
            })?,
        );
    }

    let profile_refs: Vec<&dyn IsMenuItem<_>> = profile_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<_>)
        .collect();
    let profiles_submenu = Submenu::with_items(app, "Profiles", true, &profile_refs)
        .map_err(|e| format!("Failed to build tray Profiles submenu: {}", e))?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| format!("Failed to build tray Settings item: {}", e))?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to build tray separator: {}", e))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| format!("Failed to build tray Quit item: {}", e))?;

    Menu::with_items(
        app,
        &[&profiles_submenu, &settings_item, &separator, &quit_item],
    )
    .map_err(|e| format!("Failed to build tray menu: {}", e))
}

fn sync_tray_menu(app: &AppHandle) -> Result<(), String> {
    let menu = build_tray_menu(app)?;
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };

    tray.set_menu(Some(menu))
        .map_err(|e| format!("Failed to refresh the tray menu: {}", e))
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
    let menu = build_tray_menu(app)?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "Default window icon is missing.".to_string())?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("Aura Translation")
        .show_menu_on_left_click(false)
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
            id if id.starts_with("profile:") => {
                let app_handle = app.clone();
                let profile_id = id.trim_start_matches("profile:").to_string();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = activate_profile_by_id(&app_handle, &profile_id) {
                        emit_daemon_error(&app_handle, "profile-activate-failed", err, true);
                    }
                });
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app_handle = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    handle_tray_primary_action(app_handle).await;
                });
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

                if config.aura_guard_enabled {
                    if let Some(block) = aura_guard::detect_sensitive_clipboard(&trimmed) {
                        emit_aura_guard_blocked(&app_handle, block.reason);
                        continue;
                    }
                }

                let mut runtime = runtime_state.lock().await;
                if consume_suppressed_clipboard_text(&mut runtime.clipboard, trimmed.as_str()) {
                    continue;
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
    let history_state: HistoryState =
        Arc::new(Mutex::new(history::TranslationHistoryStore::load()));
    let profiles_state: ProfilesState =
        Arc::new(RwLock::new(profiles::TranslationProfilesStore::load()));
    let ui_ready_state: UiReadyState = Arc::new(RwLock::new(HashSet::new()));
    let runtime_state: RuntimeState = Arc::new(Mutex::new(AppRuntimeState::default()));

    let mut builder = tauri::Builder::default()
        .manage(http_client)
        .manage(cancel_registry)
        .manage(config_state.clone())
        .manage(history_state)
        .manage(profiles_state)
        .manage(ui_ready_state)
        .manage(runtime_state)
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init());

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
            load_provider_api_key,
            get_paste_back_status,
            get_translation_profiles,
            create_translation_profile,
            rename_translation_profile,
            activate_translation_profile,
            delete_translation_profile,
            get_translation_history,
            delete_translation_history_entry,
            clear_translation_history,
            replay_translation_history_entry,
            get_runtime_status,
            probe_provider,
            mark_ui_ready,
            save_config,
            save_window_placement,
            realign_translation_window,
            copy_result_to_clipboard,
            paste_translation_back,
            translate::translate_text,
            translate::cancel_translate
        ])
        .setup(|app| {
            if let Err(err) = initialize_profiles_and_secrets(app.app_handle()) {
                emit_daemon_error(app.app_handle(), "secret-storage-init-failed", err, true);
            }

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

            if readiness::should_prompt_for_setup(&current_config(app.app_handle())) {
                let app_handle = app.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    show_settings_window(app_handle).await;
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
