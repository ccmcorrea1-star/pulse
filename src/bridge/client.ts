import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  BridgeError,
  BridgeEvent,
  BridgeInfo,
  BridgeReadResponse,
  BridgeSnapshot,
  BridgeStatusPayload,
  DomainEventId,
  ProductState,
  PublicRuntimePhase,
  UtcTimestamp,
} from "@/types";
import {
  BRIDGE_CONTRACT_VERSION,
  BRIDGE_STATUS_EVENT,
  DOMAIN_EVENT_EVENT,
} from "@/types";

const EMPTY_PAYLOAD: Record<string, never> = {};
const REQUEST_ID_MAX_LENGTH = 128;

export type BridgeEventHandler<T> = (event: BridgeEvent<T>) => void;
export type BridgeUnsubscribe = () => Promise<void>;

export interface BridgeTransport {
  invoke<T>(command: string, args: Record<string, unknown>): Promise<T>;
  listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<UnlistenFn>;
}

const tauriTransport: BridgeTransport = {
  invoke: <T>(command: string, args: Record<string, unknown>) => invoke<T>(command, args),
  listen: <T>(event: string, handler: (event: { payload: T }) => void) => listen<T>(event, handler),
};

export type BridgeEventDecision = "accepted" | "duplicate" | "ignored" | "resync";

export class BridgeEventSequencer {
  private streamId?: string;
  private lastSequence = 0;
  private readonly seenEventIds = new Set<string>();

  accept(event: BridgeEvent<unknown>): BridgeEventDecision {
    if (this.seenEventIds.has(event.eventId)) {
      return "duplicate";
    }

    if (!this.streamId) {
      this.streamId = event.streamId;
      this.lastSequence = event.sequence;
      this.seenEventIds.add(event.eventId);
      return "accepted";
    }

    if (this.streamId !== event.streamId) {
      this.streamId = event.streamId;
      this.lastSequence = event.sequence;
      this.seenEventIds.clear();
      this.seenEventIds.add(event.eventId);
      return "resync";
    }

    if (event.sequence <= this.lastSequence) {
      return "ignored";
    }

    if (event.sequence !== this.lastSequence + 1) {
      return "resync";
    }

    this.lastSequence = event.sequence;
    this.seenEventIds.add(event.eventId);
    return "accepted";
  }

  reset(): void {
    this.streamId = undefined;
    this.lastSequence = 0;
    this.seenEventIds.clear();
  }
}

export function validateBridgeEvent<T>(
  value: unknown,
  payloadGuard: (payload: unknown) => payload is T = (payload): payload is T => isRecord(payload),
): BridgeEvent<T> | null {
  if (!isRecord(value) || value.bridgeContractVersion !== BRIDGE_CONTRACT_VERSION) {
    return null;
  }

  if (
    !isNonEmptyString(value.streamId) ||
    !isPositiveInteger(value.sequence) ||
    !isNonEmptyString(value.eventId) ||
    !isNonEmptyString(value.emittedAt) ||
    value.modelVersion !== 1 ||
    !payloadGuard(value.payload)
  ) {
    return null;
  }

  return {
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    streamId: value.streamId,
    sequence: value.sequence,
    eventId: value.eventId as DomainEventId,
    emittedAt: value.emittedAt as UtcTimestamp,
    modelVersion: 1,
    payload: value.payload,
  };
}

export function isBridgeError(value: unknown): value is BridgeError {
  if (!isRecord(value)) {
    return false;
  }

  return (
    value.bridgeContractVersion === BRIDGE_CONTRACT_VERSION &&
    isNonEmptyString(value.requestId) &&
    isValidRequestId(value.requestId) &&
    isBridgeErrorCode(value.code) &&
    typeof value.retryable === "boolean" &&
    isPublicMessageKey(value.messageKey) &&
    (value.reasonCode === undefined || isPublicReasonCode(value.reasonCode))
  );
}

