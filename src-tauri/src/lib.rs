mod aura_guard;
mod capabilities;
mod config;
mod history;
mod hotkey;
mod profiles;
mod readiness;
mod secrets;
mod translate;

use capabilities::get_system_capabilities;

use config::{ApiKeyStorage, AppConfig, Provider, WindowPlacement};
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
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

const TRANSLATION_WINDOW_LABEL: &str = "translation";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const TRAY_ID: &str = "main";
const DEFAULT_TRANSLATION_WIDTH: f64 = 392.0;
const DEFAULT_TRANSLATION_HEIGHT: f64 = 188.0;
const DEFAULT_SETTINGS_WIDTH: f64 = 480.0;
const DEFAULT_SETTINGS_HEIGHT: f64 = 680.0;
const SETTINGS_EDGE_MARGIN: f64 = 18.0;
const TRANSLATION_EDGE_MARGIN: f64 = 14.0;
const TRANSLATION_CURSOR_GAP: f64 = 18.0;
const MAX_SUPPRESSED_CLIPBOARD_TEXTS: usize = 4;

type ConfigState = Arc<RwLock<AppConfig>>;
type HistoryState = Arc<Mutex<history::TranslationHistoryStore>>;
type ProfilesState = Arc<RwLock<profiles::TranslationProfilesStore>>;
type ReadinessState = Arc<RwLock<ReadinessProbeCache>>;
type UiReadyState = Arc<RwLock<HashSet<String>>>;
type RuntimeState = Arc<Mutex<AppRuntimeState>>;

const READINESS_PROBE_CACHE_TTL: Duration = Duration::from_secs(30);

fn log_clipboard_action(message: impl AsRef<str>) {
    eprintln!("[aura][clipboard] {}", message.as_ref());
}

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
struct HistoryReplayPayload {
    text: String,
    config: AppConfig,
}

#[derive(Clone, Serialize)]
struct AuraGuardBlockedPayload {
    reason: String,
}

#[derive(Default)]
struct ReadinessProbeCache {
    fingerprint: Option<String>,
    checked_at: Option<Instant>,
    result: Option<readiness::ProviderProbeResult>,
}

struct StartupLoadIssues {
    config: Vec<String>,
    profiles: Vec<String>,
    history: Vec<String>,
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
fn load_provider_api_key(provider: Provider, profile_id: Option<String>) -> Result<String, String> {
    secrets::load_provider_api_key(&provider, profile_id.as_deref())
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
    config_state: State<'_, ConfigState>,
    profiles_state: State<'_, ProfilesState>,
    entry_id: String,
    retry_with_original: bool,
) -> Result<(), String> {
    let history_state = state.inner().clone();
    let entry = history_state
        .lock()
        .await
        .find(&entry_id)
        .ok_or_else(|| format!("History entry '{}' was not found.", entry_id))?;
    if !retry_with_original {
        trigger_translation(app, entry.source_text).await;
        return Ok(());
    }

    let replay_api_key =
        resolve_history_replay_api_key(&entry, &profiles_state.read().unwrap())?;
    let mut request_config = config_state.read().unwrap().clone();
    request_config.provider = entry.provider.clone();
    request_config.model = entry.model;
    request_config.source_lang = entry.source_lang;
    request_config.target_lang = entry.target_lang;
    request_config.api_base_url = if entry.api_base_url.trim().is_empty() {
        entry.provider.default_base_url().to_string()
    } else {
        entry.api_base_url
    };
    request_config.api_key = replay_api_key;
    if let Some(profile_id) = entry.profile_id {
        request_config.active_profile_id = profile_id;
    }

    trigger_translation_with_override(
        app,
        HistoryReplayPayload {
            text: entry.source_text,
            config: request_config,
        },
    )
    .await;
    Ok(())
}

fn resolve_history_replay_api_key(
    entry: &history::TranslationHistoryEntry,
    profiles: &profiles::TranslationProfilesStore,
) -> Result<String, String> {
    if !entry.provider.requires_api_key() {
        return Ok(String::new());
    }

    if let Some(profile) = entry
        .profile_id
        .as_deref()
        .and_then(|profile_id| profiles.find(profile_id))
        .filter(|profile| profile.provider == entry.provider)
    {
        match profile.api_key_storage {
            ApiKeyStorage::PlaintextFallback | ApiKeyStorage::LegacyPlaintext => {
                return Ok(profile.api_key);
            }
            ApiKeyStorage::System => {}
        }
    }

    secrets::load_provider_api_key(&entry.provider, entry.profile_id.as_deref())
}

#[tauri::command]
fn get_runtime_status(state: State<'_, ConfigState>) -> readiness::RuntimeStatus {
    readiness::build_runtime_status(&state.read().unwrap())
}

#[tauri::command]
async fn probe_provider(
    client: State<'_, reqwest::Client>,
    readiness_state: State<'_, ReadinessState>,
    mut config: AppConfig,
) -> Result<readiness::ProviderProbeResult, String> {
    let client = client.inner().clone();
    secrets::hydrate_api_key(&mut config)?;
    let fingerprint = readiness_probe_fingerprint(&config);

    if let Some(cached) = get_cached_probe_result(readiness_state.inner(), &fingerprint) {
        return Ok(cached);
    }

    let result = readiness::probe_provider(&client, &config).await;
    store_probe_result(readiness_state.inner(), fingerprint, result.clone());
    Ok(result)
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

    let char_count = text.chars().count();
    log_clipboard_action(format!("copy requested chars={char_count}"));

    app.clipboard()
        .write_text(text.clone())
        .map_err(|e| {
            log_clipboard_action(format!("copy failed chars={char_count} error={e}"));
            format!("无法写入剪贴板文本：{}", e)
        })?;

    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn(async move {
        let mut state = runtime.lock().await;
        queue_suppressed_clipboard_text(&mut state.clipboard, text);
    });
    log_clipboard_action(format!("copy completed chars={char_count}"));

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
        WindowPlacementKind::PinnedTranslation => {
            next.pinned_translation_placement = Some(placement)
        }
    }
    next.save()?;
    *state.write().unwrap() = next;
    Ok(())
}

