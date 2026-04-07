use crate::init::popover::resize_popover;
use suzunari_error::*;
use tauri::Runtime;
use time::UtcOffset;

mod config;
mod init;
mod store;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum RunError {
    #[suzu(display("Tauri application failed to start"))]
    TauriRun { source: tauri::Error },
    #[suzu(display("Failed to setup logger"))]
    LoggerSetup { source: init::logger::LoggerSetupError },
    #[suzu(display("Failed to setup deep link"))]
    DeepLinkSetup { source: init::deep_link::DeepLinkSetupError },
    #[suzu(display("Failed to setup datastore"))]
    StoreSetup { source: init::store::StoreSetupError },
    #[suzu(display("Failed to setup tray icon"))]
    TraySetup { source: init::tray::TraySetupError },
    #[suzu(display("Failed to setup menu item"))]
    MenuSetup { source: init::menu::MenuSetupError },
}

fn setup_once<R: Runtime>(app: &mut tauri::App<R>, zone_offset: UtcOffset) -> Result<(), RunError> {
    init::logger::setup(app.handle(), zone_offset).context(LoggerSetupError)?;
    init::deep_link::setup(app.handle()).context(DeepLinkSetupError)?;

    #[cfg(desktop)]
    {
        #[cfg(target_os = "macos")]
        if let Err(e) = app.handle().set_activation_policy(tauri::ActivationPolicy::Accessory) {
            tracing::warn!("Failed to set activation policy to Accessory: {e}");
        }
        init::store::setup(app.handle()).context(StoreSetupError)?;
        init::popover::setup(app.handle());
        init::tray::setup(app.handle()).context(TraySetupError)?;
        init::menu::setup(app).context(MenuSetupError)?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), RunError> {
    let zone_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().with_denylist(&["popover"]).build())
        .setup(move |app| {
            setup_once(app, zone_offset)
                .map_err(StackReport::from)
                .map_err(DisplayError::new)
                .map_err(Into::into)
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
            config::commands::get_config,
            config::commands::set_config,
        ]);
    }

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .context(TauriRunError)?;

    Ok(())
}
