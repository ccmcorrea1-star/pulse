import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";

import { useAppStore } from "@/stores/app";
import { useDevicesStore } from "@/stores/devices";
import { useTransfersStore } from "@/stores/transfers";

describe("app bridge state", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("hydrates the infrastructure state in web preview without native calls", async () => {
    const appStore = useAppStore();

    await appStore.initialize();

    expect(appStore.bridgeMode).toBe("web-preview");
    expect(appStore.bridgeSyncState).toBe("offline");
    expect(appStore.runtimePhase).toBe("partial");
    expect(appStore.productState).toBe("not-configured");
    expect(appStore.bridgeError).toBeUndefined();
    expect(appStore.resyncPending).toBe(false);
  });

  it("keeps bridge bootstrap idempotent and preserves the fixture boundary", async () => {
    const appStore = useAppStore();
    const devicesStore = useDevicesStore();
    const transfersStore = useTransfersStore();

    await Promise.all([appStore.initialize(), appStore.initialize()]);

    expect(devicesStore.source).toBe("development-fixture");
    expect(devicesStore.isDemo).toBe(true);
    expect(devicesStore.devices).toHaveLength(3);
    expect(devicesStore.onlineDevices).toHaveLength(2);
    expect(devicesStore.devices[0].presence).toBe("online");
    expect(transfersStore.source).toBe("development-fixture");
    expect(transfersStore.activeTransfers).toHaveLength(2);
    expect(transfersStore.transfers[0].status).toBe("active");
    expect(transfersStore.transfers[0].domainState).toBeUndefined();
  });

  it("keeps cleanup safe when preview listeners are no-ops", async () => {
    const appStore = useAppStore();

    await appStore.initialize();
    await expect(appStore.dispose()).resolves.toBeUndefined();
    expect(appStore.domainEventCount).toBe(0);
  });
});
