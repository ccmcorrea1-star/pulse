/**
 * Tipos compartilhados do domínio do Pulse.
 *
 * Estes tipos descrevem estado e relações do produto. Eles não conhecem Vue,
 * Pinia, Tauri, transporte, persistência ou copy localizado.
 */

type Brand<Value, Name extends string> = Value & { readonly __brand: Name };

export type OpaqueId<Name extends string> = Brand<string, `${Name}Id`>;
export type UtcTimestamp = Brand<string, "UtcTimestamp">;
export type DurationMs = Brand<number, "DurationMs">;
export type ByteCount = Brand<number, "ByteCount">;
export type LocalPath = Brand<string, "LocalPath">;

export type DeviceId = OpaqueId<"Device">;
export type CandidateId = OpaqueId<"Candidate">;
export type PairingSessionId = OpaqueId<"PairingSession">;
export type TransferSessionId = OpaqueId<"TransferSession">;
export type TransferItemId = OpaqueId<"TransferItem">;
export type HistoryEntryId = OpaqueId<"HistoryEntry">;
export type NotificationId = OpaqueId<"Notification">;
export type RemoteCommandId = OpaqueId<"RemoteCommand">;
export type DomainEventId = OpaqueId<"DomainEvent">;

export const DOMAIN_MODEL_VERSION = 1 as const;

export type DevicePlatform = "linux" | "android" | "ios" | "windows";

export type DiscoveryCandidateState = "discovered" | "expired";
export type PresenceState = "unknown" | "online" | "stale" | "offline";
export type PairingState =
  | "requested"
  | "awaiting-confirmation"
  | "confirmed"
  | "rejected"
  | "expired"
  | "canceled"
  | "failed";
export type TrustState = "unpaired" | "trusted" | "revoked";

export type CapabilityKey =
  | "files.send"
  | "files.receive"
  | "clipboard.read"
  | "clipboard.write"
  | "text.send"
  | "links.send"
  | "media.read"
  | "media.control"
  | "notifications.receive"
  | "commands.execute";

export type CapabilityDirection = "send" | "receive" | "read" | "write" | "control" | "execute";
export type CapabilityGrantState = "requested" | "granted" | "denied" | "revoked";

export type TransferState =
  | "draft"
  | "awaiting-approval"
  | "queued"
  | "active"
  | "paused"
  | "completed"
  | "failed"
  | "canceled";
export type TransferKind = "file" | "directory" | "light-content";
export type TransferDirection = "outgoing" | "incoming";
export type TransferProgressMode = "bytes" | "items" | "indeterminate";
export type DestinationConflictPolicy = "ask" | "replace" | "rename" | "skip";

export type HistoryResult = "succeeded" | "failed" | "denied" | "canceled" | "expired";
export type NotificationSeverity = "info" | "success" | "warning" | "error";
export type NotificationState = "queued" | "delivered" | "dismissed" | "expired" | "failed";
export type MediaAvailability = "unknown" | "available" | "unavailable";
export type PlaybackState = "unknown" | "playing" | "paused" | "stopped";
export type RemoteCommandState =
  | "requested"
  | "awaiting-approval"
  | "running"
  | "succeeded"
  | "rejected"
  | "failed"
  | "canceled"
  | "expired";

export interface DiscoveryEndpoint {
  /** Endereço efêmero; o formato concreto depende da decisão de transporte. */
  value: string;
}

export interface CapabilityInfo {
  key: CapabilityKey;
  available: boolean;
  direction?: CapabilityDirection;
  observedAt: UtcTimestamp;
}

export interface DiscoveryCandidate {
  id: CandidateId;
  presentedName: string;
  platform: DevicePlatform;
  endpoint: DiscoveryEndpoint;
  advertisedCapabilities: CapabilityInfo[];
  state: DiscoveryCandidateState;
  discoveredAt: UtcTimestamp;
  lastSeenAt: UtcTimestamp;
  expiresAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
}

export interface DeviceMetadata {
  model?: string;
  platformVersion?: string;
}

export interface Device {
  id: DeviceId;
  name: string;
  platform: DevicePlatform;
  metadata?: DeviceMetadata;
  trust: TrustRelationship;
  capabilities: CapabilityInfo[];
}

