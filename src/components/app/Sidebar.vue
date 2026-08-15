<script setup lang="ts">
import { Activity, History, Home, Monitor, Settings2, Smartphone } from "@lucide/vue";
import { computed, type Component } from "vue";
import { RouterLink, useRoute } from "vue-router";

import BrandMark from "@/components/ui/BrandMark.vue";
import { useDevicesStore } from "@/stores/devices";
import { useTransfersStore } from "@/stores/transfers";
import type { DeviceListItem } from "@/types";

const route = useRoute();
const devicesStore = useDevicesStore();
const transfersStore = useTransfersStore();

const mainNav: Array<{ label: string; to: string; icon: Component }> = [
  { label: "Início", to: "/", icon: Home },
  { label: "Transferências", to: "/transfers", icon: Activity },
  { label: "Histórico", to: "/history", icon: History },
];

const deviceIcon = (device: DeviceListItem) => (device.platform === "linux" ? Monitor : Smartphone);
const presenceClass = (device: DeviceListItem) => {
  if (device.presence === "online") return "bg-success";
  if (device.presence === "stale") return "bg-warning";
  return "bg-muted";
};
const isNavActive = (to: string) => (to === "/" ? route.path === "/" : route.path.startsWith(to));
const isDeviceActive = (id: string) => route.params.id === id;
const onlineCount = computed(() => devicesStore.onlineDevices.length);
</script>

<template>
  <aside class="pulse-sidebar flex min-h-screen flex-col border-r border-border bg-surface px-4 py-5">
    <div class="flex items-center gap-3 px-3">
      <span class="grid size-9 place-items-center rounded-panel border border-accent/25 bg-accent/10 text-accent">
        <BrandMark :size="19" />
      </span>
      <div class="min-w-0">
        <p class="truncate text-sm font-semibold tracking-tight text-foreground">Pulse</p>
        <p class="text-[11px] text-muted">rede local</p>
      </div>
    </div>

    <nav aria-label="Navegação principal" class="mt-8 space-y-1">
      <RouterLink
        v-for="item in mainNav"
        :key="item.to"
        :to="item.to"
        :class="[
          'group flex items-center gap-3 rounded-control px-3 py-2.5 text-sm transition-colors',
          isNavActive(item.to) ? 'bg-surface-raised text-foreground' : 'text-muted hover:bg-surface-hover hover:text-foreground',
        ]"
      >
        <component :is="item.icon" :size="17" :stroke-width="1.8" :class="isNavActive(item.to) ? 'text-accent' : 'text-muted'" />
        <span>{{ item.label }}</span>
        <span v-if="item.to === '/transfers' && transfersStore.activeTransfers.length" class="ml-auto rounded-full bg-warning/10 px-1.5 py-0.5 text-[10px] text-warning">{{ transfersStore.activeTransfers.length }}</span>
      </RouterLink>
    </nav>

    <div class="mt-8 min-h-0 flex-1">
      <div class="flex items-center justify-between px-3">
        <p class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">Dispositivos</p>
        <span class="text-[11px] text-success">{{ onlineCount }} online</span>
      </div>
      <div class="pulse-sidebar-device-list mt-3 space-y-1">
        <RouterLink
          v-for="device in devicesStore.devices"
          :key="device.id"
          :to="`/device/${device.id}`"
          :class="[
            'flex items-center gap-3 rounded-control px-3 py-2.5 text-sm transition-colors',
            isDeviceActive(device.id) ? 'bg-surface-raised text-foreground' : 'text-muted hover:bg-surface-hover hover:text-foreground',
          ]"
        >
          <span class="relative grid size-7 shrink-0 place-items-center rounded-control border border-border bg-background">
            <component :is="deviceIcon(device)" :size="15" :stroke-width="1.7" />
            <span :class="['absolute -right-0.5 -top-0.5 size-2 rounded-full border-2 border-surface', presenceClass(device)]" />
          </span>
          <span class="min-w-0 flex-1 truncate">{{ device.name }}</span>
        </RouterLink>
      </div>
    </div>

    <div class="mt-5 border-t border-border pt-4">
      <RouterLink
        to="/settings"
        :class="[
          'flex items-center gap-3 rounded-control px-3 py-2.5 text-sm transition-colors',
          isNavActive('/settings') ? 'bg-surface-raised text-foreground' : 'text-muted hover:bg-surface-hover hover:text-foreground',
        ]"
      >
        <Settings2 :size="17" :stroke-width="1.8" />
        <span>Configurações</span>
      </RouterLink>
      <p class="mt-4 px-3 font-mono text-[10px] uppercase tracking-[0.12em] text-muted/70">Pulse {{ devicesStore.devices.length ? "· base 0.1" : "" }}</p>
    </div>
  </aside>
</template>
