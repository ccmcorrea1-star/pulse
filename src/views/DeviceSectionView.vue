<script setup lang="ts">
import { computed } from "vue";
import { RouterLink, useRoute } from "vue-router";

import MediaPlaceholder from "@/components/media/MediaPlaceholder.vue";
import Badge from "@/components/ui/badge/Badge.vue";

const route = useRoute();
const title = computed(() => String(route.meta.title ?? "Seção do dispositivo"));
const description = computed(() => String(route.meta.description ?? "Estrutura inicial."));
const isMedia = computed(() => route.name === "device-media");
</script>

<template>
  <div class="max-w-3xl">
    <Badge variant="muted">rota preparada</Badge>
    <h2 class="mt-4 text-2xl font-semibold tracking-[-0.03em] text-foreground">{{ title }}</h2>
    <p class="mt-3 max-w-2xl text-sm leading-6 text-muted">{{ description }}</p>
    <div v-if="isMedia" class="mt-7"><MediaPlaceholder /></div>
    <div v-else class="mt-7 rounded-panel border border-dashed border-border bg-surface p-7">
      <p class="text-sm font-medium text-foreground">Contrato de tela reservado</p>
      <p class="mt-2 max-w-xl text-sm leading-6 text-muted">A navegação já suporta esta área sem antecipar discovery, transferência de arquivos, Clipboard ou networking.</p>
      <RouterLink to="/settings" class="mt-5 inline-flex text-xs font-medium text-accent-strong hover:text-accent">Ver a configuração da base →</RouterLink>
    </div>
  </div>
</template>
