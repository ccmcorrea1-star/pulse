import { computed, ref } from "vue";
import { defineStore } from "pinia";

import type { Transfer } from "@/types";

const mockTransfers: Transfer[] = [
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
  const transfers = ref<Transfer[]>(mockTransfers);
  const activeTransfers = computed(() => transfers.value.filter((transfer) => transfer.status !== "complete"));

  return { transfers, activeTransfers };
});
