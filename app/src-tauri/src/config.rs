use std::sync::RwLock;

use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use suzunari_error::*;
use tauri::State;

use crate::store::MessagePack;

const CONFIG_TABLE: TableDefinition<&str, MessagePack<StoredAppConfig>> =
    TableDefinition::new("config");
const CONFIG_KEY: &str = "app_config";

// =============================================================================
// Value types
// =============================================================================

macro_rules! newtype_string {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) String);

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

macro_rules! newtype_u64 {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) u64);
    };
}

macro_rules! newtype_f64 {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) f64);
    };
}

newtype_u64! {
    /// Duration in milliseconds.
    Milliseconds
}

newtype_f64! {
    /// Device-independent logical pixels.
    LogicalPixel
}

newtype_string! {
    /// A CSS `<color>` value (e.g. `"rgba(28, 28, 30, 0.92)"`).
    CssColor
}

newtype_string! {
    /// A CSS `<filter-function-list>` value (e.g. `"blur(24px) saturate(180%)"`).
    CssFilter
}

newtype_string! {
    /// A CSS `font-family` value (e.g. `"-apple-system, BlinkMacSystemFont, sans-serif"`).
    CssFontFamily
}

newtype_string! {
    /// A single CSS `<length>` value (e.g. `"40px"`).
    CssLength
}

newtype_string! {
    /// A CSS `padding` shorthand value (e.g. `"6px 0"`).
    CssPadding
}

// =============================================================================
// Config types
// =============================================================================

/// Timing and interaction settings for the popover.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverBehaviorConfig {
    /// Debounce interval for tray icon clicks.
    pub(crate) toggle_debounce_ms: Milliseconds,
}

impl Default for PopoverBehaviorConfig {
    fn default() -> Self {
        Self { toggle_debounce_ms: Milliseconds(300) }
    }
}

/// Geometry and sizing settings for the popover.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverLayoutConfig {
    /// Initial popover window width.
    pub(crate) initial_width: LogicalPixel,
    /// Minimum gap between the popover bottom and the screen bottom edge (macOS only).
    pub(crate) bottom_margin: LogicalPixel,
    /// Gap between the tray icon bottom edge and the popover top edge (macOS only).
    pub(crate) tray_gap: LogicalPixel,
    /// Fallback max height used when monitor info is unavailable.
    pub(crate) fallback_max_height: LogicalPixel,
    /// Lower bound for the computed max height, ensuring a minimum usable height.
    pub(crate) min_max_height: LogicalPixel,
    /// Height of the header bar.
    pub(crate) header_height: CssLength,
    /// Padding for the card list container.
    pub(crate) card_list_padding: CssPadding,
}

impl Default for PopoverLayoutConfig {
    fn default() -> Self {
        Self {
            initial_width: LogicalPixel(480.0),
            bottom_margin: LogicalPixel(8.0),
            tray_gap: LogicalPixel(4.0),
            fallback_max_height: LogicalPixel(600.0),
            min_max_height: LogicalPixel(200.0),
            header_height: CssLength::from("40px"),
            card_list_padding: CssPadding::from("6px 0"),
        }
    }
}

/// Visual design tokens for the popover shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverAppearanceConfig {
    /// Background color of the popover window.
    pub(crate) surface_color: CssColor,
    /// Border color used for the header bottom separator.
    pub(crate) border_color: CssColor,
    /// Backdrop filter applied to the popover for a frosted glass effect.
    pub(crate) backdrop_filter: CssFilter,
    /// Font family for the entire popover.
    pub(crate) font_family: CssFontFamily,
}

impl Default for PopoverAppearanceConfig {
    fn default() -> Self {
        Self {
            surface_color: CssColor::from("rgba(28, 28, 30, 0.92)"),
            border_color: CssColor::from("rgba(255, 255, 255, 0.08)"),
            backdrop_filter: CssFilter::from("blur(24px) saturate(180%)"),
            font_family: CssFontFamily::from(
                "-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif",
            ),
        }
    }
}

