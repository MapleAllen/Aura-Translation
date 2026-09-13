//! The automatic clipboard monitor.
//!
//! The loop and its dispatch rules are platform independent; only "how do I observe the
//! pasteboard?" differs. Keeping that difference behind [`ClipboardObserver`] means the rules exist
//! once. They were previously written twice, in two `#[cfg]` blocks that had to be kept in step by
//! hand — and drifted: the resume-baseline defect was fixed in one shape and had to be fixed again
//! in the other.

use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::interaction::{self, ClipboardRunState};
use crate::{aura_guard, current_config, RuntimeState};

/// Clipboard poll cadence while automatic translation is enabled.
const CLIPBOARD_POLL_ENABLED: Duration = Duration::from_millis(275);

/// Idle cadence while automatic translation is disabled. The loop never reads the clipboard in this
/// state; the longer period only reduces how often it re-reads the configuration.
const CLIPBOARD_POLL_DISABLED: Duration = Duration::from_millis(1000);

/// One pasteboard observation.
///
/// `Unavailable` exists because Windows cannot report a sequence number in some states; the
/// monitor then has to fall back to reading the text and comparing it.
enum Observed {
    Sequence(u32),
    /// Only Windows can fail to report a sequence number; on macOS `changeCount` always answers.
    #[cfg(windows)]
    Unavailable,
}

/// Platform-specific pasteboard access.
pub struct ClipboardObserver;

impl ClipboardObserver {
    /// Reports the pasteboard's change sequence, if the platform can supply one.
    fn observe() -> Observed {
        #[cfg(target_os = "macos")]
        {
            use objc2_app_kit::NSPasteboard;

            let pasteboard = NSPasteboard::generalPasteboard();
            Observed::Sequence(pasteboard.changeCount() as u32)
        }

        #[cfg(windows)]
        {
            use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

            let sequence = unsafe { GetClipboardSequenceNumber() };
            if sequence == 0 {
                Observed::Unavailable
            } else {
                Observed::Sequence(sequence)
            }
        }

        #[cfg(not(any(windows, target_os = "macos")))]
        {
            Observed::Unavailable
        }
    }
}

/// Starts the clipboard monitor.
pub fn spawn(app: &AppHandle) {
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let runtime_state = app_handle.state::<RuntimeState>().inner().clone();
        let config_changed = app_handle
            .state::<std::sync::Arc<tokio::sync::Notify>>()
            .inner()
            .clone();
        let mut resume = interaction::ClipboardResumeState::default();

        loop {
            wait_for_tick(&app_handle, &config_changed).await;

            // Read the mode *before* observing the pasteboard. When automatic translation is off
            // this loop must not touch the clipboard at all, which is what makes "paused" mean
            // paused instead of merely unread.
            let config = current_config(&app_handle);

            match resume.tick(config.aura_mode_enabled) {
                ClipboardRunState::Paused => continue,
                ClipboardRunState::ResumeBaseline => {
                    // Nothing sampled the pasteboard while paused, so any stored sequence is stale.
                    // Record the current one and translate nothing: text copied during the pause
                    // must not be sent when automatic translation comes back.
                    let baseline = match ClipboardObserver::observe() {
                        Observed::Sequence(sequence) => Some(sequence),
                        #[cfg(windows)]
                        Observed::Unavailable => None,
                    };
                    let mut runtime = runtime_state.lock().await;
                    runtime.clipboard.last_sequence = baseline;
                    runtime.clipboard.last_dispatched_text = None;
                    continue;
                }
                ClipboardRunState::Running => {}
            }

            match ClipboardObserver::observe() {
                Observed::Sequence(sequence) => {
                    let mut runtime = runtime_state.lock().await;
                    if runtime.clipboard.last_sequence == Some(sequence) {
                        continue;
                    }
                    runtime.clipboard.last_sequence = Some(sequence);
                }
                #[cfg(windows)]
                Observed::Unavailable => {}
            }

            let text = app_handle.clipboard().read_text().unwrap_or_default();
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            if config.aura_guard_enabled {
                if let Some(block) = aura_guard::detect_sensitive_clipboard(&trimmed) {
                    crate::emit_aura_guard_blocked(&app_handle, block.reason);
                    continue;
                }
            }

            let mut runtime = runtime_state.lock().await;
            let suppressed =
                crate::consume_suppressed_clipboard_text(&mut runtime.clipboard, trimmed.as_str());
            let duplicate =
                runtime.clipboard.last_dispatched_text.as_deref() == Some(trimmed.as_str());

            if interaction::decide_clipboard_tick(suppressed, duplicate)
                != interaction::MonitorAction::Dispatch
            {
                continue;
            }

            runtime.clipboard.last_dispatched_text = Some(trimmed.clone());
            drop(runtime);

            let identity = interaction::RequestIdentity::from_config(&trimmed, &config);
            let app_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                match crate::reserve_request_identity(&app_clone, identity.clone()).await {
                    Ok(interaction::ReservationOutcome::Reserved) => {
                        crate::trigger_translation(app_clone, trimmed, identity).await;
                    }
                    // Another path already dispatched this exact request.
                    Ok(interaction::ReservationOutcome::AlreadyInFlight) => {}
                    Err(err) => {
                        crate::emit_daemon_error(
                            &app_clone,
                            "request-reservation-failed",
                            err,
                            true,
                        );
                    }
                }
            });
        }
    });
}

/// Sleeps until the next tick, or until the configuration changes.
///
/// A slower cadence while automatic translation is disabled keeps the idle loop cheap without ever
/// reading the clipboard; the `config_changed` notification makes a toggle feel immediate.
async fn wait_for_tick(app: &AppHandle, config_changed: &tokio::sync::Notify) {
    let period = if current_config(app).aura_mode_enabled {
        CLIPBOARD_POLL_ENABLED
    } else {
        CLIPBOARD_POLL_DISABLED
    };

    tokio::select! {
        _ = tokio::time::sleep(period) => {}
        _ = config_changed.notified() => {}
    }
}
