// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

import vue from "@astrojs/vue";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: "Magika",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/google/magika",
        },
      ],
      sidebar: [
        {
          label: "Introduction",
          items: [
            // Each item here is one entry in the navigation menu.
            { label: "Overview", slug: "introduction/overview" },
            { label: "Web Demo", slug: "demo/magika-demo" },
          ],
        },
        {
          label: "Demos (for debugging)",
          items: [
            // Each item here is one entry in the navigation menu.
            {
              label: "Counter Demo",
              slug: "demo/vuetify-counter-demo",
            },
            {
              label: "TextArea Demo",
              slug: "demo/vuetify-textarea-demo",
            },
            { label: "Magika Demo", slug: "demo/magika-demo" },
          ],
        },
        {
          label: "Reference",
          autogenerate: { directory: "reference" },
        },
      ],
    }),
    vue({
      appEntrypoint: "/src/plugins/vue-entry.js",
    }),
  ],
  redirects: {
    "/": "/introduction/overview",
  },
});
