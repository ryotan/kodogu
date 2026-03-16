use crate::init::popover::resize_popover;
use suzunari_error::*;

mod config;
mod init;
mod store;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum RunError {
    #[suzu(display("Tauri application failed to start"))]
    TauriRun { source: tauri::Error },
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), RunError> {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().with_denylist(&["popover"]).build())
        .setup(|app| {
            init::logger::setup(app.handle())
                .map_err(StackReport::from)
                .map_err(DisplayError::new)?;
            init::deep_link::setup(app.handle())
                .map_err(StackReport::from)
                .map_err(DisplayError::new)?;

            #[cfg(desktop)]
            {
                #[cfg(target_os = "macos")]
                if let Err(e) =
                    app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory)
                {
                    tracing::warn!("Failed to set activation policy to Accessory: {e}");
                }
                init::store::setup(app.handle())
                    .map_err(StackReport::from)
                    .map_err(DisplayError::new)?;
                init::popover::setup(app.handle());
                init::tray::setup(app.handle())
                    .map_err(StackReport::from)
                    .map_err(DisplayError::new)?;
                init::menu::setup(app).map_err(StackReport::from).map_err(DisplayError::new)?;
            }
            Ok(())
        });

    #[cfg(desktop)]
    {
        builder = builder.plugin(init::single_instance::plugin());
    }

    // Desktop-only commands depend on state registered in init::store::setup and init::popover::setup,
    // which only run under #[cfg(desktop)]. Register them conditionally to keep the invariant.
    #[cfg(desktop)]
    {
        builder = builder.invoke_handler(tauri::generate_handler![
            resize_popover,
            config::get_config,
            config::set_config,
        ]);
    }

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .context(TauriRunError {})?;

    Ok(())
}
