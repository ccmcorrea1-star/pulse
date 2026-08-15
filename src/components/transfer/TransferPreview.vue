<script setup lang="ts">
import { ArrowUpRight, Clock3 } from "@lucide/vue";

import { useTransfersStore } from "@/stores/transfers";

const transfersStore = useTransfersStore();
</script>

<template>
  <div class="divide-y divide-border rounded-panel border border-border bg-surface">
    <div v-for="transfer in transfersStore.activeTransfers" :key="transfer.id" class="px-4 py-4">
      <div class="flex items-start gap-3">
        <span class="grid size-8 shrink-0 place-items-center rounded-control bg-info/10 text-info">
          <ArrowUpRight :size="15" :stroke-width="1.8" />
        </span>
        <div class="min-w-0 flex-1">
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <p class="truncate text-sm font-medium text-foreground">{{ transfer.name }}</p>
              <p class="mt-0.5 text-xs text-muted">{{ transfer.type }} · {{ transfer.deviceName }}</p>
            </div>
            <span class="shrink-0 font-mono text-xs text-muted">{{ transfer.progress }}%</span>
          </div>
          <div class="mt-3 h-1 overflow-hidden rounded-full bg-background" aria-hidden="true">
            <div class="h-full rounded-full bg-accent" :style="{ width: `${Math.max(transfer.progress, 3)}%` }" />
          </div>
          <div class="mt-2 flex items-center gap-1.5 text-[11px] text-muted">
            <Clock3 :size="12" />
            <span>{{ transfer.status === "queued" ? "aguardando estrutura de fila" : `estado mock · atualizado ${transfer.updatedAt}` }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
