import { BridgeClient } from "@/bridge/client";

const bridgeClient = new BridgeClient();

export function useRustBridge() {
  return {
    greet: (name: string) => bridgeClient.greet(name),
    getInfo: () => bridgeClient.getInfo(),
    getSnapshot: () => bridgeClient.getSnapshot(),
    listenStatus: bridgeClient.listenStatus.bind(bridgeClient),
    listenDomainEvents: bridgeClient.listenDomainEvents.bind(bridgeClient),
    dispose: () => bridgeClient.dispose(),
    isTauri: () => bridgeClient.isTauri(),
  };
}
