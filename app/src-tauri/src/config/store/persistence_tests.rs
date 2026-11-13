use super::*;
use proptest::prelude::*;
use redb::{TypeName, Value};
use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

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
    let config = store.get().unwrap();
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
}

macro_rules! compatible_record {
    ($($name:ident: $bytes:expr),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let bytes: &[u8] = $bytes;
            let (_directory, path) = seeded(bytes);
            let store = AppConfigStore::new(Database::open(&path).unwrap()).unwrap();
            let config = store.get().unwrap();
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

#[derive(Debug, Default)]
struct IoControl {
    fail_sync: AtomicBool,
    calls: AtomicUsize,
}

#[derive(Debug)]
struct ControlledBackend {
    memory: redb::backends::InMemoryBackend,
    control: Arc<IoControl>,
}

impl redb::StorageBackend for ControlledBackend {
    fn len(&self) -> io::Result<u64> {
        self.control.calls.fetch_add(1, Ordering::SeqCst);
        self.memory.len()
    }

    fn read(&self, offset: u64, output: &mut [u8]) -> io::Result<()> {
        self.control.calls.fetch_add(1, Ordering::SeqCst);
        self.memory.read(offset, output)
    }

    fn set_len(&self, length: u64) -> io::Result<()> {
        self.control.calls.fetch_add(1, Ordering::SeqCst);
        self.memory.set_len(length)
    }

    fn sync_data(&self) -> io::Result<()> {
        self.control.calls.fetch_add(1, Ordering::SeqCst);
        if self.control.fail_sync.load(Ordering::SeqCst) {
            Err(io::Error::other("Controlled sync failure"))
        } else {
            self.memory.sync_data()
        }
    }

    fn write(&self, offset: u64, bytes: &[u8]) -> io::Result<()> {
        self.control.calls.fetch_add(1, Ordering::SeqCst);
        self.memory.write(offset, bytes)
    }
}

fn controlled_store() -> (AppConfigStore, Arc<IoControl>) {
    let control = Arc::new(IoControl::default());
    let database = Database::builder()
        .create_with_backend(ControlledBackend {
            memory: redb::backends::InMemoryBackend::new(),
            control: control.clone(),
        })
        .unwrap();
    (AppConfigStore::new(database).unwrap(), control)
}

#[test]
fn an_ambiguous_commit_fences_reads_and_writes_with_the_original_cause() {
    let (store, control) = controlled_store();
    let config = store.get().unwrap();
    control.fail_sync.store(true, Ordering::SeqCst);
    let first = store.set(config.clone().try_into().unwrap()).unwrap_err();
    let ConfigError::Unavailable { source: original, .. } = first else {
        panic!("A failed commit did not fence the store")
    };
    assert!(matches!(&*original, ConfigError::Commit { .. }));
    let calls = control.calls.load(Ordering::SeqCst);
    control.fail_sync.store(false, Ordering::SeqCst);
    let ConfigError::Unavailable { source: reading, .. } = store.get().unwrap_err() else {
        panic!("An unavailable store returned its previous cache")
    };
    let ConfigError::Unavailable { source: writing, .. } =
        store.set(config.try_into().unwrap()).unwrap_err()
    else {
        panic!("An unavailable store retried a write")
    };
    assert!(Arc::ptr_eq(&original, &reading));
    assert!(Arc::ptr_eq(&original, &writing));
    assert_eq!(control.calls.load(Ordering::SeqCst), calls);
}

#[tokio::test]
async fn native_commands_do_not_return_old_configuration_after_a_failed_commit() {
    use crate::config::commands::{get_config, set_config};
    use tauri::Manager;
    let (store, control) = controlled_store();
    let app = configuration_app(store);
    let previous = get_config(app.state()).await.unwrap();
    control.fail_sync.store(true, Ordering::SeqCst);
    let update = set_config(app.state(), serde_json::to_value(previous).unwrap())
        .await
        .unwrap_err();
    let calls = control.calls.load(Ordering::SeqCst);
    let reading = get_config(app.state()).await.unwrap_err();
    let malformed = set_config(app.state(), serde_json::json!({"privateMalformedValue": "secret"}))
        .await
        .unwrap_err();
    let expected = serde_json::json!({"code":"configuration_unavailable", "message":"Configuration storage is unavailable"});
    assert_eq!(serde_json::to_value(update).unwrap(), expected);
    assert_eq!(serde_json::to_value(reading).unwrap(), expected);
    assert_eq!(serde_json::to_value(malformed).unwrap(), expected);
    assert_eq!(control.calls.load(Ordering::SeqCst), calls);
    control.fail_sync.store(false, Ordering::SeqCst);
}

fn configuration_app(store: AppConfigStore) -> tauri::App<tauri::test::MockRuntime> {
    use tauri::Manager;
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    app.manage(store);
    app
}

async fn assert_response_rejection_preserves_the_saved_record(config: AppConfig) {
    use crate::config::commands::get_config;
    use tauri::Manager;
    let bytes = rmp_serde::to_vec_named(&StoredAppConfig::V1(config)).unwrap();
    let (_directory, path) = seeded(&bytes);
    let store = AppConfigStore::new(Database::open(&path).unwrap()).unwrap();
    let app = configuration_app(store);
    let error = get_config(app.state()).await.unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        serde_json::json!({
            "code":"configuration_unrepresentable",
            "message":"Configuration cannot be represented by the GUI"
        })
    );
    let raw = app.state::<AppConfigStore>().get().unwrap();
    assert_eq!(rmp_serde::to_vec_named(&StoredAppConfig::V1(raw)).unwrap(), bytes);
    assert_eq!(record_in(&app.state::<AppConfigStore>().db), bytes);
}

macro_rules! unrepresentable_records {
    ($($name:ident: $modify:expr),+ $(,)?) => {$(
        #[tokio::test]
        async fn $name() {
            let mut config = AppConfig::default();
            ($modify)(&mut config);
            assert_response_rejection_preserves_the_saved_record(config).await;
        }
    )+};
}

unrepresentable_records! {
    saved_nan_is_not_exposed_as_null_or_rewritten:
        |config:&mut AppConfig| config.popover.layout.initial_width = crate::config::domain::LogicalPixel(f64::NAN),
    saved_infinity_is_not_exposed_as_null_or_rewritten:
        |config:&mut AppConfig| config.popover.layout.tray_gap = crate::config::domain::LogicalPixel(f64::INFINITY),
    saved_negative_infinity_is_not_exposed_as_null_or_rewritten:
        |config:&mut AppConfig| config.popover.layout.bottom_margin = crate::config::domain::LogicalPixel(f64::NEG_INFINITY),
    a_saved_unsafe_javascript_integer_is_not_rounded_or_rewritten:
        |config:&mut AppConfig| config.popover.behavior.toggle_debounce_ms = crate::config::domain::Milliseconds(9_007_199_254_740_992),
    a_saved_native_maximum_duration_is_not_rounded_or_rewritten:
        |config:&mut AppConfig| config.popover.behavior.toggle_debounce_ms = crate::config::domain::Milliseconds(u64::MAX),
}

#[tokio::test]
async fn a_legacy_finite_zero_or_negative_size_remains_readable_and_can_be_corrected() {
    use crate::config::{
        commands::{get_config, set_config},
        domain::LogicalPixel,
    };
    use tauri::Manager;
    let mut config = AppConfig::default();
    config.popover.layout.initial_width = LogicalPixel(0.0);
    config.popover.layout.fallback_max_height = LogicalPixel(-1.0);
    config.popover.layout.min_max_height = LogicalPixel(-2.0);
    let bytes = rmp_serde::to_vec_named(&StoredAppConfig::V1(&config)).unwrap();
    let (_directory, path) = seeded(&bytes);
    let app = configuration_app(AppConfigStore::new(Database::open(&path).unwrap()).unwrap());
    assert_eq!(
        serde_json::to_value(get_config(app.state()).await.unwrap()).unwrap(),
        serde_json::to_value(config).unwrap()
    );
    let replacement = serde_json::to_value(AppConfig::default()).unwrap();
    set_config(app.state(), replacement.clone()).await.unwrap();
    assert_eq!(
        serde_json::to_value(get_config(app.state()).await.unwrap()).unwrap(),
        replacement
    );
    assert_eq!(
        serde_json::to_value(decode_config(&record_in(&app.state::<AppConfigStore>().db)).unwrap())
            .unwrap(),
        replacement
    );
}

#[tokio::test]
async fn a_response_translation_failure_does_not_fence_a_later_valid_full_update() {
    use crate::config::{
        commands::{get_config, set_config},
        domain::LogicalPixel,
    };
    use tauri::Manager;
    let mut config = AppConfig::default();
    config.popover.layout.initial_width = LogicalPixel(f64::NAN);
    let bytes = rmp_serde::to_vec_named(&StoredAppConfig::V1(config)).unwrap();
    let (_directory, path) = seeded(&bytes);
    let app = configuration_app(AppConfigStore::new(Database::open(&path).unwrap()).unwrap());
    assert!(get_config(app.state()).await.is_err());
    let replacement = serde_json::to_value(AppConfig::default()).unwrap();
    set_config(app.state(), replacement.clone()).await.unwrap();
    assert_eq!(
        serde_json::to_value(get_config(app.state()).await.unwrap()).unwrap(),
        replacement
    );
    assert_eq!(
        serde_json::to_value(decode_config(&record_in(&app.state::<AppConfigStore>().db)).unwrap())
            .unwrap(),
        replacement
    );
}

#[test]
fn concurrent_successful_updates_publish_the_same_complete_record_as_the_database() {
    use crate::config::domain::{LogicalPixel, Milliseconds};
    let store = Arc::new(
        AppConfigStore::new(
            Database::builder()
                .create_with_backend(redb::backends::InMemoryBackend::new())
                .unwrap(),
        )
        .unwrap(),
    );
    let make_config = |value| {
        let mut config = AppConfig::default();
        config.popover.behavior.toggle_debounce_ms = Milliseconds(value);
        config.popover.layout.initial_width = LogicalPixel(value as f64);
        config
    };
    store.set(make_config(1).try_into().unwrap()).unwrap();
    std::thread::scope(|scope| {
        for value in 2..10 {
            let store = &store;
            let make_config = &make_config;
            scope.spawn(move || {
                for _ in 0..8 {
                    store.set(make_config(value).try_into().unwrap()).unwrap();
                    let snapshot = store.get().unwrap();
                    assert_eq!(
                        snapshot.popover.layout.initial_width.0,
                        snapshot.popover.behavior.toggle_debounce_ms.0 as f64
                    );
                }
            });
        }
    });
    let transaction = store.db.begin_read().unwrap();
    let table = transaction.open_table(LEGACY_TABLE).unwrap();
    let guard = table.get("app_config").unwrap().unwrap();
    let committed = decode_config(guard.value().0).unwrap();
    assert_eq!(
        serde_json::to_value(committed).unwrap(),
        serde_json::to_value(store.get().unwrap()).unwrap()
    );
}

proptest! {
    #[test]
    fn the_opaque_adapter_preserves_every_byte_without_interpreting_it(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
        let value = ConfigRecord::from_bytes(&bytes);
        prop_assert_eq!(ConfigRecord::as_bytes(&value), bytes.as_slice());
    }

    #[test]
    fn decoding_any_record_produces_a_result_without_panicking(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let result = std::panic::catch_unwind(|| decode_config(&bytes));
        prop_assert!(result.is_ok());
    }
}
