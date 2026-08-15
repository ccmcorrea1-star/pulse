<script setup lang="ts">
import { ArrowRight, Check, Code2, Layers3, TerminalSquare } from "@lucide/vue";
import { RouterLink } from "vue-router";

import DeviceList from "@/components/device/DeviceList.vue";
import TransferPreview from "@/components/transfer/TransferPreview.vue";
import Badge from "@/components/ui/badge/Badge.vue";
import { useDevicesStore } from "@/stores/devices";
import { useTransfersStore } from "@/stores/transfers";

const devicesStore = useDevicesStore();
const transfersStore = useTransfersStore();
</script>

<template>
  <div class="mx-auto max-w-6xl">
    <section class="border-b border-border pb-8">
      <div class="max-w-3xl">
        <div class="flex flex-wrap items-center gap-2">
          <Badge><span class="size-1.5 rounded-full bg-success" /> fundação inicial</Badge>
          <Badge variant="muted">mock local · sem networking</Badge>
        </div>
        <h1 class="mt-5 max-w-3xl text-3xl font-semibold tracking-[-0.04em] text-foreground sm:text-4xl">Um shell calmo para acompanhar seus dispositivos.</h1>
        <p class="mt-4 max-w-2xl text-base leading-7 text-muted">O Pulse começa com uma base navegável, tokens compartilhados e uma bridge mínima entre Vue e Rust. As capacidades de produto entram depois, em camadas separadas.</p>
      </div>
      <div class="mt-6 flex flex-wrap items-center gap-3">
        <RouterLink to="/settings" class="inline-flex h-10 items-center gap-2 rounded-control bg-accent px-4 text-sm font-medium text-background transition-colors hover:bg-accent-strong">
          Ver a bridge Tauri
          <ArrowRight :size="15" />
        </RouterLink>
        <span class="font-mono text-[11px] uppercase tracking-[0.12em] text-muted">Pulse v0.1 · desktop Linux</span>
      </div>
    </section>

    <section class="grid gap-5 border-b border-border py-7 md:grid-cols-3">
      <div class="md:col-span-2">
        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">Estado do shell</p>
            <h2 class="mt-1 text-lg font-semibold tracking-tight text-foreground">Camadas prontas para crescer</h2>
          </div>
          <Check :size="18" class="text-success" />
        </div>
        <div class="mt-5 grid gap-2 sm:grid-cols-3">
          <div class="rounded-panel border border-border bg-surface p-4">
            <Layers3 :size="17" class="text-accent" />
            <p class="mt-4 text-sm font-medium text-foreground">UI modular</p>
            <p class="mt-1 text-xs leading-5 text-muted">Vue, Tailwind e shadcn-vue em componentes pequenos.</p>
          </div>
          <div class="rounded-panel border border-border bg-surface p-4">
            <Code2 :size="17" class="text-info" />
            <p class="mt-4 text-sm font-medium text-foreground">Estado local</p>
            <p class="mt-1 text-xs leading-5 text-muted">Stores Pinia mínimas para app, dispositivos e transferências.</p>
          </div>
          <div class="rounded-panel border border-border bg-surface p-4">
            <TerminalSquare :size="17" class="text-warning" />
            <p class="mt-4 text-sm font-medium text-foreground">Tauri 2</p>
            <p class="mt-1 text-xs leading-5 text-muted">Command Rust isolado, pronto para novas capacidades.</p>
          </div>
        </div>
      </div>
      <div class="rounded-panel border border-border bg-surface p-5">
        <p class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">Próximas fronteiras</p>
        <ul class="mt-4 space-y-3 text-sm text-muted">
          <li class="flex gap-3"><span class="mt-2 size-1.5 shrink-0 rounded-full bg-border" />Discovery e pairing</li>
          <li class="flex gap-3"><span class="mt-2 size-1.5 shrink-0 rounded-full bg-border" />Protocolo de transferência</li>
          <li class="flex gap-3"><span class="mt-2 size-1.5 shrink-0 rounded-full bg-border" />Clipboard, mídia e notificações</li>
        </ul>
        <p class="mt-5 border-t border-border pt-4 text-xs leading-5 text-muted">Nenhum desses fluxos está ativo nesta etapa.</p>
      </div>
    </section>

    <section class="grid gap-5 py-7 lg:grid-cols-[1.1fr_0.9fr]">
      <div>
        <div class="mb-4 flex items-end justify-between gap-4">
          <div>
            <p class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">Dados demonstrativos</p>
            <h2 class="mt-1 text-lg font-semibold tracking-tight text-foreground">Dispositivos</h2>
          </div>
          <span class="text-xs text-muted">{{ devicesStore.onlineDevices.length }} online</span>
        </div>
        <DeviceList />
      </div>
      <div>
        <div class="mb-4 flex items-end justify-between gap-4">
          <div>
            <p class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">Estrutura demonstrativa</p>
            <h2 class="mt-1 text-lg font-semibold tracking-tight text-foreground">Transferências</h2>
          </div>
          <span class="text-xs text-muted">{{ transfersStore.activeTransfers.length }} mock</span>
        </div>
        <TransferPreview />
      </div>
    </section>
  </div>
</template>
