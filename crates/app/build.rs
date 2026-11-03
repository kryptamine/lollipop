// Declared commands get permissions, so only the windows a capability names may call them.
fn main() {
    let commands = &[
        "read_switches",
        "set_widget",
        "set_launch_at_login",
        "read_openai_key",
        "save_openai_key",
        "remove_openai_key",
        "open_openai_usage",
    ];
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(commands));
    #[allow(
        clippy::expect_used,
        reason = "a build script reports failure by panicking"
    )]
    tauri_build::try_build(attributes).expect("tauri-build failed");
}
