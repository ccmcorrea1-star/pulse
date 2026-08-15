import { invoke } from "@tauri-apps/api/core";

export function useRustBridge() {
  const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

  async function greet(name: string) {
    if (!isTauri()) {
      return {
        message: `Prévia web ativa para ${name}.`,
        isDemo: true,
      };
    }

    return {
      message: await invoke<string>("greet", { name }),
      isDemo: false,
    };
  }

  return { greet, isTauri };
}
