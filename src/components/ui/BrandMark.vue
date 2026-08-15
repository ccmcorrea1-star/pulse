<script setup lang="ts">
import { siLinux } from "simple-icons";

const props = withDefaults(
  defineProps<{
    size?: number;
  }>(),
  { size: 18 },
);
</script>

<template>
  <svg
    :width="props.size"
    :height="props.size"
    viewBox="0 0 24 24"
    role="img"
    :aria-label="siLinux.title"
    fill="currentColor"
  >
    <path :d="siLinux.path" />
  </svg>
</template>
