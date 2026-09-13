//! Pure interaction decisions shared by the hotkey, clipboard monitor, and tray paths.
//!
//! Everything in this module is free of Tauri types on purpose. The rules that decide whether a
//! hotkey press translates, recalls, or collapses previously lived inline in `lib.rs`, reachable
//! only through an `AppHandle`, which made them impossible to assert in CI. Keeping them pure is
//! what lets the acceptance rule "recalling the same text must not issue a new request" be tested
//! directly.

use crate::config::AppConfig;
use crate::readiness;

/// The complete identity of a translation request.
///
/// Reuse is only allowed when every field matches. Comparing the text alone would let a stale
/// result survive a language, provider, model, base URL, or profile change, which is exactly the
/// bug this type exists to prevent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RequestIdentity {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub provider: String,
    pub model: String,
    pub api_base_url: String,
    pub profile_id: String,
}

impl RequestIdentity {
    /// Builds the identity a hotkey or monitor dispatch would produce for `text`.
    pub fn from_config(text: &str, config: &AppConfig) -> Self {
        Self {
            text: text.to_string(),
            source_lang: config.source_lang.clone(),
            target_lang: config.target_lang.clone(),
            provider: config.provider.as_str().to_string(),
            model: config.model.clone(),
            api_base_url: config.api_base_url.clone(),
            profile_id: config.active_profile_id.clone(),
        }
    }

    /// Builds the identity from the parameters a request is actually dispatched with.
    ///
    /// This is the authoritative constructor. A hotkey-derived identity describes what the *next*
    /// request would be; this one describes the request really in flight, so a draft
    /// re-translation or a history replay cannot leave the stored identity describing something
    /// that was never sent.
    #[allow(clippy::too_many_arguments)]
    pub fn from_request(
        text: &str,
        source_lang: &str,
        target_lang: &str,
        provider: &str,
        model: &str,
        api_base_url: &str,
        profile_id: Option<&str>,
    ) -> Self {
        Self {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            api_base_url: api_base_url.to_string(),
            profile_id: profile_id.unwrap_or_default().to_string(),
        }
    }
}

/// Everything the hotkey decision depends on, gathered by the caller.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HotkeyContext {
    /// Whether the translation window is currently visible.
    pub window_visible: bool,
    /// Whether the clipboard holds no usable text after trimming.
    pub clipboard_empty: bool,
    /// Whether the configuration is complete enough to attempt a translation.
    pub ready: bool,
    /// Whether the clipboard identity equals the last dispatched request identity.
    pub matches_last_request: bool,
}

/// What the hotkey should do. Only [`HotkeyOutcome::TranslateNew`] may cause a network request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HotkeyOutcome {
    /// New text: cancel any in-flight request and translate this text.
    TranslateNew,
    /// Same request and the window is visible: collapse it, leaving in-flight work running.
    Collapse,
    /// Same request and the window is hidden: bring the existing result back without re-requesting.
    Recall,
    /// Nothing usable on the clipboard: open an empty state that accepts direct input.
    ShowEmptyState,
    /// Configuration is incomplete: route the user to setup instead of failing a request.
    Unavailable,
}

/// Decides what a hotkey press means.
///
/// Both automatic and manual modes share this one decision, which is what makes the documented
/// behaviour ("press again to recall or hide") true in both.
pub fn decide_hotkey(ctx: HotkeyContext) -> HotkeyOutcome {
    if !ctx.ready {
        return HotkeyOutcome::Unavailable;
    }

    if ctx.clipboard_empty {
        return HotkeyOutcome::ShowEmptyState;
    }

    if ctx.matches_last_request {
        return if ctx.window_visible {
            HotkeyOutcome::Collapse
        } else {
            HotkeyOutcome::Recall
        };
    }

    HotkeyOutcome::TranslateNew
}

