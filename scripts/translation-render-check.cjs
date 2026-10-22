// Opt-in browser layout check of an unpacked EPUB; no external network or JS.
// NODE_PATH must expose Playwright. Screenshots are local acceptance artifacts.
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const { chromium } = require('playwright');

async function main() {
  const [input, output, ...documents] = process.argv.slice(2);
  if (!input || !output || !documents.length) throw Error('Usage: node translation-render-check.cjs UNPACKED OUTPUT DOC...');
  const root = path.resolve(input), out = path.resolve(output);
  fs.mkdirSync(out, { recursive: true });
  const server = http.createServer((req, res) => {
    let requested;
    try { requested = path.resolve(root, '.' + decodeURIComponent(new URL(req.url, 'http://localhost').pathname)); }
    catch { res.writeHead(400).end(); return; }
    if (!requested.startsWith(root + path.sep)) { res.writeHead(403).end(); return; }
    const mime = { '.xhtml': 'application/xhtml+xml', '.html': 'text/html', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.ttf': 'font/ttf' };
    fs.readFile(requested, (err, data) => {
      if (err) { res.writeHead(404).end(); return; }
      res.writeHead(200, { 'Content-Type': mime[path.extname(requested)] || 'application/octet-stream' }); res.end(data);
    });
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  let browser;
  const reports = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 900, height: 1000 } });
    await context.route('**/*', route => route.request().url().startsWith(origin + '/') ? route.continue() : route.abort());
    const page = await context.newPage();
    for (const document of documents) {
      const response = await page.goto(origin + '/' + document.replaceAll('\\', '/'), { waitUntil: 'networkidle' });
      if (!response || !response.ok()) throw Error(`Cannot render ${document}`);
      await page.evaluate(() => document.fonts.ready);
      const metrics = await page.evaluate(() => {
        const paragraph = document.querySelector('p') || document.body;
        const css = getComputedStyle(paragraph);
        return { language: document.documentElement.lang, writingMode: css.writingMode, direction: css.direction,
          font: css.fontFamily, bodyWidth: document.body.scrollWidth, viewport: innerWidth,
          textCharacters: document.body.textContent.trim().length,
          brokenImages: Array.from(document.images).filter(img => !img.complete || !img.naturalWidth).length,
          svgImages: document.querySelectorAll('svg image').length };
      });
      const screenshot = path.join(out, document.replace(/[\\/:]/g, '_') + '.png');
      await page.screenshot({ path: screenshot });
      reports.push({ document, ...metrics, screenshot, renderer: 'Chromium XHTML viewport; EPUB pagination still needs reader acceptance' });
    }
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
  fs.writeFileSync(path.join(out, 'render-report.json'), JSON.stringify(reports, null, 2));
  for (const report of reports) {
    if (report.writingMode !== 'horizontal-tb' || report.direction !== 'ltr' || report.brokenImages) throw Error(`Layout failure: ${report.document}`);
  }
  console.log(JSON.stringify(reports));
}
main().catch(err => { console.error(err.message); process.exitCode = 1; });
