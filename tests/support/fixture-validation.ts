import { BRIDGE_CONTRACT_VERSION, type BridgeErrorFixture, type BridgeEventFixture } from "../fixtures/bridge";
import { FIXTURE_VERSION } from "../fixtures/domain";

export function assertBridgeEventFixture(value: unknown): asserts value is BridgeEventFixture<unknown> {
  if (!value || typeof value !== "object") {
    throw new Error("Bridge event fixture must be an object");
  }

  const event = value as Partial<BridgeEventFixture<unknown>>;
  if (event.fixtureVersion !== FIXTURE_VERSION) {
    throw new Error(`Bridge event fixtureVersion must be ${FIXTURE_VERSION}`);
  }
  if (event.bridgeContractVersion !== BRIDGE_CONTRACT_VERSION) {
    throw new Error(`Bridge event bridgeContractVersion must be ${BRIDGE_CONTRACT_VERSION}`);
  }
  if (!event.streamId || !Number.isInteger(event.sequence) || !event.eventId || !event.payload) {
    throw new Error("Bridge event fixture is missing a required field");
  }
}

export function assertBridgeErrorFixture(value: unknown): asserts value is BridgeErrorFixture {
  if (!value || typeof value !== "object") {
    throw new Error("Bridge error fixture must be an object");
  }

  const error = value as Partial<BridgeErrorFixture>;
  if (error.fixtureVersion !== FIXTURE_VERSION || error.bridgeContractVersion !== BRIDGE_CONTRACT_VERSION) {
    throw new Error("Bridge error fixture has an unsupported version");
  }
  if (!error.requestId || !error.code || !error.messageKey || typeof error.retryable !== "boolean") {
    throw new Error("Bridge error fixture is missing a required field");
  }
}
