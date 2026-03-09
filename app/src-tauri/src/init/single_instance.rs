use tauri::plugin::TauriPlugin;
use tauri::{Manager, Wry};

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("popover") {
            if let Err(e) = window.show().and_then(|()| window.set_focus()) {
                tracing::warn!("Failed to show/focus popover on duplicate launch: {e}");
            }
        } else {
            tracing::warn!("Popover window not found on duplicate instance launch");
        }
    })
}
