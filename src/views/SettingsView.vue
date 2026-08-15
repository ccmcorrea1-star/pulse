<script setup lang="ts">
import { CheckCircle2, LoaderCircle, Settings2, XCircle } from "@lucide/vue";
import { computed } from "vue";

import Badge from "@/components/ui/badge/Badge.vue";
import Button from "@/components/ui/button/Button.vue";
import { useAppStore } from "@/stores/app";

const appStore = useAppStore();
const bridgeLabel = computed(() => {
  if (appStore.bridgeState === "loading") return "testando…";
  if (appStore.bridgeState === "success") return "bridge respondendo";
  if (appStore.bridgeState === "error") return "falha na bridge";
  return "não testada";
});
const syncLabel = computed(() => {
  if (appStore.bridgeSyncState === "loading") return "sincronizando";
  if (appStore.bridgeSyncState === "ready") return "estado observado";
  if (appStore.bridgeSyncState === "stale") return "estado desatualizado";
  if (appStore.bridgeSyncState === "offline") return "offline / não configurado";
  if (appStore.bridgeSyncState === "error") return "erro na sincronização";
  return "aguardando inicialização";
});
</script>

<template>
  <section class="mx-auto max-w-4xl">
    <div class="border-b border-border pb-7">
      <Badge variant="muted">configuração do shell</Badge>
      <h1 class="mt-4 text-3xl font-semibold tracking-[-0.04em] text-foreground">Configurações</h1>
      <p class="mt-3 max-w-2xl text-sm leading-6 text-muted">Um ponto inicial para preferências do app. Plugins oficiais e persistência serão adicionados quando houver um fluxo para suportá-los.</p>
    </div>

    <div class="mt-7 grid gap-5 lg:grid-cols-[1.2fr_0.8fr]">
      <section class="rounded-panel border border-border bg-surface p-5">
        <div class="flex items-start gap-3">
          <span class="grid size-9 place-items-center rounded-control border border-border bg-background text-accent"><Settings2 :size="17" /></span>
          <div>
            <h2 class="text-sm font-semibold text-foreground">Comunicação Vue ↔ Rust</h2>
            <p class="mt-1 text-xs leading-5 text-muted">O command <code>greet</code> continua como smoke test. A leitura inicial da bridge é {{ syncLabel }} e ainda não há estado de produto.</p>
          </div>
        </div>
        <div class="mt-5 flex flex-wrap items-center gap-3">
          <Button :disabled="appStore.bridgeState === 'loading'" size="sm" @click="appStore.testBridge">
            <LoaderCircle v-if="appStore.bridgeState === 'loading'" :size="14" class="animate-spin" />
            Testar command greet
          </Button>
          <span class="flex items-center gap-1.5 text-xs text-muted">
            <CheckCircle2 v-if="appStore.bridgeState === 'success'" :size="14" class="text-success" />
            <XCircle v-else-if="appStore.bridgeState === 'error'" :size="14" class="text-destructive" />
            <span :class="appStore.bridgeState === 'success' ? 'text-success' : appStore.bridgeState === 'error' ? 'text-destructive' : 'text-muted'">{{ bridgeLabel }}</span>
          </span>
        </div>
        <p class="mt-4 rounded-control border border-border bg-background px-3 py-2.5 font-mono text-[11px] leading-5 text-muted">{{ appStore.bridgeMessage }}</p>
      </section>

      <section class="rounded-panel border border-border bg-surface p-5">
        <p class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">Base instalada</p>
        <dl class="mt-4 divide-y divide-border text-xs">
          <div class="flex justify-between gap-4 py-3"><dt class="text-muted">Versão</dt><dd class="font-mono text-foreground">{{ appStore.version }}</dd></div>
          <div class="flex justify-between gap-4 py-3"><dt class="text-muted">Interface</dt><dd class="text-foreground">Vue 3 + Vite</dd></div>
          <div class="flex justify-between gap-4 py-3"><dt class="text-muted">Tema</dt><dd class="text-foreground">dark / compacto</dd></div>
          <div class="flex justify-between gap-4 py-3"><dt class="text-muted">Permissões</dt><dd class="text-foreground">core:default</dd></div>
          <div class="flex justify-between gap-4 py-3"><dt class="text-muted">Modo da bridge</dt><dd class="text-foreground">{{ appStore.bridgeMode ?? "não observado" }}</dd></div>
          <div class="flex justify-between gap-4 py-3"><dt class="text-muted">Runtime</dt><dd class="text-foreground">{{ appStore.runtimePhase ?? "não observado" }}</dd></div>
        </dl>
      </section>
    </div>
  </section>
</template>
