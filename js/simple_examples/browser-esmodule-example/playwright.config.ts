import { defineConfig } from "@playwright/test";

export default defineConfig({
  // Run your local dev server before starting the tests
  webServer: {
    command: "npm run server",
    url: "http://localhost:8000",
    reuseExistingServer: !process.env.CI,
    stdout: "ignore",
    stderr: "pipe",
  },
});
