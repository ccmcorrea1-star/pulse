import type { DomainEventId, UtcTimestamp } from "./index";

export const BRIDGE_CONTRACT_VERSION = 1 as const;
export const BRIDGE_STATUS_EVENT = "pulse.bridge.status" as const;
export const DOMAIN_EVENT_EVENT = "pulse.domain.event" as const;
export const SNAPSHOT_INVALIDATED_EVENT = "pulse.domain.snapshot-invalidated" as const;

export type BridgeContractVersion = typeof BRIDGE_CONTRACT_VERSION;
export type BridgeMode = "tauri" | "web-preview";
export type PublicRuntimePhase =
  | "created"
  | "starting"
  | "partial"
  | "ready"
  | "failed"
  | "stopping"
  | "stopped";
export type ProductState = "not-configured";
export type BridgeReadStatus = "success" | "stale" | "offline";

export interface BridgeRequest<T> {
  bridgeContractVersion: BridgeContractVersion;
  requestId: string;
  payload: T;
}

export interface BridgeReadResponse<T> {
  bridgeContractVersion: BridgeContractVersion;
  requestId: string;
  status: BridgeReadStatus;
  generatedAt: UtcTimestamp;
  observedAt?: UtcTimestamp;
  data?: T;
}

export interface BridgeInfo {
  mode: BridgeMode;
  modelVersion: 1;
  runtimePhase: PublicRuntimePhase;
  productCommandsAvailable: false;
}

export interface BridgeSnapshot {
  runtimePhase: PublicRuntimePhase;
  productState: ProductState;
}

export interface BridgeEvent<T> {
  bridgeContractVersion: BridgeContractVersion;
  streamId: string;
  sequence: number;
  eventId: DomainEventId;
  emittedAt: UtcTimestamp;
  modelVersion: 1;
  payload: T;
}

export interface BridgeStatusPayload {
  runtimePhase: PublicRuntimePhase;
  productCommandsAvailable: false;
}

export type BridgeStatusEvent = BridgeEvent<BridgeStatusPayload>;

export type BridgeErrorCode =
  | "invalid-request"
  | "unsupported-contract-version"
  | "runtime-not-ready"
  | "not-found"
  | "already-resolved"
  | "trust-required"
  | "capability-denied"
  | "peer-offline"
  | "transport-unavailable"
  | "storage-unavailable"
  | "timeout"
  | "canceled"
  | "conflict"
  | "internal";

export interface BridgeError {
  bridgeContractVersion: BridgeContractVersion;
  requestId: string;
  code: BridgeErrorCode;
  retryable: boolean;
  messageKey: string;
  reasonCode?: string;
}
