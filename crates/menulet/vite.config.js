import { defineConfig } from "vite";

// Vite serves src/ on port 1420 in dev (matches tauri.conf devUrl) and bundles
// it to ../dist for release. Without it the bare @tauri-apps/* imports in
// main.js cannot resolve in the webview.
export default defineConfig({
  root: "src",
  build: { outDir: "../dist", emptyOutDir: true },
  server: { port: 1420, strictPort: true },
  clearScreen: false,
});
