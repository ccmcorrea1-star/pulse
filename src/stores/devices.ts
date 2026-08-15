import { computed, ref } from "vue";
import { defineStore } from "pinia";

import type { MockDevice } from "@/types";

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
  const devices = ref<MockDevice[]>(mockDevices);
  const selectedDeviceId = ref("studio-phone");
  const selectedDevice = computed(() => devices.value.find((device) => device.id === selectedDeviceId.value));
  const onlineDevices = computed(() => devices.value.filter((device) => device.online));

  function selectDevice(id: string) {
    if (devices.value.some((device) => device.id === id)) {
      selectedDeviceId.value = id;
    }
  }

  return { devices, selectedDeviceId, selectedDevice, onlineDevices, selectDevice };
});
