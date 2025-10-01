import { test, expect } from "@playwright/test";

test("can run Magika", async ({ page }) => {
  await page.goto("http://localhost:8000");
  await page.waitForSelector("div.status");
  await expect(page.locator("div.status")).toHaveText("ok");
  await page.waitForSelector("div.label");
  await expect(page.locator("div.label")).toHaveText("python");
});
