import { computed, ref } from "vue";
import { defineStore } from "pinia";

import type { BridgeReadStatus, CollectionSource, CollectionSyncState, DeviceListItem, MockDevice } from "@/types";

const mockDevices: MockDevice[] = [
  {
    id: "pulse-desktop",
    name: "Pulse Desktop",
    platform: "linux",
    online: true,
    lastSeen: "agora",
  },
  {
    id: "studio-phone",
    name: "Studio Phone",
    platform: "android",
    online: true,
    lastSeen: "agora",
  },
  {
    id: "travel-laptop",
    name: "Travel Laptop",
    platform: "linux",
    online: false,
    lastSeen: "há 18 min",
  },
];

export const useDevicesStore = defineStore("devices", () => {
  const developmentFixturesEnabled = import.meta.env.DEV;
  const devices = ref<DeviceListItem[]>(developmentFixturesEnabled ? mockDevices.map(toListItem) : []);
  const source = ref<CollectionSource>(developmentFixturesEnabled ? "development-fixture" : "empty");
  const syncState = ref<CollectionSyncState>(developmentFixturesEnabled ? "ready" : "offline");
  const selectedDeviceId = ref<string | undefined>(developmentFixturesEnabled ? "studio-phone" : undefined);
  const selectedDevice = computed(() => devices.value.find((device) => device.id === selectedDeviceId.value));
  const onlineDevices = computed(() => devices.value.filter((device) => device.presence === "online"));
  const isDemo = computed(() => source.value === "development-fixture");
  const sourceLabel = computed(() => (isDemo.value ? "fixture de desenvolvimento" : "sem dados conectados"));

  function selectDevice(id: string) {
    if (devices.value.some((device) => device.id === id)) {
      selectedDeviceId.value = id;
    }
  }

  function applyBridgeStatus(status: BridgeReadStatus) {
    syncState.value = status === "success" ? "ready" : status;
    if (!developmentFixturesEnabled) {
      source.value = "empty";
      devices.value = [];
      selectedDeviceId.value = undefined;
    }
  }

  function markError() {
    syncState.value = "error";
    if (!developmentFixturesEnabled) {
      source.value = "empty";
      devices.value = [];
      selectedDeviceId.value = undefined;
    }
  }

  return {
    devices,
    selectedDeviceId,
    selectedDevice,
    onlineDevices,
    source,
    sourceLabel,
    syncState,
    isDemo,
    selectDevice,
    applyBridgeStatus,
    markError,
  };
});

function toListItem(device: MockDevice): DeviceListItem {
  return {
    id: device.id,
    name: device.name,
    platform: device.platform,
    presence: device.online ? "online" : "offline",
    lastSeen: device.lastSeen,
  };
}
