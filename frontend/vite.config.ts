/// <reference types="vitest" />
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

const isTauri = process.env.TAURI_PLATFORM !== undefined;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      // Mock mode: specific aliases MUST come before the general "@" alias,
      // otherwise "@" matches "@/lib/tauri-api" as a prefix and shadows the mock.
      ...(isTauri
        ? {}
        : {
            "@/lib/tauri-api": path.resolve(__dirname, "./src/lib/mock/index.ts"),
            "@tauri-apps/api/event": path.resolve(__dirname, "./src/lib/mock/events.ts"),
            "@tauri-apps/plugin-dialog": path.resolve(__dirname, "./src/lib/mock/dialog.ts"),
          }),
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "esnext",
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
});
