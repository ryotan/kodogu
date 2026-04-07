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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfigStore;
    use crate::config::domain::{LogicalPixel, Milliseconds};
    use redb::{Database, backends::InMemoryBackend};
    use tauri::Manager;
    use tauri::test::noop_assets;

    fn in_memory_db() -> Database {
        Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .expect("in-memory database")
    }

    fn app_with_store(store: AppConfigStore) -> tauri::App<tauri::test::MockRuntime> {
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(noop_assets()))
            .expect("mock app");
        app.manage(store);
        app
    }

    // -------------------------------------------------------------------------
    // get_config
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn get_config_returns_default_config() {
        let app = app_with_store(AppConfigStore::new(in_memory_db()).expect("store"));
        let config = get_config(app.state()).await.expect("ok");
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(300));
        assert_eq!(config.popover.layout.initial_width, LogicalPixel(480.0));
    }

    #[tokio::test]
    async fn get_config_returns_persisted_value() {
        let store = AppConfigStore::new(in_memory_db()).expect("store");
        let mut config = store.get();
        config.popover.behavior.toggle_debounce_ms = Milliseconds(999);
        store.set(config).expect("set");

        let config = get_config(app_with_store(store).state()).await.expect("ok");
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(999));
    }

    // -------------------------------------------------------------------------
    // set_config
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn set_config_persists_zero_value() {
        let app = app_with_store(AppConfigStore::new(in_memory_db()).expect("store"));
        let mut config = AppConfig::default();
        config.popover.behavior.toggle_debounce_ms = Milliseconds(0);
        set_config(app.state(), config).await.expect("ok");
        assert_eq!(
            get_config(app.state()).await.expect("ok").popover.behavior.toggle_debounce_ms,
            Milliseconds(0)
        );
    }

    #[tokio::test]
    async fn set_config_persists_custom_value() {
        let app = app_with_store(AppConfigStore::new(in_memory_db()).expect("store"));
        let mut config = AppConfig::default();
        config.popover.behavior.toggle_debounce_ms = Milliseconds(500);
        set_config(app.state(), config).await.expect("ok");
        assert_eq!(
            get_config(app.state()).await.expect("ok").popover.behavior.toggle_debounce_ms,
            Milliseconds(500)
        );
    }
}
