use std::sync::RwLock;

use redb::{Database, ReadableDatabase, TableDefinition, Value};
use serde::{Deserialize, Deserializer, Serialize, de::IgnoredAny};
use suzunari_error::*;

use super::domain::AppConfig;
use crate::store::codec::{self, CodecError, MessagePack, Record};

type ConfigRecord<'a> = MessagePack<'a, StoredAppConfig>;

const CONFIG_TABLE: TableDefinition<&str, ConfigRecord<'static>> = TableDefinition::new("config");
const CONFIG_KEY: &str = "app_config";

// =============================================================================
// Versioned storage envelope
// =============================================================================

/// Versioned storage envelope for `AppConfig`.
///
/// For a breaking change, retain the v1 payload type and add a variant for the new schema.
/// Convert older variants in `decode_config` and persist the migration in a write transaction.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version", content = "data", rename_all = "camelCase")]
enum StoredAppConfig<T = AppConfig> {
    V1(T),
    #[serde(other, skip_serializing, deserialize_with = "ignore_stored_data")]
    Unsupported,
}

impl Record for StoredAppConfig {
    // redb persists this type name; changing it makes existing configuration tables unreadable.
    const TYPE_NAME: &'static str = "MessagePack<kodogu_lib::config::store::StoredAppConfig>";
    type Encode<'a> = StoredAppConfig<&'a AppConfig>;
}

fn ignore_stored_data<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    IgnoredAny::deserialize(deserializer).map(|_| ())
}

#[derive(Deserialize)]
struct VersionHeader {
    version: String,
}

fn decode_config(record: &ConfigRecord<'_>) -> Result<AppConfig, ConfigError> {
    // Read the version as text before Serde can interpret a numeric enum index.
    let header: VersionHeader =
        codec::decode(ConfigRecord::as_bytes(record)).context(CodecFailureError)?;
    match record.decode().context(CodecFailureError)? {
        StoredAppConfig::V1(config) => Ok(config),
        StoredAppConfig::Unsupported => {
            None.context(UnsupportedVersionError { version: header.version })
        }
    }
}

// =============================================================================
// Error types
// =============================================================================

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum ConfigError {
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

// =============================================================================
// Store
// =============================================================================

/// Manages a shared redb database handle and a cached in-memory copy of `AppConfig`.
pub(crate) struct AppConfigStore {
    db: Database,
    cached: RwLock<AppConfig>,
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
                let default = AppConfig::default();
                Self::write_to_db(&db, &default)?;
                default
            }
        };
        Ok(Self { db, cached: RwLock::new(config) })
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
            .map(|guard| decode_config(&guard.value()))
            .transpose()
    }

    fn write_to_db(db: &Database, config: &AppConfig) -> Result<(), ConfigError> {
        // Encode first so a value the codec rejects never starts a write transaction.
        let encoded =
            ConfigRecord::encode(&StoredAppConfig::V1(config)).context(CodecFailureError)?;
        let write_txn = db.begin_write().context(TransactionError {})?;
        {
            let mut table = write_txn.open_table(CONFIG_TABLE).context(TableError {})?;
            table.insert(CONFIG_KEY, encoded).context(StorageError {})?;
        }
        write_txn.commit().context(CommitError {})
    }

    pub(crate) fn get(&self) -> AppConfig {
        self.cached.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub(crate) fn set(&self, config: AppConfig) -> Result<(), ConfigError> {
        // Hold the write lock across DB write + cache update so readers never see a stale cache
        // after a successful DB commit.
        let mut cache = self.cached.write().unwrap_or_else(|e| e.into_inner());
        Self::write_to_db(&self.db, &config)?;
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
        assert!(
            matches!(decoded, StoredAppConfig::V1(config) if config.popover.behavior.toggle_debounce_ms.0 == 300)
        );
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
                    .insert(
                        "other_key",
                        ConfigRecord::encode(&StoredAppConfig::V1(&AppConfig::default())).unwrap(),
                    )
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

    // -------------------------------------------------------------------------
    // Shared MessagePack adapter — a second record type
    // -------------------------------------------------------------------------

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Note {
        text: String,
    }

    impl Record for Note {
        const TYPE_NAME: &'static str = "MessagePack<kodogu_lib::config::store::tests::Note>";
        type Encode<'a> = Note;
    }

    const NOTE_TABLE: TableDefinition<&str, MessagePack<'static, Note>> =
        TableDefinition::new("notes");
    const NOTE_AS_CONFIG_TABLE: TableDefinition<&str, MessagePack<'static, Note>> =
        TableDefinition::new("config");

    #[test]
    fn a_second_record_type_reuses_the_adapter_but_not_the_settings_table() {
        let store = AppConfigStore::new(in_memory_db()).expect("create store");
        let note = Note { text: "second record".to_string() };

        let write_txn = store.db.begin_write().expect("begin write");
        {
            let mut table = write_txn.open_table(NOTE_TABLE).expect("open table");
            let encoded = MessagePack::<Note>::encode(&note).expect("encode");
            table.insert("note", encoded).expect("insert");
        }
        write_txn.commit().expect("commit");

        let read_txn = store.db.begin_read().expect("begin read");
        let table = read_txn.open_table(NOTE_TABLE).expect("open table");
        let stored = table.get("note").expect("read").expect("note exists");
        assert_eq!(stored.value().decode().expect("decode"), note);

        // Naming the settings table does not help: redb rejects the other record's identity.
        let write_txn = store.db.begin_write().expect("begin write");
        assert!(matches!(
            write_txn.open_table(NOTE_AS_CONFIG_TABLE),
            Err(redb::TableError::TableTypeMismatch { .. })
        ));
    }
}
