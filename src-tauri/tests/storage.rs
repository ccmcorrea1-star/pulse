use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use pulse_lib::runtime::{RuntimeBuilder, RuntimePhase, ServiceKind, ServiceStatus};
use pulse_lib::storage::{
    HistoryRecord, KnownDeviceRecord, NotificationRecord, PreferenceKey, Storage, StorageError,
    StorageService, TrustRecord, SUPPORTED_SCHEMA_VERSION,
};
use rusqlite::params;

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TempDatabase {
    directory: PathBuf,
    path: PathBuf,
}

impl TempDatabase {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "pulse-storage-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("test directory should be created");
        let path = directory.join("pulse.sqlite");
        Self { directory, path }
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn device_record() -> KnownDeviceRecord {
    KnownDeviceRecord {
        device_id: "device-phone".to_owned(),
        public_key: "public-key".to_owned(),
        fingerprint: "fingerprint-phone".to_owned(),
        name: "Telefone".to_owned(),
        platform: "android".to_owned(),
        model: Some("Test phone".to_owned()),
        platform_version: Some("1".to_owned()),
        last_seen_at: Some("2026-08-15T10:00:00Z".to_owned()),
    }
}

#[test]
fn first_install_and_restart_preserve_only_metadata() {
    let database = TempDatabase::new();
    {
        let mut storage = Storage::open(&database.path).expect("new storage should open");
        assert_eq!(storage.schema_version().unwrap(), SUPPORTED_SCHEMA_VERSION);
        storage.save_known_device(&device_record()).unwrap();
        storage
            .save_trust(&TrustRecord {
                device_id: "device-phone".to_owned(),
                state: "trusted".to_owned(),
                updated_at: "2026-08-15T10:01:00Z".to_owned(),
                decided_at: Some("2026-08-15T10:01:00Z".to_owned()),
                revoked_at: None,
                reason_code: Some("pairing-confirmed".to_owned()),
                pairing_session_id: Some("pairing-1".to_owned()),
            })
            .unwrap();
        storage
            .set_preference(
                PreferenceKey::HistoryRetentionDays,
                "90",
                "2026-08-15T10:02:00Z",
            )
            .unwrap();
        storage
            .save_history_metadata(&HistoryRecord {
                id: "history-1".to_owned(),
                entry_type: "transfer".to_owned(),
                source_device_id: Some("device-phone".to_owned()),
                target_device_id: None,
                result: "succeeded".to_owned(),
                occurred_at: "2026-08-15T10:03:00Z".to_owned(),
                recorded_at: "2026-08-15T10:03:00Z".to_owned(),
                related_kind: "transfer".to_owned(),
                related_id: "transfer-1".to_owned(),
                reason_code: None,
            })
            .unwrap();
    }

    let storage = Storage::open(&database.path).expect("restarting storage should preserve data");
    assert_eq!(
        storage.known_device("device-phone").unwrap(),
        Some(device_record())
    );
    assert_eq!(
        storage
            .preference(PreferenceKey::HistoryRetentionDays)
            .unwrap(),
        Some("90".to_owned())
    );
    assert_eq!(storage.table_row_count("history_entries").unwrap(), 1);
    assert_eq!(storage.table_row_count("trust_relationships").unwrap(), 1);
}

#[test]
fn future_schema_is_rejected_without_downgrade() {
    let database = TempDatabase::new();
    Storage::open(&database.path).unwrap();
    let connection = rusqlite::Connection::open(&database.path).unwrap();
    connection
        .execute(
            "INSERT INTO schema_migrations (version, name, checksum, applied_at, result)
             VALUES (?1, 'future', ?2, 'future', 'applied')",
            params![99_u32, "0".repeat(64)],
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        Storage::open(&database.path),
        Err(StorageError::Incompatible {
            found: 99,
            supported: SUPPORTED_SCHEMA_VERSION,
        })
    ));
}

