<script setup lang="ts">
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const badgeVariants = cva("inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium", {
  variants: {
    variant: {
      default: "border-accent/25 bg-accent/10 text-accent-strong",
      muted: "border-border bg-surface-raised text-muted",
      warning: "border-warning/25 bg-warning/10 text-warning",
      info: "border-info/25 bg-info/10 text-info",
    },
  },
  defaultVariants: { variant: "default" },
});

const props = withDefaults(
  defineProps<{
    variant?: VariantProps<typeof badgeVariants>["variant"];
    class?: string;
  }>(),
  { variant: "default" },
);
</script>

<template>
  <span :class="cn(badgeVariants({ variant: props.variant }), props.class)"><slot /></span>
</template>
