<script setup lang="ts">
import { Monitor, Smartphone } from "@lucide/vue";
import { RouterLink } from "vue-router";

import { useDevicesStore } from "@/stores/devices";
import type { Device } from "@/types";

const devicesStore = useDevicesStore();
const iconFor = (device: Device) => (device.platform === "linux" ? Monitor : Smartphone);
</script>

<template>
  <div class="divide-y divide-border rounded-panel border border-border bg-surface">
    <RouterLink
      v-for="device in devicesStore.devices"
      :key="device.id"
      :to="`/device/${device.id}`"
      class="flex items-center gap-3 px-4 py-3.5 transition-colors hover:bg-surface-hover"
    >
      <span class="grid size-9 shrink-0 place-items-center rounded-control border border-border bg-background text-muted">
        <component :is="iconFor(device)" :size="17" :stroke-width="1.7" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-sm font-medium text-foreground">{{ device.name }}</span>
        <span class="mt-0.5 block text-xs text-muted">{{ device.online ? "Disponível agora" : `Visto ${device.lastSeen}` }}</span>
      </span>
      <span class="flex items-center gap-2 text-xs text-muted">
        <span :class="['size-1.5 rounded-full', device.online ? 'bg-success' : 'bg-muted']" />
        {{ device.online ? "online" : "offline" }}
      </span>
    </RouterLink>
  </div>
</template>
