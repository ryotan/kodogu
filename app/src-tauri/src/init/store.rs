use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

use redb::Database;
use suzunari_error::*;
use tauri::{Manager, Runtime};

use crate::config::{AppConfigStore, ConfigError};

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum StoreInitError {
    #[suzu(display("Failed to resolve app local data directory"))]
    AppLocalDataDir { source: tauri::Error },
    #[suzu(display("Failed to create data directory {path:?}"))]
    CreateDir { source: std::io::Error, path: PathBuf },
    #[suzu(display("Failed to create database at {path:?}"))]
    CreateDatabase { source: redb::DatabaseError, path: PathBuf },
    #[suzu(display("Failed to open database at {path:?}"))]
    OpenDatabase { source: redb::DatabaseError, path: PathBuf },
    #[suzu(display("Failed to load config from database"))]
    Configuration { source: ConfigError },
}

pub(crate) fn setup<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), StoreInitError> {
    let data_dir = app.path().app_local_data_dir().context(AppLocalDataDirError {})?;
    create_dir_all(&data_dir).context(CreateDirError { path: data_dir.clone() })?;

    let db_path = data_dir.join("kodogu.redb");
    let db = match db_path.exists() {
        false => Arc::new(
            Database::create(&db_path).context(CreateDatabaseError { path: db_path.clone() })?,
        ),
        true => {
            Arc::new(Database::open(&db_path).context(OpenDatabaseError { path: db_path.clone() })?)
        }
    };

    app.manage(AppConfigStore::new(db).context(ConfigurationError {})?);

    Ok(())
}
