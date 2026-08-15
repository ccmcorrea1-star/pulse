import type { DomainEvent, DomainEventId, UtcTimestamp } from "@/types";

import { domainEventFixture, FIXTURE_VERSION } from "./domain";

export const BRIDGE_CONTRACT_VERSION = 1 as const;

export interface BridgeEventFixture<T> {
  fixtureVersion: typeof FIXTURE_VERSION;
  bridgeContractVersion: typeof BRIDGE_CONTRACT_VERSION;
  streamId: string;
  sequence: number;
  eventId: DomainEventId;
  emittedAt: UtcTimestamp;
  modelVersion: 1;
  payload: T;
}

export type BridgeErrorCode =
  | "invalid-request"
  | "unsupported-contract-version"
  | "peer-offline"
  | "timeout"
  | "internal";

export interface BridgeErrorFixture {
  fixtureVersion: typeof FIXTURE_VERSION;
  bridgeContractVersion: typeof BRIDGE_CONTRACT_VERSION;
  requestId: string;
  code: BridgeErrorCode;
  retryable: boolean;
  messageKey: string;
  reasonCode?: string;
}

export function bridgeEventFixture(
  overrides: Partial<BridgeEventFixture<DomainEvent>> = {},
): BridgeEventFixture<DomainEvent> {
  const event = domainEventFixture().data;

  return {
    fixtureVersion: FIXTURE_VERSION,
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    streamId: "stream-fixture",
    sequence: 1,
    eventId: event.id,
    emittedAt: event.occurredAt,
    modelVersion: 1,
    payload: event,
    ...overrides,
  };
}

export function bridgeErrorFixture(
  overrides: Partial<BridgeErrorFixture> = {},
): BridgeErrorFixture {
  return {
    fixtureVersion: FIXTURE_VERSION,
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    requestId: "request-fixture",
    code: "peer-offline",
    retryable: true,
    messageKey: "bridge.peerOffline",
    ...overrides,
  };
}
