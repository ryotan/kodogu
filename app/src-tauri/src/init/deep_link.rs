use tauri::AppHandle;
use tauri_plugin_deep_link::Error;

pub(crate) fn setup(app: &AppHandle) -> Result<(), Error> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link().register_all()?;
    }
    Ok(())
}
