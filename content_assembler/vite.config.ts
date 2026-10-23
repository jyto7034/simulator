import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath, URL } from "node:url";
import process from "node:process";
import { contentFilesPlugin } from "./server/content-files.ts";
import { compilerPlugin } from "./server/compiler.ts";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    react(),
    compilerPlugin(process.env.CONTENT_ASSEMBLER_COMPILER),
    tailwindcss(),
    contentFilesPlugin(fileURLToPath(new URL("./content", import.meta.url))),
  ],
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  // Match the modern WebView baseline required by Tailwind CSS v4.
  build: { target: ["chrome111", "safari16.4"] },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**", "**/content/**"],
    },
  },
}));
