// Plugins.
import vue from "@vitejs/plugin-vue";
import vuetify, { transformAssetUrls } from "vite-plugin-vuetify";
import ViteFonts from "unplugin-fonts/vite";

// Utilities.
import { defineConfig } from "vite";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  base: "/magika/",
  plugins: [
    vue({ template: { transformAssetUrls } }),
    vuetify({
      autoImport: true,
      styles: {
        configFile: "assets/custom.scss",
      },
    }),
    ViteFonts({
      google: {
        families: [
          {
            name: "Google Sans Text",
            styles: "wght@100;300;400;500;700;900",
          },
          {
            name: "Google Sans",
            styles: "wght@100;300;400;500;700;900",
          },
        ],
      },
    }),
  ],
  define: { "process.env": {} },
  // https://github.com/vitejs/vite/issues/2433
  build: {
    sourcemap: false,
  },
  resolve: {
    assetsInclude: ["**/*.md", "**/*.html"],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
    extensions: [".js", ".json", ".jsx", ".mjs", ".ts", ".tsx", ".vue"],
  },
});
