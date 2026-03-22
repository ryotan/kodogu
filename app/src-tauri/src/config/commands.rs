use serde::Serialize;
use suzunari_error::StackReport;
use tauri::State;

use super::domain::AppConfig;
use super::store::{AppConfigStore, ConfigError};

// =============================================================================
// Command error type
// =============================================================================

/// Serializable error type returned from Tauri commands.
///
/// Serializes as a plain JSON string so callers can display the message directly.
/// Using a newtype instead of `String` makes the intent explicit at the call site.
///
/// TODO(KDG-27): Consider structured error format `{kind, message}` for typed JS-side handling.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct CommandError(String);

impl From<ConfigError> for CommandError {
    fn from(e: ConfigError) -> Self {
        Self(StackReport::from(e).to_string())
    }
}

// =============================================================================
// Tauri commands
// =============================================================================

/// Returns the current application configuration.
///
/// Async so Tauri runs this on the tokio thread pool instead of the main thread,
/// keeping the UI responsive even if `RwLock::read()` briefly contends.
#[tauri::command]
pub(crate) async fn get_config(
    state: State<'_, AppConfigStore>,
) -> Result<AppConfig, CommandError> {
    Ok(state.get())
}

/// Persists the given configuration and updates the in-memory cache.
///
/// Async so Tauri runs this on the tokio thread pool, keeping the main thread
/// free even though `begin_write()` may block synchronously.
#[tauri::command]
pub(crate) async fn set_config(
    state: State<'_, AppConfigStore>,
    config: AppConfig,
) -> Result<(), CommandError> {
    state.set(config).map_err(CommandError::from)
}
