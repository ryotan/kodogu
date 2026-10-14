use suzunari_error::*;
use tauri::{AppHandle, Runtime};

#[suzunari_error]
#[suzu(display("Failed to initialize deep-link plugin"))]
pub struct DeepLinkSetupError {
    source: tauri_plugin_deep_link::Error,
}

pub(crate) fn setup<R: Runtime>(
    #[cfg_attr(target_os = "macos", allow(unused_variables))] app: &AppHandle<R>,
) -> Result<(), DeepLinkSetupError> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        use tracing::info;

        info!("Registering deep link handler");
        app.deep_link().register_all().context(DeepLinkSetupSnafu)?;
    }
    Ok(())
}
