<script setup lang="ts">
import { ArrowLeft, Monitor, Smartphone } from "@lucide/vue";
import { computed } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";

import Badge from "@/components/ui/badge/Badge.vue";
import { useDevicesStore } from "@/stores/devices";
import type { DeviceListItem } from "@/types";

const route = useRoute();
const devicesStore = useDevicesStore();
const device = computed(() => devicesStore.devices.find((item) => item.id === route.params.id));
const tabs = [
  { label: "Visão geral", name: "device-overview" },
  { label: "Arquivos", name: "device-files" },
  { label: "Clipboard", name: "device-clipboard" },
  { label: "Mídia", name: "device-media" },
  { label: "Controle", name: "device-control" },
];
const iconFor = (currentDevice: DeviceListItem) => (currentDevice.platform === "linux" ? Monitor : Smartphone);
const isOnline = (currentDevice: DeviceListItem) => currentDevice.presence === "online";
const presenceLabel = (currentDevice: DeviceListItem) => {
  if (currentDevice.presence === "online") return "online";
  if (currentDevice.presence === "stale") return "desatualizado";
  return "offline";
};
</script>

<template>
  <section v-if="device" class="mx-auto max-w-6xl">
    <RouterLink to="/" class="inline-flex items-center gap-2 text-xs text-muted transition-colors hover:text-foreground"><ArrowLeft :size="14" /> voltar ao início</RouterLink>
    <div class="mt-5 flex flex-wrap items-start justify-between gap-5 border-b border-border pb-6">
      <div class="flex items-center gap-4">
        <span class="grid size-12 place-items-center rounded-panel border border-border bg-surface text-muted"><component :is="iconFor(device)" :size="22" :stroke-width="1.6" /></span>
        <div>
          <div class="flex flex-wrap items-center gap-2">
            <h1 class="text-2xl font-semibold tracking-[-0.03em] text-foreground">{{ device.name }}</h1>
            <Badge :variant="isOnline(device) ? 'default' : 'muted'"><span :class="['size-1.5 rounded-full', isOnline(device) ? 'bg-success' : device.presence === 'stale' ? 'bg-warning' : 'bg-muted']" />{{ presenceLabel(device) }}</Badge>
          </div>
          <p class="mt-1 text-sm text-muted">Rota preparada para módulos do dispositivo · {{ devicesStore.isDemo ? "fixture de desenvolvimento" : "estado não configurado" }}</p>
        </div>
      </div>
    </div>
    <nav aria-label="Seções do dispositivo" class="pulse-scrollbar flex gap-1 overflow-x-auto border-b border-border py-3">
      <RouterLink
        v-for="tab in tabs"
        :key="tab.name"
        :to="{ name: tab.name, params: { id: device.id } }"
        :class="['whitespace-nowrap rounded-control px-3 py-2 text-xs transition-colors', route.name === tab.name ? 'bg-surface-raised text-accent-strong' : 'text-muted hover:bg-surface-hover hover:text-foreground']"
      >{{ tab.label }}</RouterLink>
    </nav>
    <div class="pt-7">
      <RouterView />
    </div>
  </section>
  <section v-else class="mx-auto max-w-xl rounded-panel border border-border bg-surface p-8">
    <h1 class="text-lg font-semibold text-foreground">Dispositivo não encontrado</h1>
    <p class="mt-2 text-sm text-muted">O identificador informado não existe na fonte de dispositivos disponível.</p>
  </section>
</template>
