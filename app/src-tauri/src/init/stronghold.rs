use suzunari_error::*;
use tauri::{AppHandle, Manager};
use tracing::info;

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum StrongholdSetupError {
    #[suzu(display("Local data directory was not found"))]
    LocalDataDirNotFound { source: tauri::Error },
    #[suzu(display("Failed to initialize stronghold plugin"))]
    PluginInitialization { source: tauri::Error },
}

pub(crate) fn setup(app: &AppHandle) -> Result<(), StrongholdSetupError> {
    info!("Initializing stronghold plugin");
    let salt_path = app
        .path()
        .app_local_data_dir()
        .context(LocalDataDirNotFoundError)?
        .join("stronghold-salt.txt");
    app.plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())
        .context(PluginInitializationError)?;
    Ok(())
}