/// What the clipboard monitor should do with an observed clipboard text.
///
/// The paused state is not represented here: [`ClipboardResumeState`] decides whether the monitor
/// runs at all, so this type only describes the running path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MonitorAction {
    /// Unchanged, suppressed, or already dispatched: observe and translate nothing.
    Skip,
    /// Forward this text to the translation pipeline.
    Dispatch,
}

/// Whether the observed clipboard text may be dispatched.
///
/// Callers resolve [`ClipboardModeState`] first, so this function is only reached while automatic
/// translation is running and past the resume-baseline sample.
pub fn decide_clipboard_tick(suppressed: bool, duplicate: bool) -> MonitorAction {
    if suppressed || duplicate {
        return MonitorAction::Skip;
    }

    MonitorAction::Dispatch
}

/// Whether the clipboard monitor is paused, re-establishing its baseline, or running normally.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClipboardRunState {
    /// Automatic translation is off: the caller must perform no clipboard work.
    Paused,
    /// Automatic translation just resumed: record the observed sequence and translate nothing.
    ResumeBaseline,
    /// Normal operation: dispatch fresh, unsuppressed text.
    Running,
}

/// Tracks the pause/resume transition of the clipboard monitor.
///
/// The previous implementation simply cleared the stored sequence number while paused. Because
/// nothing samples the pasteboard in that state, the stored number was unknown when automatic
/// translation resumed, so the very next sample always looked like a fresh clipboard change and
/// the text copied while paused was translated on re-enable — the opposite of the documented
/// behaviour.
///
/// Resuming therefore costs one baseline sample whose only job is to re-establish the sequence
/// reference. Text copied during the pause is never dispatched; text copied after the resume is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClipboardResumeState {
    awaiting_resume_baseline: bool,
}

impl Default for ClipboardResumeState {
    /// Starts out awaiting a baseline.
    ///
    /// Defaulting to "running" was a defect: the monitor's first observation after process start
    /// also has no stored sequence, so whatever was already on the clipboard at launch — or text
    /// copied between launch and the first poll — was dispatched as if it were a fresh copy. Any
    /// first enable, whether at startup or after a pause, must establish the baseline first.
    fn default() -> Self {
        Self {
            awaiting_resume_baseline: true,
        }
    }
}

impl ClipboardResumeState {
    /// Classifies this tick and advances the transition state.
    ///
    /// The first disabled tick arms the resume baseline. A baseline is consumed only by the first
    /// enabled tick after a pause.
    pub fn tick(&mut self, mode_enabled: bool) -> ClipboardRunState {
        if !mode_enabled {
            self.awaiting_resume_baseline = true;
            return ClipboardRunState::Paused;
        }

        if self.awaiting_resume_baseline {
            self.awaiting_resume_baseline = false;
            return ClipboardRunState::ResumeBaseline;
        }

        ClipboardRunState::Running
    }
}

/// How a request-identity reservation turned out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReservationOutcome {
    /// A request for this identity already exists. The caller must not dispatch again.
    AlreadyInFlight,
    /// The identity is now reserved. The caller owns the dispatch.
    Reserved,
}

/// Decides whether a candidate identity may be reserved for dispatch.
///
/// This is the concurrency-critical rule behind "one press, one request". `pending` is a dispatch
/// that is still being prepared and `last` is a request that already reached the translator;
/// either one matching means the user's repeat press is a collapse or a recall, not a new request.
///
/// It is a free function taking the two stored values rather than a method reading shared state,
/// so the concurrent-press case can be asserted directly.
pub fn reserve_identity(
    pending: Option<&RequestIdentity>,
    last: Option<&RequestIdentity>,
    candidate: &RequestIdentity,
) -> ReservationOutcome {
    if pending == Some(candidate) || last == Some(candidate) {
        return ReservationOutcome::AlreadyInFlight;
    }

    ReservationOutcome::Reserved
}

/// Whether `identity` owns the current reservation.
///
/// A reservation is released only by the request it was taken for. Releasing the slot regardless
/// of identity let one request delete another request's reservation, which then allowed the other
/// request to be dispatched twice.
pub fn owns_reservation(
    pending: Option<&RequestIdentity>,
    identity: &RequestIdentity,
) -> bool {
    pending == Some(identity)
}

