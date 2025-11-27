use tauri::AppHandle;

#[derive(Debug, thiserror::Error)]
pub(crate) enum DeepLinkSetupError {
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    #[error("Failed to initialize deep-link plugin")]
    PluginInitializationError(tauri_plugin_deep_link::Error),
}

pub(crate) fn setup(
    #[cfg_attr(target_os = "macos", allow(unused_variables))] app: &AppHandle,
) -> Result<(), DeepLinkSetupError> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        use crate::init::deep_link::DeepLinkSetupError::PluginInitializationError;
        use tracing::info;
        info!("Registering deep link handler");
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link().register_all().map_err(PluginInitializationError)?;
    }
    Ok(())
}
