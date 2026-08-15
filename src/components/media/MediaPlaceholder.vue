<script setup lang="ts">
import { Image } from "@lucide/vue";
</script>

<template>
  <div class="flex min-h-44 flex-col items-center justify-center rounded-panel border border-dashed border-border bg-surface px-6 text-center">
    <span class="grid size-10 place-items-center rounded-control border border-border bg-background text-muted">
      <Image :size="18" :stroke-width="1.6" />
    </span>
    <p class="mt-3 text-sm font-medium text-foreground">Mídia entra na próxima etapa</p>
    <p class="mt-1 max-w-sm text-xs leading-5 text-muted">Este espaço já tem rota e componente próprios, sem iniciar a implementação de networking ou controle.</p>
  </div>
</template>
