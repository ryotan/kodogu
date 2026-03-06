use crate::init::popover::resize_popover;

mod init;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().with_denylist(&["popover"]).build())
        .setup(|app| {
            init::logger::setup(app.handle())?;
            init::deep_link::setup(app.handle())?;
            #[cfg(desktop)]
            {
                if let Err(e) =
                    app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory)
                {
                    tracing::warn!("Failed to set activation policy to Accessory: {e}");
                }
                init::popover::setup(app.handle());
                init::tray::setup(app.handle())?;
                init::menu::setup(app)?;
            }
            Ok(())
        });

    #[cfg(desktop)]
    {
        builder = builder.plugin(init::single_instance::plugin());
    }

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, resize_popover])
        .run(tauri::generate_context!())
        .expect("Tauri application failed to start or encountered a fatal runtime error");
}