export class BridgeClient {
  private readonly statusSubscribers = new Set<BridgeEventHandler<BridgeStatusPayload>>();
  private readonly domainSubscribers = new Set<BridgeEventHandler<unknown>>();
  private readonly domainResyncHandlers = new Set<() => void>();
  private readonly statusSequencer = new BridgeEventSequencer();
  private readonly domainSequencer = new BridgeEventSequencer();
  private statusListener?: Promise<UnlistenFn>;
  private domainListener?: Promise<UnlistenFn>;

  constructor(
    private readonly detectTauri: () => boolean = defaultIsTauri,
    private readonly transport: BridgeTransport = tauriTransport,
  ) {}

  isTauri(): boolean {
    return this.detectTauri();
  }

  async greet(name: string): Promise<{ message: string; isDemo: boolean }> {
    if (!this.isTauri()) {
      return {
        message: `Prévia web ativa para ${name}.`,
        isDemo: true,
      };
    }

    return {
      message: await this.transport.invoke<string>("greet", { name }),
      isDemo: false,
    };
  }

  async getInfo(): Promise<BridgeReadResponse<BridgeInfo>> {
    const requestId = createRequestId();
    if (!this.isTauri()) {
      return previewResponse(requestId, "offline", previewBridgeInfo());
    }

    return this.invokeRead("bridge_get_info", requestId, isBridgeInfo);
  }

  async getSnapshot(): Promise<BridgeReadResponse<BridgeSnapshot>> {
    const requestId = createRequestId();
    if (!this.isTauri()) {
      return previewResponse(requestId, "offline", previewBridgeSnapshot());
    }

    const response = await this.invokeRead("bridge_get_snapshot", requestId, isBridgeSnapshot);
    this.domainSequencer.reset();
    return response;
  }

  async listenStatus(handler: BridgeEventHandler<BridgeStatusPayload>): Promise<BridgeUnsubscribe> {
    if (!this.isTauri()) {
      return async () => undefined;
    }

    this.statusSubscribers.add(handler);
    try {
      await this.ensureStatusListener();
    } catch (error) {
      this.statusSubscribers.delete(handler);
      throw normalizeBridgeError(error, "status-listener");
    }

    let active = true;
    return async () => {
      if (!active) {
        return;
      }
      active = false;
      this.statusSubscribers.delete(handler);
      await this.releaseListener("status");
    };
  }

  async listenDomainEvents(
    handler: BridgeEventHandler<unknown>,
    onResyncRequired?: () => void,
  ): Promise<BridgeUnsubscribe> {
    if (!this.isTauri()) {
      return async () => undefined;
    }

    this.domainSubscribers.add(handler);
    if (onResyncRequired) {
      this.domainResyncHandlers.add(onResyncRequired);
    }

    try {
      await this.ensureDomainListener();
    } catch (error) {
      this.domainSubscribers.delete(handler);
      if (onResyncRequired) {
        this.domainResyncHandlers.delete(onResyncRequired);
      }
      throw normalizeBridgeError(error, "domain-listener");
    }

    let active = true;
    return async () => {
      if (!active) {
        return;
      }
      active = false;
      this.domainSubscribers.delete(handler);
      if (onResyncRequired) {
        this.domainResyncHandlers.delete(onResyncRequired);
      }
      await this.releaseListener("domain");
    };
  }

  async dispose(): Promise<void> {
    this.statusSubscribers.clear();
    this.domainSubscribers.clear();
    this.domainResyncHandlers.clear();
    this.statusSequencer.reset();
    this.domainSequencer.reset();

    const listeners = [this.statusListener, this.domainListener];
    this.statusListener = undefined;
    this.domainListener = undefined;

    await Promise.all(listeners.map((listener) => settleUnlisten(listener)));
  }

