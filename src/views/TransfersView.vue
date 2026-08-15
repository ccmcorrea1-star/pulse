<script setup lang="ts">
import { ArrowUpRight, Pause, Plus } from "@lucide/vue";

import Badge from "@/components/ui/badge/Badge.vue";
import Button from "@/components/ui/button/Button.vue";
import { useTransfersStore } from "@/stores/transfers";

const transfersStore = useTransfersStore();
</script>

<template>
  <section class="mx-auto max-w-5xl">
    <div class="border-b border-border pb-7">
      <Badge variant="muted">estrutura inicial</Badge>
      <h1 class="mt-4 text-3xl font-semibold tracking-[-0.04em] text-foreground">Transferências</h1>
      <p class="mt-3 max-w-2xl text-sm leading-6 text-muted">A rota está pronta para receber a fila de transferências. O comportamento de produto ainda não foi implementado.</p>
    </div>
    <div class="mt-7 rounded-panel border border-dashed border-border bg-surface p-8 text-center">
      <span class="mx-auto grid size-11 place-items-center rounded-control border border-border bg-background text-info"><ArrowUpRight :size="19" /></span>
      <h2 class="mt-4 text-base font-medium text-foreground">Nenhuma transferência real nesta etapa</h2>
      <p class="mx-auto mt-2 max-w-md text-sm leading-6 text-muted">A fonte atual é {{ transfersStore.sourceLabel }}; a fila real ainda não foi conectada à bridge.</p>
      <div class="mt-5 flex flex-wrap justify-center gap-2">
        <Button variant="secondary" size="sm" disabled><Plus :size="14" /> Enviar conteúdo</Button>
        <Button variant="ghost" size="sm" disabled><Pause :size="14" /> Pausar fila</Button>
      </div>
    </div>
  </section>
</template>
