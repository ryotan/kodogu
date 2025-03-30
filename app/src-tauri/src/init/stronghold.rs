use crate::init::stronghold::StrongholdSetupError::{
    LocalDataDirNotFoundError, PluginInitializationError,
};
use tauri::{AppHandle, Manager};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub(crate) enum StrongholdSetupError {
    #[error("Local data directory was not found")]
    LocalDataDirNotFoundError(tauri::Error),
    #[error("Failed to initialize stronghold plugin")]
    PluginInitializationError(tauri::Error),
}

pub(crate) fn setup(app: &AppHandle) -> Result<(), StrongholdSetupError> {
    info!("Initializing stronghold plugin");
    let salt_path = app
        .path()
        .app_local_data_dir()
        .map_err(LocalDataDirNotFoundError)?
        .join("stronghold-salt.txt");
    app.plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())
        .map_err(PluginInitializationError)?;
    Ok(())
}
