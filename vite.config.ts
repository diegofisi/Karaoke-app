import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Configuración para Tauri 2: puerto fijo (tauri.conf.json -> build.devUrl),
// sin limpiar la consola para no tapar los errores de Rust.
const host = process.env.TAURI_DEV_HOST;
const depuracion = Boolean(process.env.TAURI_ENV_DEBUG);

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/crates/**", "**/fabrica/**", "**/eval/**", "**/target/**", "**/venv/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // WebView2 en Windows; WebKit en macOS y Linux.
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome111" : "safari16",
    minify: !depuracion,
    sourcemap: depuracion,
  },
});
