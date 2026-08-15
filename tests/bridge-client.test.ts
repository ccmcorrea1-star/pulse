import { describe, expect, it, vi } from "vitest";

import {
  BridgeClient,
  BridgeEventSequencer,
  type BridgeTransport,
} from "@/bridge/client";
import {
  BRIDGE_CONTRACT_VERSION,
  BRIDGE_STATUS_EVENT,
  type BridgeStatusEvent,
} from "@/types";

function statusEvent(overrides: Partial<BridgeStatusEvent> = {}): BridgeStatusEvent {
  return {
    bridgeContractVersion: BRIDGE_CONTRACT_VERSION,
    streamId: "status-stream",
    sequence: 1,
    eventId: "status-event-1" as BridgeStatusEvent["eventId"],
    emittedAt: "2026-08-15T12:00:00.000Z" as BridgeStatusEvent["emittedAt"],
    modelVersion: 1,
    payload: {
      runtimePhase: "partial",
      productCommandsAvailable: false,
    },
    ...overrides,
  };
}

function transportFor(
  response: unknown = {
    bridgeContractVersion: 1,
    requestId: "bridge-request",
    status: "success",
    generatedAt: "2026-08-15T12:00:00.000Z",
    data: {
      mode: "tauri",
      modelVersion: 1,
      runtimePhase: "partial",
      productCommandsAvailable: false,
    },
  },
) {
  let emitStatus: ((event: { payload: unknown }) => void) | undefined;
  let emitDomain: ((event: { payload: unknown }) => void) | undefined;
  let unlistenCalls = 0;

  const invoke = vi.fn(async (_command: string, args: Record<string, unknown>) => {
    if (typeof response === "object" && response !== null) {
      return {
        ...response,
        requestId: (args.request as { requestId: string }).requestId,
      };
    }
    return response;
  });

  const transport: BridgeTransport = {
    invoke: <T>(command: string, args: Record<string, unknown>) => invoke(command, args) as Promise<T>,
    listen: vi.fn(async (event, handler) => {
      if (event === BRIDGE_STATUS_EVENT) {
        emitStatus = handler;
      } else {
        emitDomain = handler;
      }
      return () => {
        unlistenCalls += 1;
      };
    }),
  };

  return {
    transport,
    invoke,
    emitStatus: (event: unknown) => emitStatus?.({ payload: event }),
    emitDomain: (event: unknown) => emitDomain?.({ payload: event }),
    getUnlistenCalls: () => unlistenCalls,
  };
}

