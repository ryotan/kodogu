use tauri::plugin::TauriPlugin;
use tauri::{Manager, Wry};

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.set_focus() {
                tracing::warn!("Failed to focus main window on duplicate launch: {e}");
            }
        } else {
            tracing::warn!("Main window not found on duplicate instance launch");
        }
    })
}