  private async invokeRead<T>(
    command: string,
    requestId: string,
    dataGuard: (data: unknown) => data is T,
  ): Promise<BridgeReadResponse<T>> {
    const request = {
      bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
      requestId,
      payload: EMPTY_PAYLOAD,
    };

    try {
      const rawResponse = await this.transport.invoke<unknown>(command, { request });
      const response = validateReadResponse(rawResponse, requestId, dataGuard);
      if (!response || response.data === undefined) {
        throw new Error("invalid bridge response");
      }
      return response as BridgeReadResponse<T>;
    } catch (error) {
      throw normalizeBridgeError(error, requestId);
    }
  }

  private ensureStatusListener(): Promise<UnlistenFn> {
    if (!this.statusListener) {
      const onEvent = ({ payload }: { payload: unknown }) => {
        const event = validateBridgeEvent(payload, isBridgeStatusPayload);
        if (!event) {
          return;
        }

        const decision = this.statusSequencer.accept(event);
        if (decision === "accepted" || decision === "resync") {
          this.statusSubscribers.forEach((subscriber) => subscriber(event));
        }
      };
      this.statusListener = this.registerListener(BRIDGE_STATUS_EVENT, onEvent, "status");
    }
    return this.statusListener;
  }

  private ensureDomainListener(): Promise<UnlistenFn> {
    if (!this.domainListener) {
      const onEvent = ({ payload }: { payload: unknown }) => {
        const event = validateBridgeEvent(payload);
        if (!event) {
          this.requestResync();
          return;
        }

        const decision = this.domainSequencer.accept(event);
        if (decision === "accepted") {
          this.domainSubscribers.forEach((subscriber) => subscriber(event));
        } else if (decision === "resync") {
          this.requestResync();
        }
      };
      this.domainListener = this.registerListener(DOMAIN_EVENT_EVENT, onEvent, "domain");
    }
    return this.domainListener;
  }

  private registerListener(
    event: string,
    handler: (event: { payload: unknown }) => void,
    kind: "status" | "domain",
  ): Promise<UnlistenFn> {
    let registration: Promise<UnlistenFn>;
    try {
      registration = this.transport.listen(event, handler);
    } catch (error) {
      registration = Promise.reject(error);
    }

    return registration.catch((error) => {
      if (kind === "status") {
        this.statusListener = undefined;
      } else {
        this.domainListener = undefined;
      }
      throw error;
    });
  }

  private requestResync(): void {
    this.domainSequencer.reset();
    this.domainResyncHandlers.forEach((handler) => handler());
  }

  private async releaseListener(kind: "status" | "domain"): Promise<void> {
    if (kind === "status" && this.statusSubscribers.size === 0) {
      const listener = this.statusListener;
      this.statusListener = undefined;
      await settleUnlisten(listener);
      return;
    }

    if (kind === "domain" && this.domainSubscribers.size === 0) {
      const listener = this.domainListener;
      this.domainListener = undefined;
      await settleUnlisten(listener);
    }
  }
}

