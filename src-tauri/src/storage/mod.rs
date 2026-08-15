//! Persistência local do Pulse.
//!
//! O storage conhece SQLite e o schema interno, mas não conhece Tauri, a UI,
//! transporte ou segredos. O único ponto de integração com o runtime é
//! [`StorageService`], que abre a conexão no `start` e a libera no `stop`.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::runtime::{RuntimeService, ServiceFailureCode, ServiceKind};

pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;
pub const DATABASE_FILE_NAME: &str = "pulse.sqlite";
const APPLICATION_ID: i64 = 0x5055_4c53;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageError {
    Io,
    Database,
    Corrupt,
    Incompatible { found: u32, supported: u32 },
    MigrationFailed { version: u32 },
    MigrationChecksumMismatch { version: u32 },
    ForeignKeyViolation,
    InvalidInput,
}

impl Display for StorageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io => formatter.write_str("local storage I/O failed"),
            Self::Database => formatter.write_str("local storage database failed"),
            Self::Corrupt => formatter.write_str("local storage is corrupt"),
            Self::Incompatible { found, supported } => write!(
                formatter,
                "local storage schema {found} is newer than supported schema {supported}"
            ),
            Self::MigrationFailed { version } => {
                write!(formatter, "local storage migration {version} failed")
            }
            Self::MigrationChecksumMismatch { version } => write!(
                formatter,
                "local storage migration {version} checksum does not match"
            ),
            Self::ForeignKeyViolation => {
                formatter.write_str("local storage foreign key check failed")
            }
            Self::InvalidInput => formatter.write_str("local storage input is invalid"),
        }
    }
}

impl Error for StorageError {}

