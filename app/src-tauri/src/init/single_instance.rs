use tauri::plugin::TauriPlugin;
use tauri::{Manager, Wry};

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(|app, _args, _cwd| {
        let _ = app
            .get_webview_window("main")
            .expect("no main window")
            .set_focus();
    })
}
