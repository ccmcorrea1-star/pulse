import type {
  ByteCount,
  CapabilityGrant,
  CapabilityInfo,
  Device,
  DeviceId,
  DevicePlatform,
  DomainEvent,
  DomainEventId,
  PairingSession,
  PairingSessionId,
  PairingState,
  Presence,
  PresenceState,
  TransferItem,
  TransferSession,
  UtcTimestamp,
} from "@/types";

export const FIXTURE_VERSION = 1 as const;

export interface VersionedFixture<T> {
  fixtureVersion: typeof FIXTURE_VERSION;
  data: T;
}

const timestamp = (value: string) => value as UtcTimestamp;
const deviceId = (value: string) => value as DeviceId;
const pairingSessionId = (value: string) => value as PairingSessionId;
const domainEventId = (value: string) => value as DomainEventId;
const byteCount = (value: number) => value as ByteCount;

export const fixtureTimestamp = timestamp("2026-01-01T00:00:00.000Z");
export const fixtureDeviceId = deviceId("device-fixture");
export const fixturePeerId = deviceId("peer-fixture");

function versionFixture<T>(data: T): VersionedFixture<T> {
  return { fixtureVersion: FIXTURE_VERSION, data };
}

export function deviceFixture(overrides: Partial<Device> = {}): VersionedFixture<Device> {
  const trust = {
    deviceId: fixtureDeviceId,
    state: "unpaired" as const,
    updatedAt: fixtureTimestamp,
  };

  const capabilities: CapabilityInfo[] = [
    {
      key: "text.send",
      available: true,
      direction: "send",
      observedAt: fixtureTimestamp,
    },
  ];

  return versionFixture({
    id: fixtureDeviceId,
    name: "Fixture Desktop",
    platform: "linux" as DevicePlatform,
    trust,
    capabilities,
    ...overrides,
  });
}

export function presenceFixture(
  overrides: Partial<Presence> = {},
): VersionedFixture<Presence> {
  return versionFixture({
    deviceId: fixturePeerId,
    state: "online" as PresenceState,
    observedAt: fixtureTimestamp,
    lastSeenAt: fixtureTimestamp,
    ...overrides,
  });
}

export function pairingSessionFixture(
  overrides: Partial<PairingSession> = {},
): VersionedFixture<PairingSession> {
  const createdAt = fixtureTimestamp;
  return versionFixture({
    id: pairingSessionId("pairing-fixture"),
    initiatorDeviceId: fixtureDeviceId,
    candidateId: "candidate-fixture" as PairingSession["candidateId"],
    presentedIdentity: {
      name: "Fixture Peer",
      platform: "linux",
    },
    state: "awaiting-confirmation" as PairingState,
    createdAt,
    updatedAt: createdAt,
    expiresAt: timestamp("2026-01-01T00:02:00.000Z"),
    ...overrides,
  });
}

export function capabilityGrantFixture(
  overrides: Partial<CapabilityGrant> = {},
): VersionedFixture<CapabilityGrant> {
  return versionFixture({
    deviceId: fixturePeerId,
    key: "text.send",
    direction: "send",
    state: "requested",
    requestedAt: fixtureTimestamp,
    ...overrides,
  });
}

export function transferSessionFixture(
  overrides: Partial<TransferSession> = {},
): VersionedFixture<TransferSession> {
  const item: TransferItem = {
    id: "transfer-item-fixture" as TransferItem["id"],
    kind: "file",
    name: "fixture.txt",
    sizeBytes: byteCount(12),
  };

  return versionFixture({
    id: "transfer-fixture" as TransferSession["id"],
    sourceDeviceId: fixtureDeviceId,
    destinationDeviceId: fixturePeerId,
    direction: "outgoing",
    kind: "file",
    items: [item],
    state: "queued",
    progress: { mode: "bytes", completedBytes: byteCount(0), totalBytes: byteCount(12) },
    attempt: 1,
    destinationPolicy: "ask",
    createdAt: fixtureTimestamp,
    updatedAt: fixtureTimestamp,
    queuedAt: fixtureTimestamp,
    ...overrides,
  });
}

export function domainEventFixture(
  overrides: Partial<DomainEvent> = {},
): VersionedFixture<DomainEvent> {
  return versionFixture({
    id: domainEventId("event-fixture"),
    type: "presence.updated",
    entity: { kind: "presence", id: fixturePeerId },
    occurredAt: fixtureTimestamp,
    modelVersion: 1,
    payload: { state: "online" },
    ...overrides,
  });
}
