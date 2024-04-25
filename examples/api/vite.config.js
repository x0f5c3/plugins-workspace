// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { defineConfig } from "vite";
import Unocss from "unocss/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { internalIpV4 } from "internal-ip";
import process from "process";

// https://vitejs.dev/config/
export default defineConfig(async () => {
  const host =
    process.env.TAURI_ENV_PLATFORM === "android" ||
    process.env.TAURI_ENV_PLATFORM === "ios"
      ? await internalIpV4()
      : "localhost";
  return {
    plugins: [Unocss(), svelte()],
    build: {
      rollupOptions: {
        output: {
          entryFileNames: `assets/[name].js`,
          chunkFileNames: `assets/[name].js`,
          assetFileNames: `assets/[name].[ext]`,
        },
      },
    },
    server: {
      host: "0.0.0.0",
      port: 5173,
      strictPort: true,
      hmr: {
        protocol: "ws",
        host,
        port: 5183,
      },
      fs: {
        allow: [".", "../../tooling/api/dist"],
      },
    },
  };
});
