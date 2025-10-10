// Plugins.
import vue from "@vitejs/plugin-vue";

// Utilities.
import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  base: "/magika/",
  plugins: [
    vue(),
    {
      name: 'redirect-all',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (req.url && req.url.includes('/magika/model')) {
            next();
            return;
          }
          res.writeHead(301, { Location: 'https://securityresearch.google/magika/' });
          res.end();
        });
      },
      apply: 'serve',
    },
  ],
  resolve: {
    assetsInclude: ["**/*.md", "**/*.html"],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
    extensions: [".js", ".json", ".jsx", ".mjs", ".ts", ".tsx", ".vue"],
  },
});