#[test]
fn foreign_keys_are_enforced_at_the_storage_boundary() {
    let database = TempDatabase::new();
    let mut storage = Storage::open(&database.path).unwrap();
    let error = storage
        .save_trust(&TrustRecord {
            device_id: "unknown-device".to_owned(),
            state: "trusted".to_owned(),
            updated_at: "2026-08-15T10:01:00Z".to_owned(),
            decided_at: None,
            revoked_at: None,
            reason_code: None,
            pairing_session_id: None,
        })
        .expect_err("trust must reference a known device");
    assert_eq!(error, StorageError::ForeignKeyViolation);
}

#[test]
fn applied_migration_checksum_cannot_be_changed() {
    let database = TempDatabase::new();
    Storage::open(&database.path).unwrap();
    let connection = rusqlite::Connection::open(&database.path).unwrap();
    connection
        .execute(
            "UPDATE schema_migrations SET checksum = ?1 WHERE version = 1",
            params!["0".repeat(64)],
        )
        .unwrap();
    drop(connection);

    assert!(matches!(
        Storage::open(&database.path),
        Err(StorageError::MigrationChecksumMismatch { version: 1 })
    ));
}

#[test]
fn corrupt_file_is_not_replaced_with_an_empty_database() {
    let database = TempDatabase::new();
    fs::write(&database.path, b"not a sqlite database").unwrap();

    assert!(matches!(
        Storage::open(&database.path),
        Err(StorageError::Corrupt)
    ));
    assert_eq!(fs::read(&database.path).unwrap(), b"not a sqlite database");
}

#[test]
fn explicit_cleanup_respects_data_boundaries() {
    let database = TempDatabase::new();
    let mut storage = Storage::open(&database.path).unwrap();
    storage
        .save_history_metadata(&HistoryRecord {
            id: "history-old".to_owned(),
            entry_type: "trust".to_owned(),
            source_device_id: None,
            target_device_id: None,
            result: "succeeded".to_owned(),
            occurred_at: "2026-01-01T00:00:00Z".to_owned(),
            recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            related_kind: "trust".to_owned(),
            related_id: "device-phone".to_owned(),
            reason_code: None,
        })
        .unwrap();
    storage
        .save_history_metadata(&HistoryRecord {
            id: "history-new".to_owned(),
            entry_type: "trust".to_owned(),
            source_device_id: None,
            target_device_id: None,
            result: "succeeded".to_owned(),
            occurred_at: "2026-08-01T00:00:00Z".to_owned(),
            recorded_at: "2026-08-01T00:00:00Z".to_owned(),
            related_kind: "trust".to_owned(),
            related_id: "device-phone".to_owned(),
            reason_code: None,
        })
        .unwrap();
    storage
        .save_notification_metadata(&NotificationRecord {
            id: "notification-old".to_owned(),
            severity: "info".to_owned(),
            title_key: "title".to_owned(),
            body_key: "body".to_owned(),
            source_event_id: "event-1".to_owned(),
            state: "expired".to_owned(),
            queued_at: "2026-01-01T00:00:00Z".to_owned(),
            updated_at: "2026-01-01T00:00:00Z".to_owned(),
            expires_at: Some("2026-01-02T00:00:00Z".to_owned()),
        })
        .unwrap();

    assert_eq!(
        storage
            .clear_history_before("2026-02-01T00:00:00Z")
            .unwrap(),
        1
    );
    assert_eq!(
        storage
            .clear_expired_notifications("2026-02-01T00:00:00Z")
            .unwrap(),
        1
    );
    assert_eq!(storage.table_row_count("history_entries").unwrap(), 1);
    assert_eq!(storage.table_row_count("notification_records").unwrap(), 0);
}

#[test]
fn storage_service_uses_runtime_lifecycle_without_claiming_other_services() {
    let database = TempDatabase::new();
    let runtime = RuntimeBuilder::new()
        .register(StorageService::new(&database.path))
        .unwrap()
        .build();
    let mut runtime = runtime;

    let started = runtime.start().expect("storage service should start");
    assert_eq!(started.phase, RuntimePhase::Partial);
    assert_eq!(
        started
            .services
            .iter()
            .find(|service| service.kind == ServiceKind::Storage)
            .map(|service| service.status),
        Some(ServiceStatus::Running)
    );
    assert_eq!(runtime.shutdown().unwrap().phase, RuntimePhase::Stopped);
}
