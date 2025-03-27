use tauri::{AppHandle, Error, Manager};

pub(crate) fn setup(app: &AppHandle) -> Result<(), Error> {
    let salt_path = app
        .path()
        .app_local_data_dir()
        .expect("Could not resolve app local data path")
        .join("stronghold-salt.txt");
    app.plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())?;
    Ok(())
}
