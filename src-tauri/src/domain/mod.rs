//! Modelos puros do domínio do Pulse.
//!
//! Este módulo não conhece Tauri, transporte, persistência ou a bridge. Os
//! tipos espelham o contrato em `src/types/index.ts`; serialização e comandos
//! ficam para as tasks de bridge.

pub const DOMAIN_MODEL_VERSION: u16 = 1;

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name(pub String);
    };
}

opaque_id!(DeviceId);
opaque_id!(CandidateId);
opaque_id!(PairingSessionId);
opaque_id!(TransferSessionId);
opaque_id!(TransferItemId);
opaque_id!(HistoryEntryId);
opaque_id!(NotificationId);
opaque_id!(RemoteCommandId);
opaque_id!(DomainEventId);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UtcTimestamp(pub String);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LocalPath(pub String);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DurationMs(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByteCount(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DevicePlatform {
    Linux,
    Android,
    Ios,
    Windows,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryCandidateState {
    Discovered,
    Expired,
}

impl DiscoveryCandidateState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Discovered, Self::Discovered | Self::Expired)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresenceState {
    Unknown,
    Online,
    Stale,
    Offline,
}

impl PresenceState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Unknown => matches!(
                next,
                Self::Unknown | Self::Online | Self::Stale | Self::Offline
            ),
            Self::Online => matches!(next, Self::Online | Self::Stale | Self::Offline),
            Self::Stale => matches!(next, Self::Stale | Self::Online | Self::Offline),
            Self::Offline => matches!(next, Self::Offline | Self::Online | Self::Unknown),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairingState {
    Requested,
    AwaitingConfirmation,
    Confirmed,
    Rejected,
    Expired,
    Canceled,
    Failed,
}

impl PairingState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Requested => matches!(
                next,
                Self::Requested
                    | Self::AwaitingConfirmation
                    | Self::Rejected
                    | Self::Expired
                    | Self::Canceled
                    | Self::Failed
            ),
            Self::AwaitingConfirmation => matches!(
                next,
                Self::AwaitingConfirmation
                    | Self::Confirmed
                    | Self::Rejected
                    | Self::Expired
                    | Self::Canceled
                    | Self::Failed
            ),
            Self::Confirmed | Self::Rejected | Self::Expired | Self::Canceled | Self::Failed => {
                false
            }
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Confirmed | Self::Rejected | Self::Expired | Self::Canceled | Self::Failed
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustState {
    Unpaired,
    Trusted,
    Revoked,
}

impl TrustState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Unpaired => matches!(next, Self::Unpaired | Self::Trusted),
            Self::Trusted => matches!(next, Self::Trusted | Self::Revoked),
            Self::Revoked => matches!(next, Self::Revoked | Self::Unpaired),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityKey {
    FilesSend,
    FilesReceive,
    ClipboardRead,
    ClipboardWrite,
    TextSend,
    LinksSend,
    MediaRead,
    MediaControl,
    NotificationsReceive,
    CommandsExecute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDirection {
    Send,
    Receive,
    Read,
    Write,
    Control,
    Execute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityGrantState {
    Requested,
    Granted,
    Denied,
    Revoked,
}

impl CapabilityGrantState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Requested => matches!(
                next,
                Self::Requested | Self::Granted | Self::Denied | Self::Revoked
            ),
            Self::Granted => matches!(next, Self::Granted | Self::Revoked),
            Self::Denied => matches!(next, Self::Denied | Self::Requested | Self::Revoked),
            Self::Revoked => matches!(next, Self::Revoked | Self::Requested),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryEndpoint {
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInfo {
    pub key: CapabilityKey,
    pub available: bool,
    pub direction: Option<CapabilityDirection>,
    pub observed_at: UtcTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryCandidate {
    pub id: CandidateId,
    pub presented_name: String,
    pub platform: DevicePlatform,
    pub endpoint: DiscoveryEndpoint,
    pub advertised_capabilities: Vec<CapabilityInfo>,
    pub state: DiscoveryCandidateState,
    pub discovered_at: UtcTimestamp,
    pub last_seen_at: UtcTimestamp,
    pub expires_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceMetadata {
    pub model: Option<String>,
    pub platform_version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub platform: DevicePlatform,
    pub metadata: Option<DeviceMetadata>,
    pub trust: TrustRelationship,
    pub capabilities: Vec<CapabilityInfo>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Presence {
    pub device_id: DeviceId,
    pub state: PresenceState,
    pub observed_at: UtcTimestamp,
    pub last_seen_at: Option<UtcTimestamp>,
    pub stale_at: Option<UtcTimestamp>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentedIdentity {
    pub device_id: Option<DeviceId>,
    pub name: String,
    pub platform: DevicePlatform,
    pub fingerprint: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingSession {
    pub id: PairingSessionId,
    pub initiator_device_id: DeviceId,
    pub candidate_id: Option<CandidateId>,
    pub target_device_id: Option<DeviceId>,
    pub presented_identity: PresentedIdentity,
    pub state: PairingState,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub expires_at: UtcTimestamp,
    pub resolved_at: Option<UtcTimestamp>,
    pub failure_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustRelationship {
    pub device_id: DeviceId,
    pub state: TrustState,
    pub updated_at: UtcTimestamp,
    pub decided_at: Option<UtcTimestamp>,
    pub revoked_at: Option<UtcTimestamp>,
    pub reason_code: Option<String>,
    pub pairing_session_id: Option<PairingSessionId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDecisionSource {
    LocalUser,
    Peer,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityGrant {
    pub device_id: DeviceId,
    pub key: CapabilityKey,
    pub direction: Option<CapabilityDirection>,
    pub state: CapabilityGrantState,
    pub requested_at: Option<UtcTimestamp>,
    pub decided_at: Option<UtcTimestamp>,
    pub decided_by: Option<CapabilityDecisionSource>,
    pub reason_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LightContent {
    Text {
        value: String,
        byte_length: ByteCount,
    },
    Link {
        url: String,
        byte_length: ByteCount,
    },
}

pub const MAX_LIGHT_CONTENT_TEXT_BYTES: u64 = 1024 * 1024;
pub const MAX_LIGHT_CONTENT_LINK_CHARACTERS: usize = 2048;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferProgress {
    Bytes {
        completed_bytes: ByteCount,
        total_bytes: ByteCount,
    },
    Items {
        completed_items: u64,
        total_items: u64,
    },
    Indeterminate {
        reason: IndeterminateProgressReason,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndeterminateProgressReason {
    UnknownSize,
    WaitingForPeer,
    NotStarted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferItem {
    File {
        id: TransferItemId,
        name: String,
        size_bytes: ByteCount,
        local_source: Option<LocalPath>,
    },
    Directory {
        id: TransferItemId,
        name: String,
        item_count: Option<u64>,
        local_source: Option<LocalPath>,
    },
    LightContent {
        id: TransferItemId,
        content: LightContent,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferErrorCode {
    ApprovalDenied,
    PeerOffline,
    CapabilityDenied,
    IntegrityFailed,
    ConflictUnresolved,
    DestinationUnavailable,
    InvalidContent,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferError {
    pub code: TransferErrorCode,
    pub retryable: bool,
    pub occurred_at: UtcTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferResult {
    pub integrity_verified: bool,
    pub completed_at: UtcTimestamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferState {
    Draft,
    AwaitingApproval,
    Queued,
    Active,
    Paused,
    Completed,
    Failed,
    Canceled,
}

impl TransferState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Draft => matches!(next, Self::Draft | Self::AwaitingApproval | Self::Canceled),
            Self::AwaitingApproval => matches!(
                next,
                Self::AwaitingApproval | Self::Queued | Self::Canceled | Self::Failed
            ),
            Self::Queued => matches!(
                next,
                Self::Queued | Self::Active | Self::Canceled | Self::Failed
            ),
            Self::Active => matches!(
                next,
                Self::Active | Self::Paused | Self::Completed | Self::Failed | Self::Canceled
            ),
            Self::Paused => matches!(
                next,
                Self::Paused | Self::Active | Self::Canceled | Self::Failed
            ),
            Self::Completed | Self::Canceled => false,
            Self::Failed => matches!(next, Self::Failed | Self::Queued | Self::Canceled),
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Canceled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferKind {
    File,
    Directory,
    LightContent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferDirection {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DestinationConflictPolicy {
    Ask,
    Replace,
    Rename,
    Skip,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferSession {
    pub id: TransferSessionId,
    pub source_device_id: DeviceId,
    pub destination_device_id: DeviceId,
    pub direction: TransferDirection,
    pub kind: TransferKind,
    pub items: Vec<TransferItem>,
    pub state: TransferState,
    pub progress: TransferProgress,
    pub attempt: u32,
    pub destination_policy: DestinationConflictPolicy,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub queued_at: Option<UtcTimestamp>,
    pub started_at: Option<UtcTimestamp>,
    pub completed_at: Option<UtcTimestamp>,
    pub error: Option<TransferError>,
    pub result: Option<TransferResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryEntryType {
    Pairing,
    Trust,
    Capability,
    Transfer,
    Clipboard,
    LightContent,
    Media,
    RemoteCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryResult {
    Succeeded,
    Failed,
    Denied,
    Canceled,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryRelatedEntity {
    pub kind: HistoryEntryType,
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryEntry {
    pub id: HistoryEntryId,
    pub entry_type: HistoryEntryType,
    pub source_device_id: Option<DeviceId>,
    pub target_device_id: Option<DeviceId>,
    pub result: HistoryResult,
    pub occurred_at: UtcTimestamp,
    pub recorded_at: UtcTimestamp,
    pub related_entity: HistoryRelatedEntity,
    pub reason_code: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationSeverity {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationState {
    Queued,
    Delivered,
    Dismissed,
    Expired,
    Failed,
}

impl NotificationState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Queued => matches!(
                next,
                Self::Queued | Self::Delivered | Self::Expired | Self::Failed
            ),
            Self::Delivered => matches!(next, Self::Delivered | Self::Dismissed | Self::Expired),
            Self::Dismissed | Self::Expired => false,
            Self::Failed => matches!(next, Self::Failed | Self::Queued),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationContent {
    pub title_key: String,
    pub body_key: String,
    pub parameters: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalNotification {
    pub id: NotificationId,
    pub severity: NotificationSeverity,
    pub content: NotificationContent,
    pub source_event_id: DomainEventId,
    pub state: NotificationState,
    pub queued_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub expires_at: Option<UtcTimestamp>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaAvailability {
    Unknown,
    Available,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackState {
    Unknown,
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaItem {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<DurationMs>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaState {
    pub device_id: DeviceId,
    pub availability: MediaAvailability,
    pub playback: PlaybackState,
    pub item: Option<MediaItem>,
    pub observed_at: UtcTimestamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteCommandAction {
    DevicePing,
    MediaPlay,
    MediaPause,
    MediaStop,
    MediaNext,
    MediaPrevious,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteCommandDefinition {
    DevicePing,
    MediaPlay,
    MediaPause,
    MediaStop,
    MediaNext,
    MediaPrevious,
}

impl RemoteCommandDefinition {
    pub const fn action(self) -> RemoteCommandAction {
        match self {
            Self::DevicePing => RemoteCommandAction::DevicePing,
            Self::MediaPlay => RemoteCommandAction::MediaPlay,
            Self::MediaPause => RemoteCommandAction::MediaPause,
            Self::MediaStop => RemoteCommandAction::MediaStop,
            Self::MediaNext => RemoteCommandAction::MediaNext,
            Self::MediaPrevious => RemoteCommandAction::MediaPrevious,
        }
    }

    pub const fn required_capability(self) -> CapabilityKey {
        match self {
            Self::DevicePing => CapabilityKey::CommandsExecute,
            Self::MediaPlay
            | Self::MediaPause
            | Self::MediaStop
            | Self::MediaNext
            | Self::MediaPrevious => CapabilityKey::MediaControl,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteCommandState {
    Requested,
    AwaitingApproval,
    Running,
    Succeeded,
    Rejected,
    Failed,
    Canceled,
    Expired,
}

impl RemoteCommandState {
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Requested => matches!(
                next,
                Self::Requested
                    | Self::AwaitingApproval
                    | Self::Running
                    | Self::Rejected
                    | Self::Canceled
                    | Self::Expired
                    | Self::Failed
            ),
            Self::AwaitingApproval => matches!(
                next,
                Self::AwaitingApproval
                    | Self::Running
                    | Self::Rejected
                    | Self::Canceled
                    | Self::Expired
                    | Self::Failed
            ),
            Self::Running => matches!(
                next,
                Self::Running
                    | Self::Succeeded
                    | Self::Rejected
                    | Self::Failed
                    | Self::Canceled
                    | Self::Expired
            ),
            Self::Succeeded | Self::Rejected | Self::Failed | Self::Canceled | Self::Expired => {
                false
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteCommandOutcome {
    Confirmed,
    Rejected,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteCommandResult {
    pub outcome: RemoteCommandOutcome,
    pub completed_at: UtcTimestamp,
    pub reason_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteCommand {
    pub id: RemoteCommandId,
    pub source_device_id: DeviceId,
    pub target_device_id: DeviceId,
    pub definition: RemoteCommandDefinition,
    pub state: RemoteCommandState,
    pub requested_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub resolved_at: Option<UtcTimestamp>,
    pub result: Option<RemoteCommandResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardOrigin {
    Local,
    Remote,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardState {
    pub device_id: DeviceId,
    pub content: Option<LightContent>,
    pub origin: ClipboardOrigin,
    pub observed_at: UtcTimestamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainEntityKind {
    Candidate,
    Device,
    Presence,
    Pairing,
    Trust,
    Capability,
    Transfer,
    Clipboard,
    History,
    Notification,
    Media,
    RemoteCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainEventType {
    CandidateDiscovered,
    CandidateExpired,
    PresenceUpdated,
    PairingRequested,
    PairingConfirmed,
    PairingRejected,
    PairingExpired,
    PairingCanceled,
    TrustGranted,
    TrustRevoked,
    CapabilityRequested,
    CapabilityGranted,
    CapabilityDenied,
    CapabilityRevoked,
    TransferQueued,
    TransferStarted,
    TransferPaused,
    TransferResumed,
    TransferCompleted,
    TransferFailed,
    TransferCanceled,
    LightContentCompleted,
    ClipboardUpdated,
    MediaUpdated,
    RemoteCommandCompleted,
    HistoryCreated,
    NotificationUpdated,
}

pub type DomainEventPayload = Vec<(String, String)>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainEvent {
    pub id: DomainEventId,
    pub event_type: DomainEventType,
    pub entity: (DomainEntityKind, String),
    pub source_device_id: Option<DeviceId>,
    pub occurred_at: UtcTimestamp,
    pub model_version: u16,
    pub payload: Option<DomainEventPayload>,
}
