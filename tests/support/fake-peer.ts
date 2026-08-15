import type { DeviceId, PresenceState, TrustState } from "@/types";

import type { BridgeErrorFixture, BridgeEventFixture } from "../fixtures/bridge";
import { fixturePeerId } from "../fixtures/domain";

export class FakePeer {
  private readonly pendingEvents: BridgeEventFixture<unknown>[] = [];

  public presence: PresenceState = "online";
  public trust: TrustState = "unpaired";

  constructor(public readonly id: DeviceId = fixturePeerId) {}

  setPresence(state: PresenceState): void {
    this.presence = state;
  }

  setTrust(state: TrustState): void {
    this.trust = state;
  }

  enqueue<T>(event: BridgeEventFixture<T>): void {
    this.pendingEvents.push(event as BridgeEventFixture<unknown>);
  }

  duplicateLast(): void {
    const last = this.pendingEvents.at(-1);
    if (last) {
      this.pendingEvents.push({ ...last });
    }
  }

  dropNext(): void {
    this.pendingEvents.shift();
  }

  drain(): BridgeEventFixture<unknown>[] {
    return this.pendingEvents.splice(0);
  }

  error(code: BridgeErrorFixture["code"] = "peer-offline"): BridgeErrorFixture {
    return {
      fixtureVersion: 1,
      bridgeContractVersion: 1,
      requestId: "request-fake-peer",
      code,
      retryable: code !== "invalid-request",
      messageKey: code === "peer-offline" ? "bridge.peerOffline" : "bridge.requestFailed",
    };
  }
}
