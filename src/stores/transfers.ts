import { computed, ref } from "vue";
import { defineStore } from "pinia";

import type { BridgeReadStatus, CollectionSource, CollectionSyncState, MockTransfer, TransferListItem } from "@/types";

const mockTransfers: MockTransfer[] = [
  {
    id: "transfer-1",
    name: "brief-pulse.pdf",
    type: "PDF · 2,4 MB",
    status: "in-progress",
    progress: 64,
    deviceName: "Studio Phone",
    updatedAt: "agora",
  },
  {
    id: "transfer-2",
    name: "referencias-ui.zip",
    type: "ZIP · 18 MB",
    status: "queued",
    progress: 0,
    deviceName: "Travel Laptop",
    updatedAt: "há 4 min",
  },
];

export const useTransfersStore = defineStore("transfers", () => {
  const developmentFixturesEnabled = import.meta.env.DEV;
  const transfers = ref<TransferListItem[]>(developmentFixturesEnabled ? mockTransfers.map(toListItem) : []);
  const source = ref<CollectionSource>(developmentFixturesEnabled ? "development-fixture" : "empty");
  const syncState = ref<CollectionSyncState>(developmentFixturesEnabled ? "ready" : "offline");
  const activeTransfers = computed(() => transfers.value.filter((transfer) => transfer.status !== "complete"));
  const isDemo = computed(() => source.value === "development-fixture");
  const sourceLabel = computed(() => (isDemo.value ? "fixture de desenvolvimento" : "sem dados conectados"));

  function applyBridgeStatus(status: BridgeReadStatus) {
    syncState.value = status === "success" ? "ready" : status;
    if (!developmentFixturesEnabled) {
      source.value = "empty";
      transfers.value = [];
    }
  }

  function markError() {
    syncState.value = "error";
    if (!developmentFixturesEnabled) {
      source.value = "empty";
      transfers.value = [];
    }
  }

  return { transfers, activeTransfers, source, sourceLabel, syncState, isDemo, applyBridgeStatus, markError };
});

function toListItem(transfer: MockTransfer): TransferListItem {
  return {
    id: transfer.id,
    name: transfer.name,
    type: transfer.type,
    status: transfer.status === "in-progress" ? "active" : transfer.status === "complete" ? "complete" : "queued",
    progress: transfer.progress,
    deviceName: transfer.deviceName,
    updatedAt: transfer.updatedAt,
  };
}
