// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

import svelte from "@astrojs/svelte";

import tailwindcss from "@tailwindcss/vite";

import node from "@astrojs/node";

// https://astro.build/config
export default defineConfig({
  base: "/magika",
  server: { port: 8080 },
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
      customCss: ["./src/styles/global.css"],
      sidebar: [
        {
          label: "Introduction",
          items: [
            { label: "Overview", slug: "introduction/overview" },
            { label: "Web Demo", slug: "demo/magika-demo" },
          ],
        },
        {
          label: "Getting Started",
          items: [
            { label: "Installation", slug: "getting-started/installation" },
            { label: "Quick Start", slug: "getting-started/quick-start" },
          ],
        },
        {
          label: "Core Concepts",
          items: [
            {
              label: "How Magika Works",
              slug: "core-concepts/how-magika-works",
            },
            {
              label: "Models & Content Types",
              slug: "core-concepts/models-and-content-types",
            },
            {
              label: "Prediction Modes",
              slug: "core-concepts/prediction-modes",
            },
            {
              label: "Understanding the Output",
              slug: "core-concepts/understanding-the-output",
            },
          ],
        },
        {
          label: "CLI & Bindings",
          items: [
            {
              label: "Overview",
              slug: "cli-and-bindings/overview",
            },
            {
              label: "Command Line Interface (CLI)",
              slug: "cli-and-bindings/cli",
            },
            {
              label: "Python `Magika` module",
              slug: "cli-and-bindings/python",
            },
            {
              label: "Rust bindings",
              slug: "cli-and-bindings/rust",
            },
            {
              label: "JavaScript bindings",
              slug: "cli-and-bindings/js",
            },
            {
              label: "Other bindings",
              slug: "cli-and-bindings/other-bindings",
            },
          ],
        },
        {
          label: "Contributing",
          items: [
            {
              label: "Known Limitations",
              slug: "contributing/known-limitations",
            },
            {
              label: "How to Contribute",
              slug: "contributing/how-to-contribute",
            },
            {
              label: "Reporting Security Vulnerabilities",
              slug: "contributing/reporting-security-vulnerabilities",
            },
            {
              label: "Creating New Bindings",
              slug: "contributing/creating-new-bindings",
            },
          ],
        },
        {
          label: "Additional Resources",
          items: [
            {
              label: "FAQ",
              slug: "additional-resources/faq",
            },
            {
              label: "Research Papers and Citation",
              slug: "additional-resources/research-papers-and-citation",
            },
            {
              label: "Related Blog Posts",
              slug: "additional-resources/related-blog-posts",
            },
            {
              label: "Changelog",
              slug: "additional-resources/changelog",
            },
            {
              label: "License",
              slug: "additional-resources/license",
            },
            {
              label: "Disclaimer",
              slug: "additional-resources/disclaimer",
            },
          ],
        },
      ],
    }),
    svelte(),
  ],
  redirects: {
    "/": "/introduction/overview",
    "/core-concepts": "/core-concepts/how-magika-works",
  },
  vite: {
    plugins: [tailwindcss()],
    ssr: {
      noExternal: "cookie",
    },
  },
  adapter: node({
    mode: "standalone",
  }),
});
