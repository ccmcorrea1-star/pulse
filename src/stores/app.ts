import { defineStore } from "pinia";
import { ref } from "vue";

import { isBridgeError } from "@/bridge/client";
import { useRustBridge } from "@/composables/useRustBridge";
import { useDevicesStore } from "@/stores/devices";
import { useTransfersStore } from "@/stores/transfers";
import type { BridgeMode, BridgeState, ProductState, PublicRuntimePhase, UtcTimestamp } from "@/types";

export type BridgeSyncState = "idle" | "loading" | "ready" | "stale" | "offline" | "error";

export const useAppStore = defineStore("app", () => {
  const version = "0.1.0";
  const bridgeState = ref<BridgeState>("idle");
  const bridgeMessage = ref("A bridge ainda não foi testada.");
  const bridgeSyncState = ref<BridgeSyncState>("idle");
  const bridgeMode = ref<BridgeMode>();
  const runtimePhase = ref<PublicRuntimePhase>();
  const productState = ref<ProductState>();
  const lastSnapshotAt = ref<UtcTimestamp>();
  const lastEventAt = ref<UtcTimestamp>();
  const domainEventCount = ref(0);
  const resyncPending = ref(false);
  const bridgeError = ref<string>();
  const { greet, getInfo, getSnapshot, listenStatus, listenDomainEvents, dispose: disposeBridge } = useRustBridge();
  const devicesStore = useDevicesStore();
  const transfersStore = useTransfersStore();
  let bootstrapPromise: Promise<void> | undefined;
  let resyncPromise: Promise<void> | undefined;
  let stopStatusListener: (() => Promise<void>) | undefined;
  let stopDomainListener: (() => Promise<void>) | undefined;

  async function testBridge() {
    bridgeState.value = "loading";

    try {
      const result = await greet("Pulse");
      bridgeMessage.value = result.isDemo
        ? `${result.message} Execute no Tauri para validar o Rust.`
        : result.message;
      bridgeState.value = "success";
    } catch (error) {
      bridgeState.value = "error";
      bridgeMessage.value = isBridgeError(error) ? `${error.code} · ${error.messageKey}` : "bridge.requestFailed";
    }
  }

  async function initialize(): Promise<void> {
    if (bootstrapPromise) {
      return bootstrapPromise;
    }

    bootstrapPromise = bootstrap();
    return bootstrapPromise;
  }

  async function bootstrap(): Promise<void> {
    bridgeSyncState.value = "loading";
    bridgeError.value = undefined;

    try {
      stopStatusListener = await listenStatus((event) => {
        runtimePhase.value = event.payload.runtimePhase;
        lastEventAt.value = event.emittedAt;
        if (event.payload.runtimePhase === "failed") {
          bridgeSyncState.value = "error";
        } else if (event.payload.runtimePhase === "stopping" || event.payload.runtimePhase === "stopped") {
          bridgeSyncState.value = "offline";
        }
      });
      stopDomainListener = await listenDomainEvents(
        (event) => {
          domainEventCount.value += 1;
          lastEventAt.value = event.emittedAt;
        },
        () => {
          void requestResync();
        },
      );

      const info = await getInfo();
      if (info.data) {
        bridgeMode.value = info.data.mode;
        runtimePhase.value = info.data.runtimePhase;
      }

      await refreshSnapshot();
    } catch (error) {
      bridgeSyncState.value = "error";
      bridgeError.value = publicError(error);
      devicesStore.markError();
      transfersStore.markError();
    }
  }

  async function refreshSnapshot(): Promise<void> {
    const snapshot = await getSnapshot();
    if (snapshot.data) {
      runtimePhase.value = snapshot.data.runtimePhase;
      productState.value = snapshot.data.productState;
    }
    lastSnapshotAt.value = snapshot.observedAt ?? snapshot.generatedAt;
    bridgeSyncState.value = snapshot.status === "success" ? "ready" : snapshot.status;
    bridgeError.value = undefined;
    resyncPending.value = false;
    devicesStore.applyBridgeStatus(snapshot.status);
    transfersStore.applyBridgeStatus(snapshot.status);
  }

  async function requestResync(): Promise<void> {
    if (resyncPromise) {
      return resyncPromise;
    }

    resyncPending.value = true;
    resyncPromise = refreshSnapshot()
      .catch((error) => {
        bridgeSyncState.value = "error";
        bridgeError.value = publicError(error);
        devicesStore.markError();
        transfersStore.markError();
      })
      .finally(() => {
        resyncPromise = undefined;
      });
    return resyncPromise;
  }

  async function dispose(): Promise<void> {
    await Promise.all([stopStatusListener?.(), stopDomainListener?.()]);
    stopStatusListener = undefined;
    stopDomainListener = undefined;
    await disposeBridge();
    bootstrapPromise = undefined;
  }

  return {
    version,
    bridgeState,
    bridgeMessage,
    bridgeSyncState,
    bridgeMode,
    runtimePhase,
    productState,
    lastSnapshotAt,
    lastEventAt,
    domainEventCount,
    resyncPending,
    bridgeError,
    initialize,
    refreshSnapshot,
    requestResync,
    dispose,
    testBridge,
  };
});

function publicError(error: unknown): string {
  return isBridgeError(error) ? `${error.code} · ${error.messageKey}` : "bridge.requestFailed";
}