export interface Presence {
  deviceId: DeviceId;
  state: PresenceState;
  observedAt: UtcTimestamp;
  lastSeenAt?: UtcTimestamp;
  staleAt?: UtcTimestamp;
}

export interface PresentedIdentity {
  deviceId?: DeviceId;
  name: string;
  platform: DevicePlatform;
  fingerprint?: string;
}

export interface PairingSession {
  id: PairingSessionId;
  initiatorDeviceId: DeviceId;
  candidateId?: CandidateId;
  targetDeviceId?: DeviceId;
  presentedIdentity: PresentedIdentity;
  state: PairingState;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
  expiresAt: UtcTimestamp;
  resolvedAt?: UtcTimestamp;
  failureCode?: string;
}

export interface TrustRelationship {
  deviceId: DeviceId;
  state: TrustState;
  updatedAt: UtcTimestamp;
  decidedAt?: UtcTimestamp;
  revokedAt?: UtcTimestamp;
  reasonCode?: string;
  pairingSessionId?: PairingSessionId;
}

export interface CapabilityGrant {
  deviceId: DeviceId;
  key: CapabilityKey;
  direction?: CapabilityDirection;
  state: CapabilityGrantState;
  requestedAt?: UtcTimestamp;
  decidedAt?: UtcTimestamp;
  decidedBy?: "local-user" | "peer" | "system";
  reasonCode?: string;
}

export interface TextContent {
  kind: "text";
  value: string;
  byteLength: ByteCount;
}

export interface LinkContent {
  kind: "link";
  url: string;
  byteLength: ByteCount;
}

export type LightContent = TextContent | LinkContent;

export const LIGHT_CONTENT_LIMITS = {
  maxTextBytes: 1024 * 1024,
  maxLinkCharacters: 2048,
} as const;

export interface TransferProgressBytes {
  mode: "bytes";
  completedBytes: ByteCount;
  totalBytes: ByteCount;
}

export interface TransferProgressItems {
  mode: "items";
  completedItems: number;
  totalItems: number;
}

export interface TransferProgressIndeterminate {
  mode: "indeterminate";
  reason: "unknown-size" | "waiting-for-peer" | "not-started";
}

export type TransferProgress = TransferProgressBytes | TransferProgressItems | TransferProgressIndeterminate;

export interface FileTransferItem {
  id: TransferItemId;
  kind: "file";
  name: string;
  sizeBytes: ByteCount;
  localSource?: LocalPath;
}

export interface DirectoryTransferItem {
  id: TransferItemId;
  kind: "directory";
  name: string;
  itemCount?: number;
  localSource?: LocalPath;
}

export interface LightContentTransferItem {
  id: TransferItemId;
  kind: "light-content";
  content: LightContent;
}

export type TransferItem = FileTransferItem | DirectoryTransferItem | LightContentTransferItem;

export type TransferErrorCode =
  | "approval-denied"
  | "peer-offline"
  | "capability-denied"
  | "integrity-failed"
  | "conflict-unresolved"
  | "destination-unavailable"
  | "invalid-content"
  | "unknown";

export interface TransferError {
  code: TransferErrorCode;
  retryable: boolean;
  occurredAt: UtcTimestamp;
}

export interface TransferResult {
  integrityVerified: boolean;
  completedAt: UtcTimestamp;
}

export interface TransferSession {
  id: TransferSessionId;
  sourceDeviceId: DeviceId;
  destinationDeviceId: DeviceId;
  direction: TransferDirection;
  kind: TransferKind;
  items: TransferItem[];
  state: TransferState;
  progress: TransferProgress;
  attempt: number;
  destinationPolicy: DestinationConflictPolicy;
  createdAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
  queuedAt?: UtcTimestamp;
  startedAt?: UtcTimestamp;
  completedAt?: UtcTimestamp;
  error?: TransferError;
  result?: TransferResult;
}

export type HistoryEntryType =
  | "pairing"
  | "trust"
  | "capability"
  | "transfer"
  | "clipboard"
  | "light-content"
  | "media"
  | "remote-command";

export interface HistoryRelatedEntity {
  kind: HistoryEntryType;
  id: string;
}

