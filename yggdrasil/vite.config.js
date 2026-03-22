import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
// @ts-expect-error Node built-in
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [sveltekit()],
  clearScreen: false,
  resolve: {
    alias: {
      "$hlidskjalf": path.resolve("../hlidskjalf/src/lib"),
      "$svalinn": path.resolve("../svalinn/src/lib"),
      "$kvasir": path.resolve("../kvasir/src/lib"),
      "$ratatoskr": path.resolve("../ratatoskr/src/lib"),
    },
  },
  server: {
    port: 1460,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? { protocol: "ws", host, port: 1461 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
