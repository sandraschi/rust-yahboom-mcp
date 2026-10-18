import { createRequire } from "node:module";
import { mkdir } from "node:fs/promises";
import path from "node:path";

const require = createRequire("D:/Dev/repos/yahboom-mcp/webapp/package.json");
const { chromium } = require("playwright");

const outDir = "D:/Dev/repos/rust-yahboom-mcp/docs/parity/legacy";
const base = "http://127.0.0.1:10893";

const pages = [
  ["dashboard", "/dashboard"],
  ["mission-control", "/mission-control"],
  ["status", "/status"],
  ["analytics", "/analytics"],
  ["viz", "/viz"],
  ["map", "/map"],
  ["slam", "/slam"],
  ["lidar-map", "/lidar-map"],
  ["movement", "/movement"],
  ["sensors", "/sensors"],
  ["peripherals", "/peripherals"],
  ["audio", "/audio"],
  ["voice", "/voice"],
  ["voice-upgrade", "/voice-upgrade"],
  ["lidar-addon", "/lidar-addon"],
  ["chat", "/chat"],
  ["missions", "/missions"],
  ["llm", "/llm"],
  ["skills", "/skills"],
  ["workflows", "/workflows"],
  ["diagnostics", "/diagnostics"],
  ["inbox", "/inbox"],
  ["logs", "/logs"],
  ["tools", "/tools"],
  ["apps", "/apps"],
  ["settings", "/settings"],
  ["onboarding", "/onboarding"],
  ["help", "/help"],
];

await mkdir(outDir, { recursive: true });
const launchOpts = {
  executablePath: "C:/Users/sandr/AppData/Local/ms-playwright/chromium-1248/chrome-win64/chrome.exe",
  args: ["--use-gl=angle", "--enable-webgl", "--ignore-gpu-blocklist"],
};

async function shootScroll(page, name) {
  const metrics = await page.evaluate(() => {
    const el = document.querySelector("main div.overflow-y-auto");
    if (!el) return null;
    return { sh: el.scrollHeight, ch: el.clientHeight };
  });
  if (!metrics || metrics.sh <= metrics.ch + 24) return 0;
  let n = 0;
  let top = 0;
  const step = Math.max(240, metrics.ch - 80);
  while (top + metrics.ch < metrics.sh - 8) {
    top = Math.min(top + step, metrics.sh - metrics.ch);
    n += 1;
    await page.evaluate((y) => {
      const el = document.querySelector("main div.overflow-y-auto");
      if (el) el.scrollTop = y;
    }, top);
    await page.waitForTimeout(350);
    await page.screenshot({ path: path.join(outDir, name + "-scroll-" + String(n) + ".png") });
    if (top >= metrics.sh - metrics.ch) break;
  }
  return n;
}

async function shootPage(browser, name, route) {
  const page = await browser.newPage({
    viewport: { width: 1280, height: 860 },
    colorScheme: "dark",
  });
  const url = base + route;
  process.stdout.write("GET " + url + "\n");
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 30000 });
  const settle = name === "viz" ? 4000 : 1800;
  await page.waitForTimeout(settle);
  const failed = await page.evaluate(() => document.body.innerText.includes("SUBSYSTEM FAILURE"));
  if (failed) {
    await page.close();
    return false;
  }
  await page.screenshot({ path: path.join(outDir, name + ".png") });
  const extra = await shootScroll(page, name);
  process.stdout.write("OK " + name + " scroll=" + String(extra) + "\n");
  await page.close();
  return true;
}

const browser = await chromium.launch({ ...launchOpts, headless: true });
const retry = [];
for (const [name, route] of pages) {
  const ok = await shootPage(browser, name, route);
  if (!ok) retry.push([name, route]);
}
await browser.close();

if (retry.length > 0) {
  process.stdout.write("HEADED " + retry.map((row) => row[0]).join(",") + "\n");
  const headed = await chromium.launch({ ...launchOpts, headless: false });
  for (const [name, route] of retry) {
    const ok = await shootPage(headed, name, route);
    process.stdout.write("RETRY " + name + " " + String(ok) + "\n");
  }
  await headed.close();
}

await browser.close();
process.stdout.write("DONE " + String(pages.length) + "\n");
