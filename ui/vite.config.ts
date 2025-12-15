import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  
  // Vite options tailored for Tauri development
  clearScreen: false,
  
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ["**/app/**"],
    },
  },
  
  // Env variables starting with VITE_ are exposed to the source code
  envPrefix: ["VITE_", "TAURI_"],
  
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    // Suppress chunk size warning - not an issue for Tauri desktop apps
    chunkSizeWarningLimit: 1200,
  },
});
