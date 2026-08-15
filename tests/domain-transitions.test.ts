import { describe, expect, it } from "vitest";

import {
  CAPABILITY_GRANT_TRANSITIONS,
  DISCOVERY_CANDIDATE_TRANSITIONS,
  NOTIFICATION_TRANSITIONS,
  PAIRING_TRANSITIONS,
  PRESENCE_TRANSITIONS,
  REMOTE_COMMAND_TRANSITIONS,
  TERMINAL_PAIRING_STATES,
  TERMINAL_REMOTE_COMMAND_STATES,
  TERMINAL_TRANSFER_STATES,
  TRANSFER_TRANSITIONS,
  TRUST_TRANSITIONS,
} from "@/types";

import { pairingSessionFixture, presenceFixture, transferSessionFixture } from "./fixtures/domain";
import { TestClock } from "./support/test-clock";

describe("domain transition fixtures", () => {
  it("keeps discovery candidates finite", () => {
    expect(DISCOVERY_CANDIDATE_TRANSITIONS.discovered).toContain("expired");
    expect(DISCOVERY_CANDIDATE_TRANSITIONS.expired).toEqual([]);
  });

  it("keeps presence independent from trust", () => {
    const presence = presenceFixture({ state: "online" }).data;
    const clock = new TestClock(presence.observedAt);

    clock.advance(30_000);
    const stale = { ...presence, state: "stale" as const, staleAt: clock.now() };
    clock.advance(30_000);
    const offline = { ...stale, state: "offline" as const };

    expect(stale.state).toBe("stale");
    expect(offline.state).toBe("offline");
    expect(PRESENCE_TRANSITIONS.online).toContain("stale");
    expect(PRESENCE_TRANSITIONS.stale).toContain("offline");
  });

  it("does not resolve pairing or transfer fixtures by changing a timestamp", () => {
    const pairing = pairingSessionFixture().data;
    const transfer = transferSessionFixture().data;
    const clock = new TestClock(pairing.createdAt);

    clock.advance(120_000);

    expect(pairing.state).toBe("awaiting-confirmation");
    expect(pairing.expiresAt).toBe(clock.now());
    expect(transfer.state).toBe("queued");
    expect(transfer.result).toBeUndefined();
  });

  it("declares terminal states without allowing a hidden next state", () => {
    expect(TERMINAL_PAIRING_STATES).toContain("confirmed");
    expect(TERMINAL_TRANSFER_STATES).toContain("canceled");
    expect(TERMINAL_REMOTE_COMMAND_STATES).toContain("succeeded");
    expect(PAIRING_TRANSITIONS.confirmed).toEqual([]);
    expect(TRANSFER_TRANSITIONS.completed).toEqual([]);
    expect(REMOTE_COMMAND_TRANSITIONS.succeeded).toEqual([]);
  });

  it("keeps trust and capability transitions explicit", () => {
    expect(TRUST_TRANSITIONS.revoked).toEqual(["revoked", "unpaired"]);
    expect(CAPABILITY_GRANT_TRANSITIONS.denied).toContain("requested");
    expect(CAPABILITY_GRANT_TRANSITIONS.granted).not.toContain("requested");
    expect(NOTIFICATION_TRANSITIONS.dismissed).toEqual([]);
  });
});
