// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

import vue from "@astrojs/vue";
import { models } from "@tensorflow/tfjs-node";

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
            { label: "Overview", slug: "introduction/overview" },
            { label: "Web Demo", slug: "demo/web-demo" },
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
              slug: "introduction/overview",
            },
            {
              label: "Python bindings",
              slug: "introduction/overview",
            },
            {
              label: "Javascript bindings",
              slug: "introduction/overview",
            },
            {
              label: "Other bindings",
              slug: "introduction/overview",
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
          label: "Resources",
          items: [
            {
              label: "FAQ",
              slug: "introduction/overview",
            },
            {
              label: "Research Papers and Citation",
              slug: "introduction/overview",
            },
            {
              label: "Related Posts",
              slug: "introduction/overview",
            },
            {
              label: "Changelog",
              slug: "introduction/overview",
            },
            {
              label: "License & Disclaimer",
              slug: "introduction/overview",
            },
          ],
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
