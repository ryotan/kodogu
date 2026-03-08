use suzunari_error::*;
use tauri::AppHandle;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum DeepLinkSetupError {
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    #[suzu(display("Failed to initialize deep-link plugin"))]
    PluginInitialization { source: tauri_plugin_deep_link::Error },
}

pub(crate) fn setup(
    #[cfg_attr(target_os = "macos", allow(unused_variables))] app: &AppHandle,
) -> Result<(), DeepLinkSetupError> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        use tracing::info;

        info!("Registering deep link handler");
        app.deep_link().register_all().context(PluginInitializationError)?;
    }
    Ok(())
}
