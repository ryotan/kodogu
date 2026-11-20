use super::*;
use proptest::prelude::*;
use redb::{TypeName, Value};
use std::path::{Path, PathBuf};

const LEGACY_TYPE: &str = "MessagePack<kodogu_lib::config::store::StoredAppConfig>";
const LEGACY_TABLE: TableDefinition<&str, LegacyBytes<'static>> = TableDefinition::new("config");
// Captured from the original v1 named-map encoder with debounce changed to 999.
const FROZEN_V1: &[u8] = include_bytes!("fixtures/app-config-v1.msgpack");

// This fixture writes the historical metadata and bytes, independently of the production codec.
#[derive(Debug)]
struct LegacyBytes<'a>(&'a [u8]);

impl Value for LegacyBytes<'_> {
    type SelfType<'a>
        = LegacyBytes<'a>
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
        LegacyBytes(data)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.0
    }

    fn type_name() -> TypeName {
        TypeName::new(LEGACY_TYPE)
    }
}

fn seeded(bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("legacy.redb");
    let database = Database::create(&path).unwrap();
    let transaction = database.begin_write().unwrap();
    {
        let mut table = transaction.open_table(LEGACY_TABLE).unwrap();
        table.insert("app_config", LegacyBytes(bytes)).unwrap();
    }
    transaction.commit().unwrap();
    drop(database);
    (directory, path)
}

fn record(path: &Path) -> Vec<u8> {
    let database = Database::open(path).unwrap();
    record_in(&database)
}

fn record_in(database: &Database) -> Vec<u8> {
    let transaction = database.begin_read().unwrap();
    let table = transaction.open_table(LEGACY_TABLE).unwrap();
    table.get("app_config").unwrap().unwrap().value().0.to_vec()
}

#[test]
fn historical_table_metadata_is_not_derived_from_the_new_adapter() {
    assert_eq!(ConfigRecord::type_name().name(), LEGACY_TYPE);
}

#[test]
fn the_frozen_complete_v1_record_loads_without_rewriting() {
    let (_directory, path) = seeded(FROZEN_V1);
    let store = AppConfigStore::new(Database::open(&path).unwrap()).unwrap();
    let config = store.get();
    assert_eq!(
        serde_json::to_value(config).unwrap(),
        serde_json::json!({
            "popover": {
                "behavior": {"toggleDebounceMs":999},
                "layout": {"initialWidth":480.0,"bottomMargin":8.0,"trayGap":4.0,
                    "fallbackMaxHeight":600.0,"minMaxHeight":200.0,"headerHeight":"40px","cardListPadding":"6px 0"},
                "appearance": {"surfaceColor":"rgba(28, 28, 30, 0.92)", "borderColor":"rgba(255, 255, 255, 0.08)",
                    "backdropFilter":"blur(24px) saturate(180%)", "fontFamily":"-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif"}
            }
        })
    );
    drop(store);
    assert_eq!(record(&path), FROZEN_V1);
}

#[test]
fn writing_v1_preserves_the_historical_table_type_and_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.redb");
    let store = AppConfigStore::new(Database::create(&path).unwrap()).unwrap();
    let mut config = store.get();
    config.popover.behavior.toggle_debounce_ms = crate::config::domain::Milliseconds(999);
    store.set(config).unwrap();
    drop(store);

    assert_eq!(record(&path), FROZEN_V1);
}

macro_rules! rejected_record {
    ($($name:ident: $bytes:expr => $expected:pat $(if $condition:expr)?),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let bytes: &[u8] = $bytes;
            let (_directory, path) = seeded(bytes);
            let result = AppConfigStore::new(Database::open(&path).unwrap());
            let Err(error) = result else { panic!("Invalid stored record was accepted") };
            assert!(matches!(error, $expected $(if $condition)?));
            assert_eq!(record(&path), bytes);
        }
    )+};
}

rejected_record! {
    unknown_versions_are_typed_failures_without_rewriting: b"\x82\xa7version\xa2v2\xa4data\x80" => ConfigError::UnsupportedVersion { version, .. } if version == "v2",
    malformed_records_are_typed_failures_without_rewriting: b"\xc1" => ConfigError::CodecFailure { source: CodecError::Decode { .. }, .. },
    empty_records_are_typed_failures_without_rewriting: b"" => ConfigError::CodecFailure { source: CodecError::Decode { .. }, .. },
    truncated_records_are_typed_failures_without_rewriting: b"\x82\xa7version\xa2v1\xa4data\x81" => ConfigError::CodecFailure { source: CodecError::Decode { .. }, .. },
    trailing_data_is_rejected_without_rewriting: b"\x82\xa7version\xa2v1\xa4data\x80\xc0" => ConfigError::CodecFailure { source: CodecError::TrailingData { .. }, .. },
    future_data_is_not_interpreted_as_current_config: b"\x82\xa7version\xa2v2\xa4data\x01" => ConfigError::UnsupportedVersion { version, .. } if version == "v2",
    a_future_version_can_follow_its_payload: b"\x82\xa4data\x01\xa7version\xa2v2" => ConfigError::UnsupportedVersion { version, .. } if version == "v2",
    a_numeric_version_is_not_a_v1_record: b"\x82\xa7version\x00\xa4data\x80" => ConfigError::CodecFailure { source: CodecError::Decode { .. }, .. },
    a_map_version_is_not_a_v1_record: b"\x82\xa7version\x81\xa2v1\xc0\xa4data\x80" => ConfigError::CodecFailure { source: CodecError::Decode { .. }, .. },
    a_positional_envelope_is_not_a_v1_record: b"\x92\xa2v1\x80" => ConfigError::CodecFailure { source: CodecError::Decode { .. }, .. },
}

macro_rules! compatible_record {
    ($($name:ident: $bytes:expr),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let bytes: &[u8] = $bytes;
            let (_directory, path) = seeded(bytes);
            let store = AppConfigStore::new(Database::open(&path).unwrap()).unwrap();
            let config = store.get();
            assert_eq!(config.popover.behavior.toggle_debounce_ms.0, 1);
            assert_eq!(config.popover.layout.initial_width.0, 480.0);
            drop(store);
            assert_eq!(record(&path), bytes);
        }
    )+};
}

compatible_record! {
    missing_compatible_fields_use_defaults_without_rewriting: b"\x82\xa7version\xa2v1\xa4data\x81\xa7popover\x81\xa8behavior\x81\xb0toggleDebounceMs\x01",
    unknown_compatible_fields_are_ignored_without_rewriting: b"\x82\xa7version\xa2v1\xa4data\x82\xa7popover\x81\xa8behavior\x81\xb0toggleDebounceMs\x01\xa6future\x81\xa1x\x02",
}

proptest! {
    #[test]
    fn the_opaque_adapter_preserves_every_byte_without_interpreting_it(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
        let value = ConfigRecord::from_bytes(&bytes);
        prop_assert_eq!(ConfigRecord::as_bytes(&value), bytes.as_slice());
    }

    #[test]
    fn decoding_any_record_produces_a_result_without_panicking(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let result = std::panic::catch_unwind(|| decode_config(&ConfigRecord::from_bytes(&bytes)));
        prop_assert!(result.is_ok());
    }
}
