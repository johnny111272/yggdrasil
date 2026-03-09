import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import path from "path";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    alias: {
      "$hlidskjalf": path.resolve("../hlidskjalf/src/lib"),
      "$svalinn": path.resolve("../svalinn/src/lib"),
      "$kvasir": path.resolve("../kvasir/src/lib"),
      "$ratatoskr": path.resolve("../ratatoskr/src/lib"),
    },
  },
};

export default config;