export interface HistoryEntry {
  id: HistoryEntryId;
  type: HistoryEntryType;
  sourceDeviceId?: DeviceId;
  targetDeviceId?: DeviceId;
  result: HistoryResult;
  occurredAt: UtcTimestamp;
  recordedAt: UtcTimestamp;
  relatedEntity: HistoryRelatedEntity;
  reasonCode?: string;
}

export interface NotificationContent {
  titleKey: string;
  bodyKey: string;
  parameters?: Readonly<Record<string, string | number | boolean>>;
}

export interface LocalNotification {
  id: NotificationId;
  severity: NotificationSeverity;
  content: NotificationContent;
  sourceEventId: DomainEventId;
  state: NotificationState;
  queuedAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
  expiresAt?: UtcTimestamp;
}

export interface MediaItem {
  title?: string;
  artist?: string;
  album?: string;
  durationMs?: DurationMs;
}

export interface MediaState {
  deviceId: DeviceId;
  availability: MediaAvailability;
  playback: PlaybackState;
  item?: MediaItem;
  observedAt: UtcTimestamp;
}

export type ClipboardOrigin = "local" | "remote";

export interface ClipboardState {
  deviceId: DeviceId;
  content?: LightContent;
  origin: ClipboardOrigin;
  observedAt: UtcTimestamp;
}

export type RemoteCommandAction =
  | "device.ping"
  | "media.play"
  | "media.pause"
  | "media.stop"
  | "media.next"
  | "media.previous";

export type RemoteCommandDefinition =
  | { action: "device.ping"; requiredCapability: "commands.execute"; parameters: Record<string, never> }
  | { action: "media.play"; requiredCapability: "media.control"; parameters: Record<string, never> }
  | { action: "media.pause"; requiredCapability: "media.control"; parameters: Record<string, never> }
  | { action: "media.stop"; requiredCapability: "media.control"; parameters: Record<string, never> }
  | { action: "media.next"; requiredCapability: "media.control"; parameters: Record<string, never> }
  | { action: "media.previous"; requiredCapability: "media.control"; parameters: Record<string, never> };

export type RemoteCommandResult =
  | { outcome: "confirmed"; completedAt: UtcTimestamp }
  | { outcome: "rejected" | "failed"; completedAt: UtcTimestamp; reasonCode: string };

export type RemoteCommand = RemoteCommandDefinition & {
  id: RemoteCommandId;
  sourceDeviceId: DeviceId;
  targetDeviceId: DeviceId;
  state: RemoteCommandState;
  requestedAt: UtcTimestamp;
  updatedAt: UtcTimestamp;
  resolvedAt?: UtcTimestamp;
  result?: RemoteCommandResult;
};

export type DomainEntityKind =
  | "candidate"
  | "device"
  | "presence"
  | "pairing"
  | "trust"
  | "capability"
  | "transfer"
  | "clipboard"
  | "history"
  | "notification"
  | "media"
  | "remote-command";

export type DomainEventType =
  | "candidate.discovered"
  | "candidate.expired"
  | "presence.updated"
  | "pairing.requested"
  | "pairing.confirmed"
  | "pairing.rejected"
  | "pairing.expired"
  | "pairing.canceled"
  | "trust.granted"
  | "trust.revoked"
  | "capability.requested"
  | "capability.granted"
  | "capability.denied"
  | "capability.revoked"
  | "transfer.queued"
  | "transfer.started"
  | "transfer.paused"
  | "transfer.resumed"
  | "transfer.completed"
  | "transfer.failed"
  | "transfer.canceled"
  | "light-content.completed"
  | "clipboard.updated"
  | "media.updated"
  | "remote-command.completed"
  | "history.created"
  | "notification.updated";

export interface DomainEvent {
  id: DomainEventId;
  type: DomainEventType;
  entity: {
    kind: DomainEntityKind;
    id: string;
  };
  sourceDeviceId?: DeviceId;
  occurredAt: UtcTimestamp;
  modelVersion: typeof DOMAIN_MODEL_VERSION;
  payload?: Readonly<Record<string, unknown>>;
}

type StateTransitionMap<State extends string> = {
  readonly [CurrentState in State]: readonly State[];
};

export const DISCOVERY_CANDIDATE_TRANSITIONS = {
  discovered: ["discovered", "expired"],
  expired: [],
} as const satisfies StateTransitionMap<DiscoveryCandidateState>;