describe("BridgeClient", () => {
  it("keeps web preview explicit and does not call the native transport", async () => {
    const fake = transportFor();
    const client = new BridgeClient(() => false, fake.transport);

    const info = await client.getInfo();
    const snapshot = await client.getSnapshot();

    expect(info.status).toBe("offline");
    expect(info.data?.mode).toBe("web-preview");
    expect(snapshot.status).toBe("offline");
    expect(snapshot.data?.productState).toBe("not-configured");
    expect(fake.invoke).not.toHaveBeenCalled();
  });

  it("validates the native response and sends the versioned request envelope", async () => {
    const fake = transportFor();
    const client = new BridgeClient(() => true, fake.transport);

    const response = await client.getInfo();
    const [command, args] = fake.invoke.mock.calls[0];

    expect(command).toBe("bridge_get_info");
    expect(args).toEqual({
      request: {
        bridgeContractVersion: 1,
        requestId: expect.any(String),
        payload: {},
      },
    });
    expect(response.data?.mode).toBe("tauri");
  });

  it("redacts raw IPC failures into a closed public error", async () => {
    const fake = transportFor();
    fake.transport.invoke = vi.fn(async () => {
      throw new Error("SQL token=/tmp/private.db");
    });
    const client = new BridgeClient(() => true, fake.transport);

    const error = await client.getInfo().catch((value) => value);

    expect(error).toEqual({
      bridgeContractVersion: 1,
      requestId: expect.any(String),
      code: "internal",
      retryable: true,
      messageKey: "bridge.requestFailed",
      reasonCode: "ipc-failure",
    });
    expect(JSON.stringify(error)).not.toContain("private.db");
  });

  it("does not trust an IPC rejection that only resembles a bridge error", async () => {
    const fake = transportFor();
    fake.transport.invoke = vi.fn(async () => {
      throw {
        bridgeContractVersion: 1,
        requestId: "bridge-request",
        code: "internal",
        retryable: true,
        messageKey: "/tmp/private.sql",
        reasonCode: "secret-token",
      };
    });
    const client = new BridgeClient(() => true, fake.transport);

    const error = await client.getInfo().catch((value) => value);

    expect(error.code).toBe("internal");
    expect(error.messageKey).toBe("bridge.requestFailed");
    expect(JSON.stringify(error)).not.toMatch(/private\.sql|secret-token/);
  });

  it("shares one native listener and only unlistens after the last subscriber", async () => {
    const fake = transportFor();
    const client = new BridgeClient(() => true, fake.transport);
    const first = vi.fn();
    const second = vi.fn();

    const unsubscribeFirst = await client.listenStatus(first);
    const unsubscribeSecond = await client.listenStatus(second);
    fake.emitStatus(statusEvent());
    fake.emitStatus(statusEvent());

    expect(fake.transport.listen).toHaveBeenCalledTimes(1);
    expect(first).toHaveBeenCalledTimes(1);
    expect(second).toHaveBeenCalledTimes(1);

    await unsubscribeFirst();
    expect(fake.getUnlistenCalls()).toBe(0);
    await unsubscribeSecond();
    expect(fake.getUnlistenCalls()).toBe(1);
  });

  it("keeps sequence gaps and stream changes on the resync path", () => {
    const sequencer = new BridgeEventSequencer();
    const first = statusEvent();
    const gap = statusEvent({ sequence: 3, eventId: "status-event-3" as BridgeStatusEvent["eventId"] });
    const newStream = statusEvent({
      streamId: "new-stream",
      eventId: "status-event-new" as BridgeStatusEvent["eventId"],
    });

    expect(sequencer.accept(first)).toBe("accepted");
    expect(sequencer.accept(first)).toBe("duplicate");
    expect(sequencer.accept(gap)).toBe("resync");
    expect(sequencer.accept(newStream)).toBe("resync");
  });

  it("requests a resync for malformed or out-of-order domain events", async () => {
    const fake = transportFor();
    const client = new BridgeClient(() => true, fake.transport);
    const received = vi.fn();
    const resync = vi.fn();

    const unsubscribe = await client.listenDomainEvents(received, resync);
    fake.emitDomain({
      bridgeContractVersion: 1,
      streamId: "domain-stream",
      sequence: 1,
      eventId: "domain-event-1",
      emittedAt: "2026-08-15T12:00:00.000Z",
      modelVersion: 1,
      payload: { type: "test" },
    });
    fake.emitDomain({
      bridgeContractVersion: 1,
      streamId: "domain-stream",
      sequence: 3,
      eventId: "domain-event-3",
      emittedAt: "2026-08-15T12:00:02.000Z",
      modelVersion: 1,
      payload: { type: "test" },
    });
    fake.emitDomain({ malformed: true });

    expect(received).toHaveBeenCalledTimes(1);
    expect(resync).toHaveBeenCalledTimes(2);
    await unsubscribe();
  });

  it("waits for a pending listener registration before unlistening on dispose", async () => {
    let resolveRegistration!: (unlisten: () => void) => void;
    let unlistenCalls = 0;
    const registration = new Promise<() => void>((resolve) => {
      resolveRegistration = resolve;
    });
    const transport: BridgeTransport = {
      invoke: vi.fn(),
      listen: vi.fn(() => registration),
    };
    const client = new BridgeClient(() => true, transport);
    const subscription = client.listenStatus(vi.fn());
    const disposed = client.dispose();

    expect(unlistenCalls).toBe(0);
    resolveRegistration(() => {
      unlistenCalls += 1;
    });
    await Promise.all([subscription, disposed]);
    expect(unlistenCalls).toBe(1);
  });
});
