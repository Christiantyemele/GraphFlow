import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import puppeteer from 'puppeteer';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  const [,, scenePath, outPath] = process.argv;
  if (!scenePath || !outPath) {
    console.error('Usage: node render.js <scene.json> <out.svg|out.png>');
    process.exit(1);
  }

  const sceneRaw = await fs.readFile(scenePath, 'utf-8');
  const scene = JSON.parse(sceneRaw);
  const target = outPath.toLowerCase().endsWith('.png') ? 'png' : 'svg';

  const browser = await puppeteer.launch({ args: ['--no-sandbox','--disable-setuid-sandbox','--allow-file-access-from-files'] });
  const page = await browser.newPage();
  page.on('console', msg => console.log('[page]', msg.type(), msg.text()));
  page.on('pageerror', err => console.error('[pageerror]', err));
  const url = 'file://' + path.join(__dirname, 'index.html');

  await page.goto(url, { waitUntil: 'load', timeout: 60000 });

  await page.evaluate((scene, target) => {
    window.__SCENE__ = scene;
    window.__TARGET__ = target;
  }, scene, target);

  // Wait for renderer function to be available and trigger
  await page.waitForFunction('typeof window.renderExcalidraw === "function"', { timeout: 60000 });
  await page.evaluate(() => window.renderExcalidraw());

  // Wait for renderer to finish
  await page.waitForFunction('window.__DONE__ === true', { timeout: 60000 });

  if (target === 'svg') {
    const svg = await page.evaluate(() => window.__SVG__);
    await fs.writeFile(outPath, svg, 'utf-8');
  } else {
    const dataUrl = await page.evaluate(() => window.__PNG_BASE64__);
    const base64 = dataUrl.replace(/^data:image\/png;base64,/, '');
    await fs.writeFile(outPath, base64, 'base64');
  }

  await browser.close();
  console.log(`Wrote ${outPath}`);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
