import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Стандартный конфиг для связки Vite + Tauri:
// https://v2.tauri.app/start/frontend/vite/
export default defineConfig(async () => ({
  plugins: [react()],

  // Порт должен совпадать с devUrl в src-tauri/tauri.conf.json.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Не перезапускать Vite при пересборке Rust-части.
      ignored: ["**/src-tauri/**"],
    },
  },
}));
