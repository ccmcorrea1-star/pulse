<script setup lang="ts">
import { RouterLink } from "vue-router";
import { Network, Settings2 } from "@lucide/vue";
</script>

<template>
  <main class="pulse-content-scroll pulse-scrollbar">
    <header class="flex min-h-[86px] items-center justify-between gap-5 border-b border-border px-6 py-5 sm:px-8">
      <div class="min-w-0">
        <div class="flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.16em] text-muted">
          <Network :size="13" class="text-accent" />
          <span>rede local</span>
          <span class="text-border">/</span>
          <span>ambiente de base</span>
        </div>
        <p class="mt-1 truncate text-sm text-muted">Fundação desktop do Pulse</p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <span class="hidden items-center gap-2 rounded-full border border-border bg-surface px-3 py-1.5 text-xs text-muted sm:flex">
          <span class="size-1.5 rounded-full bg-success" />
          shell pronto
        </span>
        <RouterLink
          to="/settings"
          aria-label="Abrir configurações"
          class="grid size-9 place-items-center rounded-control border border-border bg-surface text-muted transition-colors hover:border-muted hover:text-foreground"
        >
          <Settings2 :size="16" :stroke-width="1.8" />
        </RouterLink>
      </div>
    </header>
    <div class="px-6 py-8 sm:px-8 lg:px-10">
      <RouterView />
    </div>
  </main>
</template>