/// All settings for the popover window.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverConfig {
    pub(crate) behavior: PopoverBehaviorConfig,
    pub(crate) layout: PopoverLayoutConfig,
    pub(crate) appearance: PopoverAppearanceConfig,
}

/// Application configuration persisted in redb and exposed to JavaScript via Tauri commands.
///
/// All values are serialized with camelCase field names (for both storage and JS interop).
/// Add new fields at any time — missing fields in stored data are filled from `Default::default()`.
///
/// ## Schema evolution
/// See ADR-0004. `#[serde(default)]` on each struct enables forward-compatible field additions.
/// Renaming or changing field types requires bumping `StoredAppConfig` to a new variant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AppConfig {
    pub(crate) popover: PopoverConfig,
}

// =============================================================================
// Versioned storage envelope
// =============================================================================

/// Versioned storage envelope for `AppConfig`.
///
/// Wrapping `AppConfig` in an enum from the first version ensures a version discriminant
/// is always present in stored bytes, enabling future-breaking schema changes.
///
/// When a breaking change is needed:
///   1. Rename `AppConfig` to `AppConfigV1`
///   2. Define the new `AppConfig`
///   3. Add the `V2(AppConfig)` variant
///   4. Implement migration in `AppConfigStore::load_from_db`
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version", content = "data", rename_all = "camelCase")]
enum StoredAppConfig {
    V1(AppConfig),
}

// =============================================================================
// Error types
// =============================================================================

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum ConfigError {
    #[suzu(display("Failed to begin database transaction"))]
    Transaction { source: redb::TransactionError },
    #[suzu(display("Failed to access config table"))]
    Table { source: redb::TableError },
    #[suzu(display("Failed to read from config storage"))]
    Storage { source: redb::StorageError },
    #[suzu(display("Failed to commit config changes"))]
    Commit { source: redb::CommitError },
}

// =============================================================================
// Store
// =============================================================================

/// Manages a shared redb database handle and a cached in-memory copy of `AppConfig`.
pub(crate) struct AppConfigStore {
    db: Database,
    cached: RwLock<AppConfig>,
}

impl AppConfigStore {
    /// Creates a new store from a database handle and loads the stored config.
    /// On first launch when no config exists, writes and caches `AppConfig::default()`.
    pub(crate) fn new(db: Database) -> Result<Self, ConfigError> {
        let store = Self { db, cached: RwLock::new(AppConfig::default()) };
        let config = store.load_from_db()?;
        // Poison recovery: if a panicking thread held the lock, we still want the latest value.
        // AppConfig is independently validated, so a poisoned cache is safe to overwrite.
        *store.cached.write().unwrap_or_else(|e| e.into_inner()) = config;
        Ok(store)
    }

    fn load_from_db(&self) -> Result<AppConfig, ConfigError> {
        let read_txn = self.db.begin_read().context(TransactionError {})?;

        let table = match read_txn.open_table(CONFIG_TABLE) {
            Ok(t) => t,
            // First launch: table does not exist yet.
            // Release the read transaction before starting a write — holding both is
            // wasteful and prevents freed-page reclamation until the reader drops.
            Err(redb::TableError::TableDoesNotExist(_)) => {
                drop(read_txn);
                let default = AppConfig::default();
                self.write_to_db(&default)?;
                return Ok(default);
            }
            Err(e) => Err(e).context(TableError {})?,
        };

        let Some(guard) = table.get(CONFIG_KEY).context(StorageError {})? else {
            // Same rationale as above — release read resources before writing.
            drop(table);
            drop(read_txn);
            let default = AppConfig::default();
            self.write_to_db(&default)?;
            return Ok(default);
        };

        Ok(match guard.value() {
            StoredAppConfig::V1(config) => config,
        })
    }

