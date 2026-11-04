use tauri::{ActivationPolicy, App, AppHandle, Builder, Wry};

/// Without this, launching takes focus from League: see ADR 0006.
pub(crate) fn keep_focus_off(builder: Builder<Wry>) -> Builder<Wry> {
    builder.activate_ignoring_other_apps(false)
}

/// No Dock icon and no menu bar of its own.
pub(crate) fn run_as_accessory(app: &mut App) {
    app.set_activation_policy(ActivationPolicy::Accessory);
}

/// Hiding the app is what gives focus back to the app a window was opened over. The
/// overlay window opts out of hiding, so it stays.
pub(crate) fn hide_app(app: &AppHandle) {
    let _ = app.hide();
}
