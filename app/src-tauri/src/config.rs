use std::sync::{Arc, RwLock};

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
        #[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[serde(tag = "version", rename_all = "SCREAMING_SNAKE_CASE")]
enum StoredAppConfig {
    V1(AppConfig),
}

// =============================================================================
// Error types
// =============================================================================

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum ConfigError {
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
    db: Arc<Database>,
    cached: RwLock<AppConfig>,
}

impl AppConfigStore {
    /// Creates a new store from a shared database handle and loads the stored config.
    /// On first launch when no config exists, writes and caches `AppConfig::default()`.
    pub(crate) fn new(db: Arc<Database>) -> Result<Self, ConfigError> {
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
            // First launch: table does not exist yet
            Err(redb::TableError::TableDoesNotExist(_)) => {
                drop(read_txn);
                let default = AppConfig::default();
                self.write_to_db(&default)?;
                return Ok(default);
            }
            Err(e) => Err(e).context(TableError {})?,
        };

        let Some(guard) = table.get(CONFIG_KEY).context(StorageError {})? else {
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
#[tauri::command]
pub(crate) fn get_config(state: State<'_, AppConfigStore>) -> AppConfig {
    state.get()
}

/// Persists the given configuration and updates the in-memory cache.
#[tauri::command]
pub(crate) fn set_config(
    state: State<'_, AppConfigStore>,
    config: AppConfig,
) -> Result<(), CommandError> {
    state.set(config).map_err(CommandError::from)
}