    fn write_to_db(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let write_txn = self.db.begin_write().context(TransactionError {})?;
        {
            let mut table = write_txn.open_table(CONFIG_TABLE).context(TableError {})?;
            table
                .insert(CONFIG_KEY, &StoredAppConfig::V1(config.clone()))
                .context(StorageError {})?;
        }
        write_txn.commit().context(CommitError {})
    }

    pub(crate) fn get(&self) -> AppConfig {
        // Poison recovery: reading a potentially stale cache is acceptable — the value
        // is always a valid AppConfig, and the next set() will overwrite it.
        self.cached.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub(crate) fn set(&self, config: AppConfig) -> Result<(), ConfigError> {
        self.write_to_db(&config)?;
        // Poison recovery: same rationale as new() — overwriting with a freshly persisted value.
        *self.cached.write().unwrap_or_else(|e| e.into_inner()) = config;
        Ok(())
    }
}

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
/// Async to avoid blocking the main thread — `begin_write()` may block
/// if another write transaction is in progress.
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

    use redb::backends::InMemoryBackend;

    fn in_memory_db() -> Database {
        Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .expect("in-memory database")
    }

    // -------------------------------------------------------------------------
    // StoredAppConfig serialization
    // -------------------------------------------------------------------------

    #[test]
    fn stored_app_config_msgpack_roundtrip() {
        let original = StoredAppConfig::V1(AppConfig::default());
        let bytes = rmp_serde::to_vec_named(&original).expect("serialize");
        let decoded: StoredAppConfig = rmp_serde::from_slice(&bytes).expect("deserialize");
        // Verify the inner config survived the roundtrip.
        let StoredAppConfig::V1(config) = decoded;
        assert_eq!(config.popover.behavior.toggle_debounce_ms.0, 300);
    }

    #[test]
    fn stored_app_config_serializes_with_version_and_data_keys() {
        let stored = StoredAppConfig::V1(AppConfig::default());
        let json = serde_json::to_value(&stored).expect("serialize to JSON");

        // Adjacently tagged enum: { "version": "v1", "data": { ... } }
        assert_eq!(json["version"], "v1");
        assert!(json["data"].is_object(), "data key should contain an object");
        assert!(json["data"]["popover"].is_object(), "data should contain popover config");
    }

    // -------------------------------------------------------------------------
    // AppConfigStore
    // -------------------------------------------------------------------------

