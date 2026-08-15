<script setup lang="ts">
import { History } from "@lucide/vue";

import Badge from "@/components/ui/badge/Badge.vue";
</script>

<template>
  <section class="mx-auto max-w-5xl">
    <div class="border-b border-border pb-7">
      <Badge variant="muted">estrutura inicial</Badge>
      <h1 class="mt-4 text-3xl font-semibold tracking-[-0.04em] text-foreground">Histórico</h1>
      <p class="mt-3 max-w-2xl text-sm leading-6 text-muted">A navegação e o espaço de leitura estão definidos. Persistência e histórico de eventos entram em uma etapa posterior.</p>
    </div>
    <div class="mt-7 flex min-h-56 flex-col items-center justify-center rounded-panel border border-dashed border-border bg-surface px-6 text-center">
      <span class="grid size-11 place-items-center rounded-control border border-border bg-background text-muted"><History :size="19" /></span>
      <p class="mt-4 text-sm font-medium text-foreground">O histórico aparecerá aqui</p>
      <p class="mt-2 max-w-sm text-xs leading-5 text-muted">Por enquanto, não há armazenamento local nem eventos de rede conectados ao app.</p>
    </div>
  </section>
</template>
