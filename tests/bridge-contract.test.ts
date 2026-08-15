import { describe, expect, it } from "vitest";

import { bridgeErrorFixture, bridgeEventFixture } from "./fixtures/bridge";
import { FIXTURE_VERSION } from "./fixtures/domain";
import { assertBridgeErrorFixture, assertBridgeEventFixture } from "./support/fixture-validation";

describe("bridge contract fixtures", () => {
  it("keeps event envelopes versioned and sequenced", () => {
    const event = bridgeEventFixture({ sequence: 3 });

    assertBridgeEventFixture(event);

    expect(event.fixtureVersion).toBe(FIXTURE_VERSION);
    expect(event.bridgeContractVersion).toBe(1);
    expect(event.modelVersion).toBe(1);
    expect(event.sequence).toBe(3);
  });

  it("rejects an event fixture with an unsupported contract version", () => {
    const event = bridgeEventFixture({ bridgeContractVersion: 2 as 1 });

    expect(() => assertBridgeEventFixture(event)).toThrow("bridgeContractVersion");
  });

  it("keeps public errors closed and redacted", () => {
    const error = bridgeErrorFixture({
      code: "invalid-request",
      retryable: false,
      messageKey: "bridge.invalidRequest",
    });

    assertBridgeErrorFixture(error);

    expect(error.messageKey).toBe("bridge.invalidRequest");
    expect(JSON.stringify(error)).not.toMatch(/token|secret|\/home|sql/i);
    expect(error.retryable).toBe(false);
  });

  it("diagnoses malformed errors instead of accepting partial data", () => {
    const malformed = { ...bridgeErrorFixture(), messageKey: "" };

    expect(() => assertBridgeErrorFixture(malformed)).toThrow("required field");
  });
});
