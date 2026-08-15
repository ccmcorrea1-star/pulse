import { describe, expect, it } from "vitest";

import { bridgeEventFixture } from "./fixtures/bridge";
import { FakePeer } from "./support/fake-peer";

describe("fake peer", () => {
  it("changes presence without changing trust", () => {
    const peer = new FakePeer();
    peer.setTrust("trusted");
    peer.setPresence("offline");

    expect(peer.presence).toBe("offline");
    expect(peer.trust).toBe("trusted");
  });

  it("can duplicate and drop events deterministically", () => {
    const peer = new FakePeer();
    const first = bridgeEventFixture({ sequence: 1 });
    const second = bridgeEventFixture({ sequence: 2, eventId: "event-second" as typeof first.eventId });

    peer.enqueue(first);
    peer.enqueue(second);
    peer.duplicateLast();
    peer.dropNext();

    expect(peer.drain().map((event) => event.sequence)).toEqual([2, 2]);
  });

  it("returns a public retryable error without transport details", () => {
    const error = new FakePeer().error("peer-offline");

    expect(error.code).toBe("peer-offline");
    expect(error.retryable).toBe(true);
    expect(JSON.stringify(error)).not.toMatch(/socket|quic|127\.0\.0\.1|token/i);
  });
});
