import { defineStore } from "pinia";
import { ref } from "vue";

import type { BridgeState } from "@/types";
import { useRustBridge } from "@/composables/useRustBridge";

export const useAppStore = defineStore("app", () => {
  const version = "0.1.0";
  const bridgeState = ref<BridgeState>("idle");
  const bridgeMessage = ref("A bridge ainda não foi testada.");
  const { greet } = useRustBridge();

  async function testBridge() {
    bridgeState.value = "loading";

    try {
      const result = await greet("Pulse");
      bridgeMessage.value = result.isDemo
        ? `${result.message} Execute no Tauri para validar o Rust.`
        : result.message;
      bridgeState.value = "success";
    } catch (error) {
      bridgeState.value = "error";
      bridgeMessage.value = error instanceof Error ? error.message : "Não foi possível chamar o Rust.";
    }
  }

  return { version, bridgeState, bridgeMessage, testBridge };
});
