use std::sync::{Arc, RwLock};

use redb::{Database, ReadableDatabase, TableDefinition, TypeName, Value};
use serde::{Deserialize, Serialize};
use suzunari_error::*;

use super::domain::{AppConfig, ConfigUpdate, ConfigValidationError};
use crate::store::codec::{self, CodecError};

const CONFIG_TABLE: TableDefinition<&str, ConfigRecord<'static>> = TableDefinition::new("config");
const CONFIG_KEY: &str = "app_config";

// redb persists this type name; changing it makes existing configuration tables unreadable.
const CONFIG_VALUE_TYPE: &str = "MessagePack<kodogu_lib::config::store::StoredAppConfig>";

#[derive(Debug)]
struct ConfigRecord<'a>(&'a [u8]);

impl Value for ConfigRecord<'_> {
    type SelfType<'a>
        = ConfigRecord<'a>
    where
        Self: 'a;
    type AsBytes<'a>
        = &'a [u8]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        ConfigRecord(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.0
    }

    fn type_name() -> TypeName {
        TypeName::new(CONFIG_VALUE_TYPE)
    }
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
///   3. Add the `V2(AppConfig)` variant (serialized as `"v2"` due to `rename_all = "camelCase"`)
///   4. Implement migration in `AppConfigStore::load_from_db`
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version", content = "data", rename_all = "camelCase")]
enum StoredAppConfig<T = AppConfig> {
    V1(T),
}

#[derive(Deserialize)]
struct VersionHeader {
    version: String,
}

fn decode_config(bytes: &[u8]) -> Result<AppConfig, ConfigError> {
    let header: VersionHeader = codec::decode(bytes).context(CodecFailureError)?;
    ensure!(header.version == "v1", UnsupportedVersionError { version: header.version });
    match codec::decode::<StoredAppConfig>(bytes).context(CodecFailureError)? {
        StoredAppConfig::V1(config) => Ok(config),
    }
}

// =============================================================================
// Error types
// =============================================================================

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum ConfigError {
    #[suzu(display("Configuration storage is unavailable"))]
    Unavailable { source: Arc<ConfigError> },
    #[suzu(display("Configuration cache is poisoned"))]
    CachePoisoned,
    #[suzu(display("Default configuration is invalid"))]
    InvalidDefault { source: ConfigValidationError },
    #[suzu(display("Failed to translate stored configuration"))]
    CodecFailure { source: CodecError },
    #[suzu(display("Unsupported stored configuration version {version}"))]
    UnsupportedVersion { version: String },
    #[suzu(display("Failed to begin database transaction"))]
    Transaction { source: redb::TransactionError },
    #[suzu(display("Failed to access config table"))]
    Table { source: redb::TableError },
    #[suzu(display("Failed to access configuration storage"))]
    Storage { source: redb::StorageError },
    #[suzu(display("Configuration commit did not complete cleanly"))]
    Commit { source: redb::CommitError },
}

impl ConfigError {
    pub(super) fn is_known_rejection(&self) -> bool {
        matches!(self, Self::Storage { source: redb::StorageError::ValueTooLarge(_), .. })
    }
}

enum CacheState {
    Available(AppConfig),
    Unavailable(Arc<ConfigError>),
}

impl CacheState {
    fn current(&self) -> Result<&AppConfig, ConfigError> {
        match self {
            Self::Available(config) => Ok(config),
            Self::Unavailable(cause) => Err(Arc::clone(cause)).context(UnavailableError),
        }
    }
}

// =============================================================================
// Store
// =============================================================================

/// Manages a shared redb database handle and a cached in-memory copy of `AppConfig`.
pub(crate) struct AppConfigStore {
    db: Database,
    cached: RwLock<CacheState>,
}

#[cfg(test)]
mod persistence_tests;

