import { mergeConfig, defineConfig } from "vitest/config";

import viteConfig from "./vite.config";

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      include: ["tests/**/*.test.ts"],
      environment: "node",
      clearMocks: true,
      restoreMocks: true,
      unstubGlobals: true,
    },
  }),
);