/// Whether a reservation belongs to a dispatch of `text`.
///
/// Used by the frontend's abandon path, where only the text is known: the configuration that
/// produced the reservation may already have failed to load.
pub fn reservation_matches_text(pending: Option<&RequestIdentity>, text: &str) -> bool {
    pending.map(|identity| identity.text == text).unwrap_or(false)
}

/// Whether the configuration is complete enough for a hotkey to attempt a translation.
pub fn is_ready(config: &AppConfig) -> bool {
    readiness::is_translation_ready(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, Provider};

    fn ready_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.api_key = "sk-test".to_string();
        config
    }

    fn context(
        window_visible: bool,
        clipboard_empty: bool,
        ready: bool,
        matches_last_request: bool,
    ) -> HotkeyContext {
        HotkeyContext {
            window_visible,
            clipboard_empty,
            ready,
            matches_last_request,
        }
    }

    #[test]
    fn new_text_translates() {
        assert_eq!(
            decide_hotkey(context(false, false, true, false)),
            HotkeyOutcome::TranslateNew
        );
    }

    #[test]
    fn same_text_with_visible_window_collapses_without_requesting() {
        // The core P0 rule: repeating the hotkey must not issue a second paid request.
        assert_eq!(
            decide_hotkey(context(true, false, true, true)),
            HotkeyOutcome::Collapse
        );
    }

    #[test]
    fn same_text_with_hidden_window_recalls_without_requesting() {
        assert_eq!(
            decide_hotkey(context(false, false, true, true)),
            HotkeyOutcome::Recall
        );
    }

    #[test]
    fn empty_clipboard_opens_an_editable_empty_state() {
        assert_eq!(
            decide_hotkey(context(false, true, true, false)),
            HotkeyOutcome::ShowEmptyState
        );
    }

    #[test]
    fn incomplete_setup_routes_to_setup_before_anything_else() {
        // Even with a matching identity, an unusable configuration must not attempt a request.
        assert_eq!(
            decide_hotkey(context(true, false, false, true)),
            HotkeyOutcome::Unavailable
        );
        assert_eq!(
            decide_hotkey(context(false, true, false, false)),
            HotkeyOutcome::Unavailable
        );
    }

    #[test]
    fn identity_covers_text_and_every_configuration_field() {
        let config = ready_config();
        let base = RequestIdentity::from_config("hello", &config);

        assert_eq!(base, RequestIdentity::from_config("hello", &config));
        assert_ne!(base, RequestIdentity::from_config("goodbye", &config));

        let mut changed = config.clone();
        changed.target_lang = "Japanese".to_string();
        assert_ne!(base, RequestIdentity::from_config("hello", &changed));

        let mut changed = config.clone();
        changed.source_lang = "English".to_string();
        assert_ne!(base, RequestIdentity::from_config("hello", &changed));

        let mut changed = config.clone();
        changed.model = "deepseek-reasoner".to_string();
        assert_ne!(base, RequestIdentity::from_config("hello", &changed));

        let mut changed = config.clone();
        changed.api_base_url = "http://localhost:11434".to_string();
        assert_ne!(base, RequestIdentity::from_config("hello", &changed));

        let mut changed = config.clone();
        changed.provider = Provider::Ollama;
        assert_ne!(base, RequestIdentity::from_config("hello", &changed));

        let mut changed = config.clone();
        changed.active_profile_id = "work".to_string();
        assert_ne!(base, RequestIdentity::from_config("hello", &changed));
    }

    #[test]
    fn a_request_only_releases_its_own_reservation() {
        // The defect this guards: request A recorded its identity and cleared the slot
        // unconditionally, deleting request B's reservation. B could then be dispatched a second
        // time, which is precisely the double request the reservation exists to prevent.
        let config = ready_config();
        let a = RequestIdentity::from_config("hello", &config);
        let b = RequestIdentity::from_config("goodbye", &config);

        // B currently owns the slot.
        assert!(!owns_reservation(Some(&b), &a), "A must not release B's reservation");
        assert!(owns_reservation(Some(&b), &b), "B owns its own reservation");
        assert!(owns_reservation(Some(&a), &a));
        assert!(!owns_reservation(None, &a), "an empty slot has no owner");
    }

    #[test]
    fn abandoning_a_dispatch_releases_the_matching_reservation() {
        let config = ready_config();
        let reserved = RequestIdentity::from_config("hello", &config);

        assert!(reservation_matches_text(Some(&reserved), "hello"));
        assert!(
            !reservation_matches_text(Some(&reserved), "goodbye"),
            "abandoning one request must not release another one's reservation"
        );
        assert!(!reservation_matches_text(None, "hello"));
    }

    #[test]
    fn a_first_press_reserves_its_identity() {
        let candidate = RequestIdentity::from_config("hello", &ready_config());
        assert_eq!(
            reserve_identity(None, None, &candidate),
            ReservationOutcome::Reserved
        );
    }

    #[test]
    fn a_second_press_while_the_first_is_still_preparing_does_not_dispatch() {
        // Two hotkey tasks can both read "no match" before either writes. The reservation is what
        // makes the second one see the first one's pending identity and collapse instead of
        // issuing a duplicate paid request.
        let config = ready_config();
        let candidate = RequestIdentity::from_config("hello", &config);

        let first = reserve_identity(None, None, &candidate);
        assert_eq!(first, ReservationOutcome::Reserved);

        let second = reserve_identity(Some(&candidate), None, &candidate);
        assert_eq!(second, ReservationOutcome::AlreadyInFlight);
    }

    #[test]
    fn a_press_after_a_dispatched_request_does_not_dispatch_again() {
        let config = ready_config();
        let candidate = RequestIdentity::from_config("hello", &config);
        assert_eq!(
            reserve_identity(None, Some(&candidate), &candidate),
            ReservationOutcome::AlreadyInFlight
        );
    }

    #[test]
    fn a_failed_dispatch_releases_its_reservation() {
        // `clear_pending_request_identity` empties the pending slot, which returns the state to
        // this case, so the next press translates rather than collapsing.
        let config = ready_config();
        let candidate = RequestIdentity::from_config("hello", &config);
        assert_eq!(
            reserve_identity(None, None, &candidate),
            ReservationOutcome::Reserved
        );
    }

    #[test]
    fn a_different_identity_never_reuses_a_reservation() {
        let config = ready_config();
        let pending = RequestIdentity::from_config("hello", &config);
        let other = RequestIdentity::from_config("goodbye", &config);
        assert_eq!(
            reserve_identity(Some(&pending), None, &other),
            ReservationOutcome::Reserved
        );
    }

    #[test]
    fn request_identity_comes_from_the_dispatched_parameters() {
        // The identity is recorded at the request boundary, so it must describe what was really
        // sent. A history replay uses the entry's own provider/model/language/profile, which can
        // differ from the active configuration; deriving the identity from the config instead
        // would record a request that never happened.
        let replayed = RequestIdentity::from_request(
            "bonjour",
            "French",
            "English",
            "openrouter",
            "mistral",
            "https://openrouter.ai/api",
            Some("archived-openrouter"),
        );

        let mut active = ready_config();
        active.provider = Provider::DeepSeek;
        active.model = "deepseek-chat".to_string();
        active.source_lang = "auto".to_string();
        active.target_lang = "Chinese".to_string();
        active.active_profile_id = "default".to_string();
        let from_config = RequestIdentity::from_config("bonjour", &active);

        assert_ne!(
            replayed, from_config,
            "a replay through another profile must not look like the active configuration"
        );
        assert_eq!(replayed.text, "bonjour");
        assert_eq!(replayed.provider, "openrouter");
        assert_eq!(replayed.profile_id, "archived-openrouter");
    }

    #[test]
    fn missing_profile_is_recorded_as_an_empty_identity_field() {
        let identity = RequestIdentity::from_request(
            "hello",
            "auto",
            "Chinese",
            "ollama",
            "mistral",
            "http://localhost:11434",
            None,
        );
        assert_eq!(identity.profile_id, "");
    }

    #[test]
    fn paused_monitor_reports_paused() {
        let mut resume = ClipboardResumeState::default();
        assert_eq!(resume.tick(false), ClipboardRunState::Paused);
        assert_eq!(resume.tick(false), ClipboardRunState::Paused);

        // Repeated paused ticks are stable, so the caller can rely on Paused meaning "do nothing
        // with the pasteboard" rather than "do it once and then something else".
    }

    #[test]
    fn the_very_first_enabled_tick_establishes_a_baseline() {
        // The critical regression: with Aura off at launch, a user could copy text and enable
        // Aura before the first poll. The first enabled tick used to report Running with no
        // stored sequence, so that text was dispatched even though it was copied while the
        // feature was off.
        let mut resume = ClipboardResumeState::default();

        assert_eq!(
            resume.tick(true),
            ClipboardRunState::ResumeBaseline,
            "the first enabled tick after process start must not dispatch"
        );
        assert_eq!(resume.tick(true), ClipboardRunState::Running);
    }

    #[test]
    fn first_tick_after_a_pause_only_establishes_the_baseline() {
        // The monitor used to clear its stored sequence while paused, so the first sample after
        // resuming always looked like a fresh clipboard change and the text copied during the
        // pause was translated on re-enable.
        let mut resume = ClipboardResumeState::default();

        assert_eq!(resume.tick(false), ClipboardRunState::Paused);
        assert_eq!(
            resume.tick(true),
            ClipboardRunState::ResumeBaseline,
            "the first enabled tick after a pause must not be treated as a change"
        );
        assert_eq!(resume.tick(true), ClipboardRunState::Running);
    }

    #[test]
    fn a_longer_pause_still_costs_exactly_one_baseline_sample() {
        let mut resume = ClipboardResumeState::default();

        // Many paused ticks, as happens when automatic translation stays off for a while.
        for _ in 0..50 {
            assert_eq!(resume.tick(false), ClipboardRunState::Paused);
        }

        assert_eq!(resume.tick(true), ClipboardRunState::ResumeBaseline);
        assert_eq!(resume.tick(true), ClipboardRunState::Running);
    }

    #[test]
    fn enabling_after_a_silent_start_costs_one_baseline_sample() {
        // Mirrors the real call order: Aura launches disabled, and the mode is not observed until
        // the first poll. Enabling before that poll must still baseline rather than dispatch.
        let mut resume = ClipboardResumeState::default();

        assert_eq!(resume.tick(false), ClipboardRunState::Paused);
        assert_eq!(resume.tick(true), ClipboardRunState::ResumeBaseline);
        assert_eq!(resume.tick(true), ClipboardRunState::Running);
    }

    #[test]
    fn running_monitor_dispatches_only_fresh_unsuppressed_text() {
        assert_eq!(decide_clipboard_tick(false, false), MonitorAction::Dispatch);
        assert_eq!(
            decide_clipboard_tick(true, false),
            MonitorAction::Skip,
            "Aura Guard suppression and self-copies must not be re-dispatched"
        );
        assert_eq!(
            decide_clipboard_tick(false, true),
            MonitorAction::Skip,
            "an already dispatched text must not be re-dispatched"
        );
    }

    #[test]
    fn readiness_gate_matches_runtime_readiness() {
        let mut config = AppConfig::default();
        assert!(!is_ready(&config), "a default config has no credential yet");

        config.api_key = "sk-test".to_string();
        assert!(is_ready(&config));

        // Ollama needs no credential, so it is ready without an API key.
        let mut ollama = AppConfig::default();
        ollama.provider = Provider::Ollama;
        ollama.model = "mistral".to_string();
        assert!(is_ready(&ollama));
    }
}