function defaultIsTauri(): boolean {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

function createRequestId(): string {
  return `bridge-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

function validateReadResponse<T>(
  value: unknown,
  expectedRequestId: string,
  dataGuard: (data: unknown) => data is T,
): BridgeReadResponse<T> | null {
  if (!isRecord(value) || value.bridgeContractVersion !== BRIDGE_CONTRACT_VERSION) {
    return null;
  }

  if (
    value.requestId !== expectedRequestId ||
    !isBridgeReadStatus(value.status) ||
    !isNonEmptyString(value.generatedAt) ||
    (value.observedAt !== undefined && !isNonEmptyString(value.observedAt)) ||
    (value.data !== undefined && !dataGuard(value.data))
  ) {
    return null;
  }

  return {
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    requestId: value.requestId,
    status: value.status,
    generatedAt: value.generatedAt as UtcTimestamp,
    ...(value.observedAt === undefined
      ? {}
      : { observedAt: value.observedAt as UtcTimestamp }),
    ...(value.data === undefined ? {} : { data: value.data }),
  };
}

function previewResponse<T>(
  requestId: string,
  status: "offline",
  data: T,
): BridgeReadResponse<T> {
  return {
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    requestId,
    status,
    generatedAt: timestampNow(),
    data,
  };
}

function previewBridgeInfo(): BridgeInfo {
  return {
    mode: "web-preview",
    modelVersion: 1,
    runtimePhase: "partial",
    productCommandsAvailable: false,
  };
}

function previewBridgeSnapshot(): BridgeSnapshot {
  return {
    runtimePhase: "partial",
    productState: "not-configured" satisfies ProductState,
  };
}

function timestampNow(): UtcTimestamp {
  return new Date().toISOString() as UtcTimestamp;
}

function normalizeBridgeError(error: unknown, requestId: string): BridgeError {
  if (isBridgeError(error)) {
    return error;
  }

  return {
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    requestId: isValidRequestId(requestId) ? requestId : "bridge-request",
    code: "internal",
    retryable: true,
    messageKey: "bridge.requestFailed",
    reasonCode: "ipc-failure",
  };
}

function isBridgeInfo(value: unknown): value is BridgeInfo {
  return (
    isRecord(value) &&
    (value.mode === "tauri" || value.mode === "web-preview") &&
    value.modelVersion === 1 &&
    isPublicRuntimePhase(value.runtimePhase) &&
    value.productCommandsAvailable === false
  );
}

function isBridgeSnapshot(value: unknown): value is BridgeSnapshot {
  return isRecord(value) && isPublicRuntimePhase(value.runtimePhase) && value.productState === "not-configured";
}

function isBridgeStatusPayload(value: unknown): value is BridgeStatusPayload {
  return isRecord(value) && isPublicRuntimePhase(value.runtimePhase) && value.productCommandsAvailable === false;
}

function isPublicRuntimePhase(value: unknown): value is PublicRuntimePhase {
  return (
    value === "created" ||
    value === "starting" ||
    value === "partial" ||
    value === "ready" ||
    value === "failed" ||
    value === "stopping" ||
    value === "stopped"
  );
}

function isBridgeReadStatus(value: unknown): value is BridgeReadResponse<unknown>["status"] {
  return value === "success" || value === "stale" || value === "offline";
}

function isBridgeErrorCode(value: unknown): value is BridgeError["code"] {
  return (
    value === "invalid-request" ||
    value === "unsupported-contract-version" ||
    value === "runtime-not-ready" ||
    value === "not-found" ||
    value === "already-resolved" ||
    value === "trust-required" ||
    value === "capability-denied" ||
    value === "peer-offline" ||
    value === "transport-unavailable" ||
    value === "storage-unavailable" ||
    value === "timeout" ||
    value === "canceled" ||
    value === "conflict" ||
    value === "internal"
  );
}

function isPublicMessageKey(value: unknown): value is string {
  return (
    value === "bridge.invalidRequest" ||
    value === "bridge.unsupportedContractVersion" ||
    value === "bridge.runtimeNotReady" ||
    value === "bridge.requestFailed"
  );
}

function isPublicReasonCode(value: unknown): value is string {
  return (
    value === "invalid-request-id" ||
    value === "unsupported-contract-version" ||
    value === "unsupported-window" ||
    value === "runtime-snapshot-unavailable" ||
    value === "ipc-failure"
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function isPositiveInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value > 0;
}

function isValidRequestId(value: string): boolean {
  return (
    value.length > 0 &&
    value.length <= REQUEST_ID_MAX_LENGTH &&
    [...value].every((character) => /[A-Za-z0-9._:-]/.test(character))
  );
}

async function settleUnlisten(listener: Promise<UnlistenFn> | undefined): Promise<void> {
  if (!listener) {
    return;
  }

  try {
    const unlisten = await listener;
    unlisten();
  } catch {
    // A failed registration has no listener to remove and must not leak raw IPC errors.
  }
}