#[derive(Clone, Copy)]
struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: [Migration; 1] = [Migration {
    version: 1,
    name: "001_initial_domain_storage",
    sql: r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY CHECK (version > 0),
  name TEXT NOT NULL,
  checksum TEXT NOT NULL CHECK (length(checksum) = 64),
  applied_at TEXT NOT NULL,
  result TEXT NOT NULL CHECK (result = 'applied')
);

CREATE TABLE IF NOT EXISTS local_identity_public (
  slot INTEGER PRIMARY KEY CHECK (slot = 1),
  format TEXT NOT NULL,
  algorithm TEXT NOT NULL,
  device_id TEXT NOT NULL UNIQUE,
  public_key TEXT NOT NULL,
  fingerprint TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS known_devices (
  device_id TEXT PRIMARY KEY,
  public_key TEXT NOT NULL,
  fingerprint TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  platform TEXT NOT NULL,
  model TEXT,
  platform_version TEXT,
  last_seen_at TEXT
);

CREATE TABLE IF NOT EXISTS trust_relationships (
  device_id TEXT PRIMARY KEY REFERENCES known_devices(device_id) ON DELETE CASCADE,
  state TEXT NOT NULL CHECK (state IN ('unpaired', 'trusted', 'revoked')),
  updated_at TEXT NOT NULL,
  decided_at TEXT,
  revoked_at TEXT,
  reason_code TEXT,
  pairing_session_id TEXT
);

CREATE TABLE IF NOT EXISTS capability_grants (
  device_id TEXT NOT NULL REFERENCES known_devices(device_id) ON DELETE CASCADE,
  capability_key TEXT NOT NULL,
  direction TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('requested', 'granted', 'denied', 'revoked')),
  requested_at TEXT,
  decided_at TEXT,
  decided_by TEXT CHECK (decided_by IS NULL OR decided_by IN ('local-user', 'peer', 'system')),
  reason_code TEXT,
  PRIMARY KEY (device_id, capability_key, direction)
);

CREATE TABLE IF NOT EXISTS revocation_blocks (
  fingerprint TEXT PRIMARY KEY,
  blocked_at TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('local-user', 'peer', 'system')),
  reason_code TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pairing_sessions (
  id TEXT PRIMARY KEY,
  initiator_device_id TEXT NOT NULL,
  candidate_id TEXT,
  target_device_id TEXT REFERENCES known_devices(device_id) ON DELETE SET NULL,
  presented_device_id TEXT,
  presented_name TEXT NOT NULL,
  presented_platform TEXT NOT NULL,
  presented_fingerprint TEXT,
  state TEXT NOT NULL CHECK (state IN ('requested', 'awaiting-confirmation', 'confirmed', 'rejected', 'expired', 'canceled', 'failed')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  resolved_at TEXT,
  failure_code TEXT
);

CREATE TABLE IF NOT EXISTS transfer_sessions (
  id TEXT PRIMARY KEY,
  source_device_id TEXT NOT NULL REFERENCES known_devices(device_id),
  destination_device_id TEXT NOT NULL REFERENCES known_devices(device_id),
  direction TEXT NOT NULL CHECK (direction IN ('outgoing', 'incoming')),
  kind TEXT NOT NULL CHECK (kind IN ('file', 'directory', 'light-content')),
  state TEXT NOT NULL CHECK (state IN ('draft', 'awaiting-approval', 'queued', 'active', 'paused', 'completed', 'failed', 'canceled')),
  attempt INTEGER NOT NULL CHECK (attempt >= 0),
  item_count INTEGER NOT NULL CHECK (item_count >= 0),
  total_bytes INTEGER NOT NULL CHECK (total_bytes >= 0),
  completed_bytes INTEGER NOT NULL CHECK (completed_bytes >= 0),
  error_code TEXT,
  error_retryable INTEGER CHECK (error_retryable IS NULL OR error_retryable IN (0, 1)),
  integrity_verified INTEGER CHECK (integrity_verified IS NULL OR integrity_verified IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT
);

CREATE TABLE IF NOT EXISTS history_entries (
  id TEXT PRIMARY KEY,
  entry_type TEXT NOT NULL CHECK (entry_type IN ('pairing', 'trust', 'capability', 'transfer', 'clipboard', 'light-content', 'media', 'remote-command')),
  source_device_id TEXT REFERENCES known_devices(device_id) ON DELETE SET NULL,
  target_device_id TEXT REFERENCES known_devices(device_id) ON DELETE SET NULL,
  result TEXT NOT NULL CHECK (result IN ('succeeded', 'failed', 'denied', 'canceled', 'expired')),
  occurred_at TEXT NOT NULL,
  recorded_at TEXT NOT NULL,
  related_kind TEXT NOT NULL,
  related_id TEXT NOT NULL,
  reason_code TEXT
);

CREATE TABLE IF NOT EXISTS notification_records (
  id TEXT PRIMARY KEY,
  severity TEXT NOT NULL CHECK (severity IN ('info', 'success', 'warning', 'error')),
  title_key TEXT NOT NULL,
  body_key TEXT NOT NULL,
  source_event_id TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN ('queued', 'delivered', 'dismissed', 'expired', 'failed')),
  queued_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  expires_at TEXT
);

CREATE TABLE IF NOT EXISTS preferences (
  key TEXT PRIMARY KEY,
  value_text TEXT NOT NULL CHECK (length(value_text) <= 4096),
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_known_devices_last_seen ON known_devices(last_seen_at);
CREATE INDEX IF NOT EXISTS idx_history_recorded_at ON history_entries(recorded_at);
CREATE INDEX IF NOT EXISTS idx_notifications_expires_at ON notification_records(expires_at);
CREATE INDEX IF NOT EXISTS idx_transfer_sessions_updated_at ON transfer_sessions(updated_at);
"#,
}];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnownDeviceRecord {
    pub device_id: String,
    pub public_key: String,
    pub fingerprint: String,
    pub name: String,
    pub platform: String,
    pub model: Option<String>,
    pub platform_version: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustRecord {
    pub device_id: String,
    pub state: String,
    pub updated_at: String,
    pub decided_at: Option<String>,
    pub revoked_at: Option<String>,
    pub reason_code: Option<String>,
    pub pairing_session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryRecord {
    pub id: String,
    pub entry_type: String,
    pub source_device_id: Option<String>,
    pub target_device_id: Option<String>,
    pub result: String,
    pub occurred_at: String,
    pub recorded_at: String,
    pub related_kind: String,
    pub related_id: String,
    pub reason_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationRecord {
    pub id: String,
    pub severity: String,
    pub title_key: String,
    pub body_key: String,
    pub source_event_id: String,
    pub state: String,
    pub queued_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreferenceKey {
    HistoryRetentionDays,
    HistoryMaxEntries,
    NotificationsEnabled,
}

impl PreferenceKey {
    const fn as_str(self) -> &'static str {
        match self {
            Self::HistoryRetentionDays => "history.retention-days",
            Self::HistoryMaxEntries => "history.max-entries",
            Self::NotificationsEnabled => "notifications.enabled",
        }
    }
}

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|_| StorageError::Io)?;
            }
        }

        let existing_nonempty = fs::metadata(path)
            .map(|metadata| metadata.len() > 0)
            .unwrap_or(false);
        let connection = Connection::open(path).map_err(map_sqlite_error)?;
        Self::open_connection(connection, true, existing_nonempty)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory().map_err(map_sqlite_error)?;
        Self::open_connection(connection, false, false)
    }

    pub fn schema_version(&self) -> Result<u32, StorageError> {
        read_schema_version(&self.connection)
    }

    pub fn application_id(&self) -> Result<i64, StorageError> {
        pragma_value(&self.connection, "application_id")
    }

    pub fn save_known_device(&mut self, record: &KnownDeviceRecord) -> Result<(), StorageError> {
        validate_required(&record.device_id)?;
        validate_required(&record.public_key)?;
        validate_required(&record.fingerprint)?;
        validate_required(&record.name)?;
        validate_required(&record.platform)?;

        self.connection
            .execute(
                "INSERT INTO known_devices
                 (device_id, public_key, fingerprint, name, platform, model, platform_version, last_seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(device_id) DO UPDATE SET
                   public_key = excluded.public_key,
                   fingerprint = excluded.fingerprint,
                   name = excluded.name,
                   platform = excluded.platform,
                   model = excluded.model,
                   platform_version = excluded.platform_version,
                   last_seen_at = excluded.last_seen_at",
                params![
                    record.device_id,
                    record.public_key,
                    record.fingerprint,
                    record.name,
                    record.platform,
                    record.model,
                    record.platform_version,
                    record.last_seen_at,
                ],
            )
            .map_err(map_sqlite_error)?;
        Ok(())
    }

    pub fn known_device(&self, device_id: &str) -> Result<Option<KnownDeviceRecord>, StorageError> {
        validate_required(device_id)?;
        self.connection
            .query_row(
                "SELECT device_id, public_key, fingerprint, name, platform, model, platform_version, last_seen_at
                 FROM known_devices WHERE device_id = ?1",
                params![device_id],
                |row| {
                    Ok(KnownDeviceRecord {
                        device_id: row.get(0)?,
                        public_key: row.get(1)?,
                        fingerprint: row.get(2)?,
                        name: row.get(3)?,
                        platform: row.get(4)?,
                        model: row.get(5)?,
                        platform_version: row.get(6)?,
                        last_seen_at: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(map_sqlite_error)
    }

    pub fn save_trust(&mut self, record: &TrustRecord) -> Result<(), StorageError> {
        validate_required(&record.device_id)?;
        validate_state(record.state.as_str(), &["unpaired", "trusted", "revoked"])?;
        self.connection
            .execute(
                "INSERT INTO trust_relationships
                 (device_id, state, updated_at, decided_at, revoked_at, reason_code, pairing_session_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(device_id) DO UPDATE SET
                   state = excluded.state,
                   updated_at = excluded.updated_at,
                   decided_at = excluded.decided_at,
                   revoked_at = excluded.revoked_at,
                   reason_code = excluded.reason_code,
                   pairing_session_id = excluded.pairing_session_id",
                params![
                    record.device_id,
                    record.state,
                    record.updated_at,
                    record.decided_at,
                    record.revoked_at,
                    record.reason_code,
                    record.pairing_session_id,
                ],
            )
            .map_err(map_sqlite_error)
            .map(|_| ())
    }

    pub fn set_preference(
        &mut self,
        key: PreferenceKey,
        value: &str,
        updated_at: &str,
    ) -> Result<(), StorageError> {
        validate_preference_value(key, value)?;
        validate_required(updated_at)?;
        if value.len() > 4096 {
            return Err(StorageError::InvalidInput);
        }
        self.connection
            .execute(
                "INSERT INTO preferences (key, value_text, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value_text = excluded.value_text, updated_at = excluded.updated_at",
                params![key.as_str(), value, updated_at],
            )
            .map_err(map_sqlite_error)
            .map(|_| ())
    }

    pub fn save_history_metadata(&mut self, record: &HistoryRecord) -> Result<(), StorageError> {
        validate_required(&record.id)?;
        validate_state(
            record.entry_type.as_str(),
            &[
                "pairing",
                "trust",
                "capability",
                "transfer",
                "clipboard",
                "light-content",
                "media",
                "remote-command",
            ],
        )?;
        validate_state(
            record.result.as_str(),
            &["succeeded", "failed", "denied", "canceled", "expired"],
        )?;
        validate_required(&record.occurred_at)?;
        validate_required(&record.recorded_at)?;
        validate_required(&record.related_kind)?;
        validate_required(&record.related_id)?;

        self.connection
            .execute(
                "INSERT INTO history_entries
                 (id, entry_type, source_device_id, target_device_id, result, occurred_at, recorded_at,
                  related_kind, related_id, reason_code)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                   entry_type = excluded.entry_type,
                   source_device_id = excluded.source_device_id,
                   target_device_id = excluded.target_device_id,
                   result = excluded.result,
                   occurred_at = excluded.occurred_at,
                   recorded_at = excluded.recorded_at,
                   related_kind = excluded.related_kind,
                   related_id = excluded.related_id,
                   reason_code = excluded.reason_code",
                params![
                    record.id,
                    record.entry_type,
                    record.source_device_id,
                    record.target_device_id,
                    record.result,
                    record.occurred_at,
                    record.recorded_at,
                    record.related_kind,
                    record.related_id,
                    record.reason_code,
                ],
            )
            .map_err(map_sqlite_error)
            .map(|_| ())
    }

    pub fn save_notification_metadata(
        &mut self,
        record: &NotificationRecord,
    ) -> Result<(), StorageError> {
        validate_required(&record.id)?;
        validate_state(&record.severity, &["info", "success", "warning", "error"])?;
        validate_state(
            &record.state,
            &["queued", "delivered", "dismissed", "expired", "failed"],
        )?;
        validate_required(&record.title_key)?;
        validate_required(&record.body_key)?;
        validate_required(&record.source_event_id)?;
        validate_required(&record.queued_at)?;
        validate_required(&record.updated_at)?;

        self.connection
            .execute(
                "INSERT INTO notification_records
                 (id, severity, title_key, body_key, source_event_id, state, queued_at, updated_at, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                   severity = excluded.severity,
                   title_key = excluded.title_key,
                   body_key = excluded.body_key,
                   source_event_id = excluded.source_event_id,
                   state = excluded.state,
                   queued_at = excluded.queued_at,
                   updated_at = excluded.updated_at,
                   expires_at = excluded.expires_at",
                params![
                    record.id,
                    record.severity,
                    record.title_key,
                    record.body_key,
                    record.source_event_id,
                    record.state,
                    record.queued_at,
                    record.updated_at,
                    record.expires_at,
                ],
            )
            .map_err(map_sqlite_error)
            .map(|_| ())
    }

    pub fn preference(&self, key: PreferenceKey) -> Result<Option<String>, StorageError> {
        self.connection
            .query_row(
                "SELECT value_text FROM preferences WHERE key = ?1",
                params![key.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(map_sqlite_error)
    }

    pub fn clear_history_before(&mut self, cutoff: &str) -> Result<usize, StorageError> {
        validate_required(cutoff)?;
        self.connection
            .execute(
                "DELETE FROM history_entries WHERE recorded_at < ?1",
                params![cutoff],
            )
            .map_err(map_sqlite_error)
    }

    pub fn clear_expired_notifications(&mut self, cutoff: &str) -> Result<usize, StorageError> {
        validate_required(cutoff)?;
        self.connection
            .execute(
                "DELETE FROM notification_records
                 WHERE expires_at IS NOT NULL AND expires_at < ?1
                   AND state IN ('dismissed', 'expired', 'failed')",
                params![cutoff],
            )
            .map_err(map_sqlite_error)
    }

    pub fn table_row_count(&self, table: &str) -> Result<u64, StorageError> {
        let query = match table {
            "known_devices"
            | "trust_relationships"
            | "capability_grants"
            | "history_entries"
            | "notification_records"
            | "preferences" => format!("SELECT COUNT(*) FROM {table}"),
            _ => return Err(StorageError::InvalidInput),
        };
        self.connection
            .query_row(&query, [], |row| row.get::<_, i64>(0))
            .map(|count| count as u64)
            .map_err(map_sqlite_error)
    }

    fn open_connection(
        mut connection: Connection,
        persistent: bool,
        existing_nonempty: bool,
    ) -> Result<Self, StorageError> {
        configure_connection(&connection, persistent)?;
        let application_id = validate_application_id(&connection)?;

        let schema_table_exists = table_exists(&connection, "schema_migrations")?;
        let current_version = read_schema_version(&connection)?;
        let user_version = pragma_value(&connection, "user_version")? as u32;
        if current_version > SUPPORTED_SCHEMA_VERSION {
            return Err(StorageError::Incompatible {
                found: current_version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        }
        if user_version > SUPPORTED_SCHEMA_VERSION {
            return Err(StorageError::Incompatible {
                found: user_version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        }
        if user_version != 0 && user_version != current_version {
            return Err(StorageError::Corrupt);
        }
        if existing_nonempty && application_id == 0 && !schema_table_exists {
            return Err(StorageError::Corrupt);
        }
        if schema_table_exists {
            validate_applied_migrations(&connection, current_version)?;
        }

        apply_pending_migrations(&mut connection, current_version, &MIGRATIONS)?;
        connection
            .pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(map_sqlite_error)?;
        connection
            .pragma_update(None, "user_version", SUPPORTED_SCHEMA_VERSION)
            .map_err(map_sqlite_error)?;
        validate_application_id(&connection)?;
        quick_check(&connection)?;
        foreign_key_check(&connection)?;

        Ok(Self { connection })
    }
}

pub struct StorageService {
    path: PathBuf,
    storage: Option<Storage>,
}

impl StorageService {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            storage: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.storage.is_some()
    }
}

impl RuntimeService for StorageService {
    fn kind(&self) -> ServiceKind {
        ServiceKind::Storage
    }

    fn start(&mut self) -> Result<(), ServiceFailureCode> {
        if self.storage.is_some() {
            return Err(ServiceFailureCode::InternalState);
        }
        self.storage =
            Some(Storage::open(&self.path).map_err(|_| ServiceFailureCode::InitializationFailed)?);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceFailureCode> {
        self.storage = None;
        Ok(())
    }
}

fn configure_connection(connection: &Connection, persistent: bool) -> Result<(), StorageError> {
    connection
        .busy_timeout(BUSY_TIMEOUT)
        .map_err(map_sqlite_error)?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(map_sqlite_error)?;

    if persistent {
        let journal_mode: String = connection
            .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))
            .map_err(map_sqlite_error)?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(StorageError::Database);
        }
    } else {
        connection
            .pragma_update(None, "journal_mode", "MEMORY")
            .map_err(map_sqlite_error)?;
    }

    connection
        .pragma_update(None, "synchronous", "FULL")
        .map_err(map_sqlite_error)?;
    let foreign_keys = pragma_value(connection, "foreign_keys")?;
    if foreign_keys != 1 {
        return Err(StorageError::Database);
    }
    Ok(())
}

fn validate_application_id(connection: &Connection) -> Result<i64, StorageError> {
    let application_id = pragma_value(connection, "application_id")?;
    if application_id != 0 && application_id != APPLICATION_ID {
        return Err(StorageError::Incompatible {
            found: application_id as u32,
            supported: APPLICATION_ID as u32,
        });
    }
    Ok(application_id)
}

fn apply_pending_migrations(
    connection: &mut Connection,
    current_version: u32,
    migrations: &[Migration],
) -> Result<(), StorageError> {
    let mut expected_version = current_version.saturating_add(1);
    for migration in migrations
        .iter()
        .filter(|migration| migration.version > current_version)
    {
        if migration.version != expected_version {
            return Err(StorageError::MigrationFailed {
                version: migration.version,
            });
        }

        let checksum = migration_checksum(migration.sql);
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| StorageError::MigrationFailed {
                version: migration.version,
            })?;
        if transaction.execute_batch(migration.sql).is_err() {
            return Err(StorageError::MigrationFailed {
                version: migration.version,
            });
        }
        if transaction
            .execute(
                "INSERT INTO schema_migrations (version, name, checksum, applied_at, result)
                 VALUES (?1, ?2, ?3, ?4, 'applied')",
                params![
                    migration.version,
                    migration.name,
                    checksum,
                    utc_epoch_seconds()
                ],
            )
            .is_err()
        {
            return Err(StorageError::MigrationFailed {
                version: migration.version,
            });
        }
        if transaction.commit().is_err() {
            return Err(StorageError::MigrationFailed {
                version: migration.version,
            });
        }
        expected_version = expected_version.saturating_add(1);
    }
    Ok(())
}

fn validate_applied_migrations(
    connection: &Connection,
    current_version: u32,
) -> Result<(), StorageError> {
    let mut statement = connection
        .prepare("SELECT version, name, checksum, result FROM schema_migrations ORDER BY version")
        .map_err(map_sqlite_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(map_sqlite_error)?;

    for row in rows {
        let (version, name, checksum, result) = row.map_err(map_sqlite_error)?;
        let Some(migration) = MIGRATIONS
            .iter()
            .find(|migration| migration.version == version)
        else {
            return Err(StorageError::Incompatible {
                found: version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        };
        if migration.name != name || migration_checksum(migration.sql) != checksum {
            return Err(StorageError::MigrationChecksumMismatch { version });
        }
        if result != "applied" {
            return Err(StorageError::MigrationFailed { version });
        }
    }

    if current_version > 0 && current_version != SUPPORTED_SCHEMA_VERSION {
        return Err(StorageError::Incompatible {
            found: current_version,
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn read_schema_version(connection: &Connection) -> Result<u32, StorageError> {
    if !table_exists(connection, "schema_migrations")? {
        return Ok(0);
    }
    connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .map_err(map_sqlite_error)
}

fn table_exists(connection: &Connection, name: &str) -> Result<bool, StorageError> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![name],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|value| value.is_some())
        .map_err(map_sqlite_error)
}

fn pragma_value(connection: &Connection, name: &str) -> Result<i64, StorageError> {
    connection
        .query_row(&format!("PRAGMA {name}"), [], |row| row.get(0))
        .map_err(map_sqlite_error)
}

fn quick_check(connection: &Connection) -> Result<(), StorageError> {
    let result: String = connection
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(map_sqlite_error)?;
    if result.eq_ignore_ascii_case("ok") {
        Ok(())
    } else {
        Err(StorageError::Corrupt)
    }
}

fn foreign_key_check(connection: &Connection) -> Result<(), StorageError> {
    let mut statement = connection
        .prepare("PRAGMA foreign_key_check")
        .map_err(map_sqlite_error)?;
    let mut rows = statement.query([]).map_err(map_sqlite_error)?;
    if rows.next().map_err(map_sqlite_error)?.is_some() {
        Err(StorageError::ForeignKeyViolation)
    } else {
        Ok(())
    }
}

fn validate_required(value: &str) -> Result<(), StorageError> {
    if value.is_empty() || value.contains('\0') {
        Err(StorageError::InvalidInput)
    } else {
        Ok(())
    }
}

fn validate_state(value: &str, allowed: &[&str]) -> Result<(), StorageError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(StorageError::InvalidInput)
    }
}

fn validate_preference_value(key: PreferenceKey, value: &str) -> Result<(), StorageError> {
    validate_required(value)?;
    let valid = match key {
        PreferenceKey::HistoryRetentionDays => value
            .parse::<u32>()
            .map(|days| (1..=3650).contains(&days))
            .unwrap_or(false),
        PreferenceKey::HistoryMaxEntries => value
            .parse::<u32>()
            .map(|entries| (1..=100_000).contains(&entries))
            .unwrap_or(false),
        PreferenceKey::NotificationsEnabled => matches!(value, "true" | "false"),
    };
    if valid {
        Ok(())
    } else {
        Err(StorageError::InvalidInput)
    }
}

fn migration_checksum(sql: &str) -> String {
    let digest = Sha256::digest(sql.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn utc_epoch_seconds() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn map_sqlite_error(error: rusqlite::Error) -> StorageError {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("not a database") || message.contains("database disk image is malformed") {
        StorageError::Corrupt
    } else if message.contains("foreign key") {
        StorageError::ForeignKeyViolation
    } else {
        StorageError::Database
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_checksum_is_stable_and_hex_encoded() {
        let checksum = migration_checksum("pulse");
        assert_eq!(checksum.len(), 64);
        assert!(checksum
            .chars()
            .all(|character| character.is_ascii_hexdigit()));
        assert_eq!(checksum, migration_checksum("pulse"));
    }

    #[test]
    fn memory_storage_applies_schema_and_pragmas() {
        let storage = Storage::open_in_memory().expect("memory storage should open");
        assert_eq!(storage.schema_version().unwrap(), SUPPORTED_SCHEMA_VERSION);
        assert_eq!(storage.application_id().unwrap(), APPLICATION_ID);
    }

    #[test]
    fn failed_migration_rolls_back_its_partial_ddl() {
        let mut connection = Connection::open_in_memory().expect("memory database should open");
        configure_connection(&connection, false).expect("test connection should configure");
        let migrations = [
            Migration {
                version: 1,
                name: "one",
                sql: "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, name TEXT NOT NULL, checksum TEXT NOT NULL, applied_at TEXT NOT NULL, result TEXT NOT NULL); CREATE TABLE preserved (id INTEGER PRIMARY KEY);",
            },
            Migration {
                version: 2,
                name: "two",
                sql: "CREATE TABLE rolled_back (id INTEGER PRIMARY KEY); THIS IS NOT SQL;",
            },
        ];

        assert_eq!(
            apply_pending_migrations(&mut connection, 0, &migrations),
            Err(StorageError::MigrationFailed { version: 2 })
        );
        assert!(table_exists(&connection, "preserved").unwrap());
        assert!(!table_exists(&connection, "rolled_back").unwrap());
        assert_eq!(read_schema_version(&connection).unwrap(), 1);
    }

    #[test]
    fn schema_does_not_declare_forbidden_payload_columns() {
        let storage = Storage::open_in_memory().expect("memory storage should open");
        let schema: String = storage
            .connection
            .query_row(
                "SELECT group_concat(sql, ' ') FROM sqlite_master WHERE type = 'table'",
                [],
                |row| row.get(0),
            )
            .expect("schema SQL should be queryable");
        let schema = schema.to_ascii_lowercase();
        for forbidden in [
            "private_key",
            "payload",
            "local_path",
            "clipboard_value",
            "session_token",
        ] {
            assert!(
                !schema.contains(forbidden),
                "schema contains forbidden field {forbidden}"
            );
        }
    }
}