impl AppConfigStore {
    /// Creates a new store from a database handle and loads the stored config.
    /// On first launch when no config exists, writes and caches `AppConfig::default()`.
    pub(crate) fn new(db: Database) -> Result<Self, ConfigError> {
        let config = match Self::read_from_db(&db)? {
            Some(config) => config,
            None => {
                let default =
                    ConfigUpdate::try_from(AppConfig::default()).context(InvalidDefaultError)?;
                let encoded = codec::encode(&StoredAppConfig::V1(default.as_ref()))
                    .context(CodecFailureError)?;
                Self::write_encoded(&db, &encoded)?;
                default.into_inner()
            }
        };
        Ok(Self { db, cached: RwLock::new(CacheState::Available(config)) })
    }

    fn read_from_db(db: &Database) -> Result<Option<AppConfig>, ConfigError> {
        // Returning releases every read guard before absence can trigger the first-launch write.
        let transaction = db.begin_read().context(TransactionError {})?;
        let table = match transaction.open_table(CONFIG_TABLE) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(error) => return Err(error).context(TableError {}),
        };
        table
            .get(CONFIG_KEY)
            .context(StorageError {})?
            .map(|guard| decode_config(guard.value().0))
            .transpose()
    }

    fn write_encoded(db: &Database, encoded: &[u8]) -> Result<(), ConfigError> {
        let write_txn = db.begin_write().context(TransactionError {})?;
        {
            let mut table = write_txn.open_table(CONFIG_TABLE).context(TableError {})?;
            table.insert(CONFIG_KEY, ConfigRecord(encoded)).context(StorageError {})?;
        }
        write_txn.commit().context(CommitError {})
    }

    pub(crate) fn get(&self) -> Result<AppConfig, ConfigError> {
        self.cached.read().ok().context(CachePoisonedError)?.current().cloned()
    }

    pub(crate) fn ensure_available(&self) -> Result<(), ConfigError> {
        self.cached.read().ok().context(CachePoisonedError)?.current().map(|_| ())
    }

    pub(crate) fn set(&self, update: ConfigUpdate) -> Result<(), ConfigError> {
        self.ensure_available()?;
        let config = update.into_inner();
        let encoded = codec::encode(&StoredAppConfig::V1(&config)).context(CodecFailureError)?;
        // Publication and durable commit share one lock; another writer can invalidate during encoding.
        let mut cache = self.cached.write().ok().context(CachePoisonedError)?;
        cache.current()?;
        if let Err(error) = Self::write_encoded(&self.db, &encoded) {
            if error.is_known_rejection() {
                return Err(error);
            }
            let cause = Arc::new(error);
            *cache = CacheState::Unavailable(cause.clone());
            return Err(cause).context(UnavailableError);
        }
        *cache = CacheState::Available(config);
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
        let config = store.get().unwrap();
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(300));
    }

    #[test]
    fn set_then_get_returns_updated_config() {
        let store = AppConfigStore::new(in_memory_db()).expect("create store");

        let mut config = store.get().unwrap();
        config.popover.behavior.toggle_debounce_ms = Milliseconds(500);
        store.set(config.try_into().unwrap()).expect("persist config");

        let loaded = store.get().unwrap();
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
                    .insert(
                        "other_key",
                        ConfigRecord(
                            &codec::encode(&StoredAppConfig::V1(AppConfig::default())).unwrap(),
                        ),
                    )
                    .expect("insert");
            }
            write_txn.commit().expect("commit");
        }

        let store = AppConfigStore::new(db).expect("create store");
        let config = store.get().unwrap();
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
            let mut config = store.get().unwrap();
            config.popover.behavior.toggle_debounce_ms = Milliseconds(999);
            store.set(config.try_into().unwrap()).expect("persist config");
        }

        // Read with a new store to verify DB persistence.
        let db2 = Database::create(&db_path).expect("reopen db");
        let store2 = AppConfigStore::new(db2).expect("create store 2");
        assert_eq!(store2.get().unwrap().popover.behavior.toggle_debounce_ms, Milliseconds(999));
    }
}