    #[test]
    fn new_store_returns_default_config_on_empty_db() {
        let store = AppConfigStore::new(in_memory_db()).expect("create store");
        let config = store.get();
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(300));
    }

    #[test]
    fn set_then_get_returns_updated_config() {
        let store = AppConfigStore::new(in_memory_db()).expect("create store");

        let mut config = store.get();
        config.popover.behavior.toggle_debounce_ms = Milliseconds(500);
        store.set(config).expect("persist config");

        let loaded = store.get();
        assert_eq!(loaded.popover.behavior.toggle_debounce_ms, Milliseconds(500));
    }

    // -------------------------------------------------------------------------
    // Default values — full coverage of all config fields
    // -------------------------------------------------------------------------

    #[test]
    fn default_popover_behavior_config() {
        let config = PopoverBehaviorConfig::default();
        assert_eq!(config.toggle_debounce_ms, Milliseconds(300));
    }

    #[test]
    fn default_popover_layout_config() {
        let config = PopoverLayoutConfig::default();
        assert_eq!(config.initial_width, LogicalPixel(480.0));
        assert_eq!(config.bottom_margin, LogicalPixel(8.0));
        assert_eq!(config.tray_gap, LogicalPixel(4.0));
        assert_eq!(config.fallback_max_height, LogicalPixel(600.0));
        assert_eq!(config.min_max_height, LogicalPixel(200.0));
        assert_eq!(config.header_height, CssLength::from("40px"));
        assert_eq!(config.card_list_padding, CssPadding::from("6px 0"));
    }

    #[test]
    fn default_popover_appearance_config() {
        let config = PopoverAppearanceConfig::default();
        assert_eq!(config.surface_color, CssColor::from("rgba(28, 28, 30, 0.92)"));
        assert_eq!(config.border_color, CssColor::from("rgba(255, 255, 255, 0.08)"));
        assert_eq!(config.backdrop_filter, CssFilter::from("blur(24px) saturate(180%)"));
        assert_eq!(
            config.font_family,
            CssFontFamily::from("-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif")
        );
    }

    // -------------------------------------------------------------------------
    // Newtype serde roundtrip — #[serde(transparent)]
    // -------------------------------------------------------------------------

    #[test]
    fn newtype_u64_json_roundtrip() {
        let ms = Milliseconds(42);
        let json = serde_json::to_string(&ms).expect("serialize");
        assert_eq!(json, "42");
        let decoded: Milliseconds = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, ms);
    }

    #[test]
    fn newtype_f64_json_roundtrip() {
        let px = LogicalPixel(12.5);
        let json = serde_json::to_string(&px).expect("serialize");
        assert_eq!(json, "12.5");
        let decoded: LogicalPixel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, px);
    }

    #[test]
    fn newtype_string_json_roundtrip() {
        let color = CssColor::from("rgba(0, 0, 0, 1)");
        let json = serde_json::to_string(&color).expect("serialize");
        assert_eq!(json, r#""rgba(0, 0, 0, 1)""#);
        let decoded: CssColor = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, color);
    }

    // -------------------------------------------------------------------------
    // serde(default) — missing fields filled from Default
    // -------------------------------------------------------------------------

    #[test]
    fn missing_fields_filled_from_default() {
        // Empty JSON object should deserialize to default values.
        let config: AppConfig = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(300));
        assert_eq!(config.popover.layout.initial_width, LogicalPixel(480.0));
        assert_eq!(
            config.popover.appearance.surface_color,
            CssColor::from("rgba(28, 28, 30, 0.92)")
        );
    }

    #[test]
    fn partial_fields_merged_with_defaults() {
        let json = r#"{"popover": {"behavior": {"toggleDebounceMs": 500}}}"#;
        let config: AppConfig = serde_json::from_str(json).expect("deserialize");
        // Overridden field
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(500));
        // Default-filled fields
        assert_eq!(config.popover.layout.initial_width, LogicalPixel(480.0));
        assert_eq!(
            config.popover.appearance.surface_color,
            CssColor::from("rgba(28, 28, 30, 0.92)")
        );
    }

    // -------------------------------------------------------------------------
    // AppConfigStore — table-exists-but-key-missing path
    // -------------------------------------------------------------------------

    #[test]
    fn load_returns_default_when_table_exists_but_key_missing() {
        let db = in_memory_db();

        // Create the table with a different key, so CONFIG_KEY is absent.
        {
            let write_txn = db.begin_write().expect("begin write");
            {
                let mut table = write_txn.open_table(CONFIG_TABLE).expect("open table");
                table
                    .insert("other_key", &StoredAppConfig::V1(AppConfig::default()))
                    .expect("insert");
            }
            write_txn.commit().expect("commit");
        }

        let store = AppConfigStore::new(db).expect("create store");
        let config = store.get();
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(300));
    }

    // -------------------------------------------------------------------------
    // AppConfigStore — persistence
    // -------------------------------------------------------------------------

    #[test]
    fn set_persists_to_db_and_survives_new_store() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.redb");

        // Write with first store instance, then drop it.
        {
            let db = Database::create(&db_path).expect("create db");
            let store = AppConfigStore::new(db).expect("create store");
            let mut config = store.get();
            config.popover.behavior.toggle_debounce_ms = Milliseconds(999);
            store.set(config).expect("persist config");
        }

        // Read with a new store to verify DB persistence.
        let db2 = Database::create(&db_path).expect("reopen db");
        let store2 = AppConfigStore::new(db2).expect("create store 2");
        assert_eq!(store2.get().popover.behavior.toggle_debounce_ms, Milliseconds(999));
    }
}
