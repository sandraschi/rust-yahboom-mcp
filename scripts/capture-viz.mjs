import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire("D:/Dev/repos/yahboom-mcp/webapp/package.json");
const { chromium } = require("playwright");
const outDir = "D:/Dev/repos/rust-yahboom-mcp/docs/parity/legacy";

const browser = await chromium.launch({
  headless: false,
  executablePath: "C:/Users/sandr/AppData/Local/ms-playwright/chromium-1248/chrome-win64/chrome.exe",
  args: ["--use-gl=angle", "--enable-webgl", "--ignore-gpu-blocklist"],
});
const page = await browser.newPage({
  viewport: { width: 1280, height: 860 },
  colorScheme: "dark",
});
await page.route("**/*.stl", () => new Promise(() => {}));
await page.goto("http://127.0.0.1:10893/viz", { waitUntil: "domcontentloaded", timeout: 30000 });
await page.waitForFunction(() => document.body.innerText.includes("3D Visualization"), { timeout: 8000 });
await page.waitForTimeout(400);
await page.screenshot({ path: path.join(outDir, "viz.png") });
const early = await page.evaluate(() => document.body.innerText.includes("SUBSYSTEM FAILURE"));
process.stdout.write("SHELL_FAIL " + String(early) + "\n");
await browser.close();