#[tauri::command]
async fn realign_translation_window(
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

    position_translation_near_anchor(&window, runtime.inner(), &config).await
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
    invalidate_readiness_probe_cache(app.state::<ReadinessState>().inner());
    sync_existing_windows(app, config);
    sync_tray_runtime_status(app, config)?;
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

fn emit_startup_load_issues(app: &AppHandle, issues: &StartupLoadIssues) {
    for message in &issues.config {
        emit_daemon_error(app, "config-load-failed", message.clone(), true);
        notify_shell_background(
            app,
            "Aura 配置提醒",
            message.clone(),
            &[TRANSLATION_WINDOW_LABEL, SETTINGS_WINDOW_LABEL],
        );
    }

    for message in &issues.profiles {
        emit_daemon_error(app, "profiles-load-failed", message.clone(), true);
        notify_shell_background(
            app,
            "Aura 配置提醒",
            message.clone(),
            &[TRANSLATION_WINDOW_LABEL, SETTINGS_WINDOW_LABEL],
        );
    }

    for message in &issues.history {
        emit_daemon_error(app, "history-load-failed", message.clone(), true);
        notify_shell_background(
            app,
            "Aura 配置提醒",
            message.clone(),
            &[TRANSLATION_WINDOW_LABEL, SETTINGS_WINDOW_LABEL],
        );
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

fn readiness_probe_fingerprint(config: &AppConfig) -> String {
    format!(
        "{:?}\n{}\n{}\n{}",
        config.provider,
        config.api_base_url.trim(),
        config.model.trim(),
        config.api_key.trim()
    )
}

fn get_cached_probe_result(
    state: &ReadinessState,
    fingerprint: &str,
) -> Option<readiness::ProviderProbeResult> {
    let cache = state.read().unwrap();
    if cache.fingerprint.as_deref() != Some(fingerprint) {
        return None;
    }

    let checked_at = cache.checked_at?;
    if checked_at.elapsed() > READINESS_PROBE_CACHE_TTL {
        return None;
    }

    cache.result.clone()
}

fn store_probe_result(
    state: &ReadinessState,
    fingerprint: String,
    result: readiness::ProviderProbeResult,
) {
    let mut cache = state.write().unwrap();
    cache.fingerprint = Some(fingerprint);
    cache.checked_at = Some(Instant::now());
    cache.result = Some(result);
}

fn invalidate_readiness_probe_cache(state: &ReadinessState) {
    *state.write().unwrap() = ReadinessProbeCache::default();
}

fn sync_tray_runtime_status(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };

    let summary = readiness::build_runtime_status(config).summary;
    tray.set_tooltip(Some(summary))
        .map_err(|e| format!("Failed to refresh the tray tooltip: {}", e))
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
        restore_window_from_saved_placement(
            window,
            placement,
            DEFAULT_SETTINGS_WIDTH,
            DEFAULT_SETTINGS_HEIGHT,
            SETTINGS_EDGE_MARGIN,
        );
        return;
    }

    if let Some((x, y)) =
        default_bottom_right_position(window, DEFAULT_SETTINGS_WIDTH, DEFAULT_SETTINGS_HEIGHT)
    {
        set_window_logical_bounds(
            window,
            Some(DEFAULT_SETTINGS_WIDTH),
            Some(DEFAULT_SETTINGS_HEIGHT),
            x,
            y,
        );
    }
}

fn should_show_settings_on_startup(config: &AppConfig) -> bool {
    #[cfg(target_os = "macos")]
    {
        readiness::should_prompt_for_setup(config) || config.settings_window_placement.is_none()
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = config;
        false
    }
}

async fn capture_cursor_anchor(
    window: &WebviewWindow,
    runtime: &RuntimeState,
) -> Result<(), String> {
    let position = window
        .cursor_position()
        .map_err(|e| format!("Failed to read cursor position: {}", e))?;

    let mut state = runtime.lock().await;
    state.translation.last_anchor = Some(CursorAnchor {
        x: position.x,
        y: position.y,
    });

    Ok(())
}

async fn position_translation_near_anchor(
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
        let runtime = runtime.lock().await;
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

async fn prepare_translation_window_for_new_request(
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
    capture_cursor_anchor(window, runtime).await?;
    position_translation_near_anchor(window, runtime, config).await
}

async fn prepare_translation_window_for_recall(
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

    capture_cursor_anchor(window, runtime).await?;
    position_translation_near_anchor(window, runtime, config).await
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
    if let Err(err) = prepare_translation_window_for_new_request(&window, &config, &runtime).await {
        emit_daemon_error(&app, "translation-window-prepare-failed", err, true);
    }

    {
        let mut state = runtime.lock().await;
        state.translation.last_requested_text = Some(clipboard_text.clone());
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

async fn trigger_translation_with_override(app: AppHandle, payload: HistoryReplayPayload) {
    let window = match ensure_window(&app, TRANSLATION_WINDOW_LABEL) {
        Ok(window) => window,
        Err(err) => {
            emit_daemon_error(&app, "translation-window-create-failed", err, false);
            return;
        }
    };

    let runtime = app.state::<RuntimeState>().inner().clone();
    if let Err(err) =
        prepare_translation_window_for_new_request(&window, &current_config(&app), &runtime).await
    {
        emit_daemon_error(&app, "translation-window-prepare-failed", err, true);
    }

    let _ = window.show();
    let _ = window.set_focus();

    if let Err(err) =
        wait_for_ui_ready(&app, TRANSLATION_WINDOW_LABEL, Duration::from_secs(5)).await
    {
        emit_daemon_error(&app, "translation-window-ui-timeout", err, true);
        return;
    }

    let _ = window.emit("trigger-translate-with-override", payload);
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
    if let Err(err) = prepare_translation_window_for_recall(&window, &config, &runtime).await {
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
    let profiles_submenu = Submenu::with_items(app, "配置方案", true, &profile_refs)
        .map_err(|e| format!("Failed to build tray Profiles submenu: {}", e))?;
    let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
        .map_err(|e| format!("Failed to build tray Settings item: {}", e))?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| format!("Failed to build tray separator: {}", e))?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
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
    let tooltip = readiness::build_runtime_status(&current_config(app)).summary;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip(tooltip)
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
                        "Failed to register startup hotkey '{}': {}. Falling back to {}.",
                        config.hotkey,
                        e,
                        hotkey::default_hotkey()
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
                    "Failed to parse startup hotkey '{}': {}. Falling back to {}.",
                    config.hotkey,
                    e,
                    hotkey::default_hotkey()
                ),
                true,
            );
            register_fallback_hotkey(app);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn register_fallback_hotkey(app: &AppHandle) {
    let fallback_hotkey = hotkey::default_hotkey();
    match hotkey::parse_hotkey(fallback_hotkey) {
        Ok(fallback) => {
            if let Err(e) = app.global_shortcut().register(fallback) {
                emit_daemon_error(
                    app,
                    "hotkey-fallback-register-failed",
                    format!(
                        "Failed to register {} fallback hotkey: {}",
                        fallback_hotkey, e
                    ),
                    false,
                );
            }
        }
        Err(e) => {
            emit_daemon_error(
                app,
                "hotkey-fallback-parse-failed",
                format!(
                    "Failed to parse {} fallback hotkey: {}",
                    fallback_hotkey, e
                ),
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


fn spawn_clipboard_monitor(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_clipboard_manager::ClipboardExt;
        use objc2_app_kit::NSPasteboard;

        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(275));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let runtime_state = app_handle.state::<RuntimeState>().inner().clone();

            loop {
                interval.tick().await;

                let sequence = {
                    let pb = NSPasteboard::generalPasteboard();
                    pb.changeCount() as u32
                };

                {
                    let mut runtime = runtime_state.lock().await;
                    if runtime.clipboard.last_sequence == Some(sequence) {
                        continue;
                    }
                    runtime.clipboard.last_sequence = Some(sequence);
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

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = app;
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
        emit_daemon_error(
            &app,
            "hotkey-empty-clipboard",
            "请先复制要翻译的文本，然后按快捷键。",
            true,
        );
        return;
    }

    trigger_translation(app, trimmed).await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let http_client = reqwest::Client::new();
    let cancel_registry: CancellationRegistry = Arc::new(Mutex::new(HashMap::new()));
    let (config, config_load_issues) = AppConfig::load_with_issues();
    let (history, history_load_issues) = history::TranslationHistoryStore::load_with_issues();
    let (profiles, profiles_load_issues) = profiles::TranslationProfilesStore::load_with_issues();
    let startup_load_issues = StartupLoadIssues {
        config: config_load_issues,
        profiles: profiles_load_issues,
        history: history_load_issues,
    };
    let config_state: ConfigState = Arc::new(RwLock::new(config));
    let history_state: HistoryState = Arc::new(Mutex::new(history));
    let profiles_state: ProfilesState = Arc::new(RwLock::new(profiles));
    let readiness_state: ReadinessState = Arc::new(RwLock::new(ReadinessProbeCache::default()));
    let ui_ready_state: UiReadyState = Arc::new(RwLock::new(HashSet::new()));
    let runtime_state: RuntimeState = Arc::new(Mutex::new(AppRuntimeState::default()));

    let mut builder = tauri::Builder::default()
        .manage(http_client)
        .manage(cancel_registry)
        .manage(config_state.clone())
        .manage(history_state)
        .manage(profiles_state)
        .manage(readiness_state)
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
            get_system_capabilities,
            load_provider_api_key,
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
            translate::translate_text,
            translate::cancel_translate
        ])
        .setup(move |app| {
            if let Err(err) = initialize_profiles_and_secrets(app.app_handle()) {
                emit_daemon_error(app.app_handle(), "secret-storage-init-failed", err, true);
            }

            let startup_config = current_config(app.app_handle());
            emit_startup_load_issues(app.app_handle(), &startup_load_issues);

            if let Err(err) = build_tray(app.app_handle()) {
                emit_daemon_error(app.app_handle(), "tray-build-failed", err, false);
            }

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                register_startup_hotkey(app.app_handle());
            }

            spawn_clipboard_monitor(app.app_handle());

            if should_show_settings_on_startup(&startup_config) {
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

#[cfg(all(test, target_os = "macos"))]
mod startup_tests {
    use super::should_show_settings_on_startup;
    use crate::config::AppConfig;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_shows_settings_on_first_launch() {
        let config = AppConfig::default();
        assert!(should_show_settings_on_startup(&config));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_hides_startup_settings_after_setup_and_saved_placement() {
        let mut config = AppConfig::default();
        config.api_key = "sk-test".to_string();
        config.settings_window_placement = Some(crate::config::WindowPlacement {
            x: 120.0,
            y: 160.0,
            width: Some(480.0),
            height: Some(680.0),
            monitor: Some("Built-in".to_string()),
        });

        assert!(!should_show_settings_on_startup(&config));
    }
}

#[cfg(test)]
mod p2_tests {
    use super::resolve_history_replay_api_key;
    use crate::config::{ApiKeyStorage, AppConfig, Provider};
    use crate::history::{TranslationHistoryEntry, TranslationHistoryStatus};
    use crate::profiles::{TranslationProfile, TranslationProfilesStore};

    #[test]
    fn history_replay_uses_original_plaintext_profile_key_without_activation() {
        let active_config = AppConfig::default();
        let entry = TranslationHistoryEntry {
            id: "history-1".to_string(),
            source_text: "hello".to_string(),
            translated_text: "你好".to_string(),
            error_message: None,
            source_lang: "English".to_string(),
            target_lang: "Chinese".to_string(),
            provider: Provider::OpenRouter,
            model: "mistral".to_string(),
            api_base_url: "https://openrouter.ai/api".to_string(),
            profile_id: Some("archived-openrouter".to_string()),
            usage: None,
            status: TranslationHistoryStatus::Success,
            created_at_ms: 1,
        };
        let profiles = TranslationProfilesStore {
            active_profile_id: active_config.active_profile_id.clone(),
            profiles: vec![TranslationProfile {
                id: "archived-openrouter".to_string(),
                name: "Archived OpenRouter".to_string(),
                api_key: "sk-original".to_string(),
                api_key_storage: ApiKeyStorage::PlaintextFallback,
                model: "mistral".to_string(),
                source_lang: "English".to_string(),
                target_lang: "Chinese".to_string(),
                provider: Provider::OpenRouter,
                api_base_url: "https://openrouter.ai/api".to_string(),
                available_models: vec!["mistral".to_string()],
            }],
        };

        assert_eq!(
            resolve_history_replay_api_key(&entry, &profiles).unwrap(),
            "sk-original"
        );
        assert_eq!(profiles.active_profile_id, active_config.active_profile_id);
    }
}
