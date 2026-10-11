use std::sync::RwLock;

use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use suzunari_error::*;

use super::domain::AppConfig;
use crate::store::MessagePack;

const CONFIG_TABLE: TableDefinition<&str, MessagePack<StoredAppConfig>> =
    TableDefinition::new("config");
const CONFIG_KEY: &str = "app_config";

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
///   3. Add the `V2(AppConfig)` variant (serialized as `"v2"` due to `rename_all = "camelCase"`)
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
    /// Holds a whole `AppConfig` even after a panic: `set()` can only panic before it replaces the value.
    cached: RwLock<AppConfig>,
}

impl AppConfigStore {
    /// Creates a new store from a database handle and loads the stored config.
    /// On first launch when no config exists, writes and caches `AppConfig::default()`.
    pub(crate) fn new(db: Database) -> Result<Self, ConfigError> {
        let mut store = Self { db, cached: RwLock::new(AppConfig::default()) };
        let config = store.load_from_db()?;
        store.cached = RwLock::new(config);
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
        // Safe to recover from poisoning; see `cached`.
        self.cached.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub(crate) fn set(&self, config: AppConfig) -> Result<(), ConfigError> {
        // Hold the write lock across DB write + cache update so readers never see a stale cache
        // after a successful DB commit.
        // Safe to recover from poisoning; see `cached`.
        let mut cache = self.cached.write().unwrap_or_else(|e| e.into_inner());
        self.write_to_db(&config)?;
        *cache = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use redb::backends::InMemoryBackend;

    use super::super::domain::Milliseconds;

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
