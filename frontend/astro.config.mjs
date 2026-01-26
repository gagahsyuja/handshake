import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";

import tailwindcss from "@tailwindcss/vite";

import node from "@astrojs/node";

export default defineConfig({
  integrations: [svelte()],

  server: {
    port: 3000,
  },

  vite: {
    plugins: [tailwindcss()],
  },

  adapter: node({
    mode: "standalone",
  }),
});
