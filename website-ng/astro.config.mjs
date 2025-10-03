// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

import svelte from "@astrojs/svelte";

import tailwindcss from "@tailwindcss/vite";

// https://astro.build/config
export default defineConfig({
  redirects: {
    "/": "/demo/magika-demo/",
  },

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
      customCss: [
        './src/styles/global.css',
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
            { label: "Magika Demo", slug: "demo/magika-demo" },
          ],
        },
        {
          label: "Reference",
          autogenerate: { directory: "reference" },
        },
      ],

    }),
    svelte(),

  ],
  vite: {
    plugins: [tailwindcss()],

    // The following is necessary to fix the SSR issues with TFJS.
    define: {
      global: 'globalThis',
      process: { env: {} },
    },
    optimizeDeps: {
      include: ['buffer', 'stream-browserify', 'http-browserify', 'url-browserify', 'util', 'events']
    },
    resolve: {
      alias: {
        'whatwg-url': 'whatwg-url/lib/public-api.js',
        stream: 'stream-browserify',
        util: 'util',
        url: 'url-browserify',
        buffer: 'buffer',
        http: 'http-browserify',
        events: 'events',
      }
    }
  }
});