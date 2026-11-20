// SPDX-License-Identifier: GPL-3.0-or-later
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed dev-server port and serves the built `dist/` in release.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: { target: "es2022" },
});