export const PRESENCE_TRANSITIONS = {
  unknown: ["unknown", "online", "stale", "offline"],
  online: ["online", "stale", "offline"],
  stale: ["stale", "online", "offline"],
  offline: ["offline", "online", "unknown"],
} as const satisfies StateTransitionMap<PresenceState>;

export const PAIRING_TRANSITIONS = {
  requested: ["requested", "awaiting-confirmation", "rejected", "expired", "canceled", "failed"],
  "awaiting-confirmation": ["awaiting-confirmation", "confirmed", "rejected", "expired", "canceled", "failed"],
  confirmed: [],
  rejected: [],
  expired: [],
  canceled: [],
  failed: [],
} as const satisfies StateTransitionMap<PairingState>;

export const TRUST_TRANSITIONS = {
  unpaired: ["unpaired", "trusted"],
  trusted: ["trusted", "revoked"],
  revoked: ["revoked", "unpaired"],
} as const satisfies StateTransitionMap<TrustState>;

export const CAPABILITY_GRANT_TRANSITIONS = {
  requested: ["requested", "granted", "denied", "revoked"],
  granted: ["granted", "revoked"],
  denied: ["denied", "requested", "revoked"],
  revoked: ["revoked", "requested"],
} as const satisfies StateTransitionMap<CapabilityGrantState>;

export const TRANSFER_TRANSITIONS = {
  draft: ["draft", "awaiting-approval", "canceled"],
  "awaiting-approval": ["awaiting-approval", "queued", "canceled", "failed"],
  queued: ["queued", "active", "canceled", "failed"],
  active: ["active", "paused", "completed", "failed", "canceled"],
  paused: ["paused", "active", "canceled", "failed"],
  completed: [],
  failed: ["failed", "queued", "canceled"],
  canceled: [],
} as const satisfies StateTransitionMap<TransferState>;

export const NOTIFICATION_TRANSITIONS = {
  queued: ["queued", "delivered", "expired", "failed"],
  delivered: ["delivered", "dismissed", "expired"],
  dismissed: [],
  expired: [],
  failed: ["failed", "queued"],
} as const satisfies StateTransitionMap<NotificationState>;

export const REMOTE_COMMAND_TRANSITIONS = {
  requested: ["requested", "awaiting-approval", "running", "rejected", "canceled", "expired", "failed"],
  "awaiting-approval": ["awaiting-approval", "running", "rejected", "canceled", "expired", "failed"],
  running: ["running", "succeeded", "rejected", "failed", "canceled", "expired"],
  succeeded: [],
  rejected: [],
  failed: [],
  canceled: [],
  expired: [],
} as const satisfies StateTransitionMap<RemoteCommandState>;

export const TERMINAL_PAIRING_STATES: readonly PairingState[] = [
  "confirmed",
  "rejected",
  "expired",
  "canceled",
  "failed",
];

export const TERMINAL_TRANSFER_STATES: readonly TransferState[] = ["completed", "canceled"];
export const TERMINAL_NOTIFICATION_STATES: readonly NotificationState[] = ["dismissed", "expired"];
export const TERMINAL_REMOTE_COMMAND_STATES: readonly RemoteCommandState[] = [
  "succeeded",
  "rejected",
  "failed",
  "canceled",
  "expired",
];

/**
 * Tipos usados somente pelos fixtures visuais atuais. Eles não são modelos de
 * domínio e serão substituídos por adaptadores na TASK 10.
 */
export interface MockDevice {
  id: string;
  name: string;
  platform: DevicePlatform;
  online: boolean;
  lastSeen: string;
}

export type MockTransferStatus = "queued" | "in-progress" | "complete";

export interface MockTransfer {
  id: string;
  name: string;
  type: string;
  status: MockTransferStatus;
  progress: number;
  deviceName: string;
  updatedAt: string;
}

/** @deprecated Use MockDevice until the TASK 10 adapter is introduced. */
export type LegacyDevice = MockDevice;

/** @deprecated Use MockTransfer until the TASK 10 adapter is introduced. */
export type LegacyTransfer = MockTransfer;

export type BridgeState = "idle" | "loading" | "success" | "error";
