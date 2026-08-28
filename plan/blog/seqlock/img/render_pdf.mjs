import puppeteer from 'puppeteer';
const [,, htmlPath, outPath] = process.argv;
const browser = await puppeteer.launch({ args: ['--no-sandbox'] });
const page = await browser.newPage();
await page.goto('file://' + htmlPath, { waitUntil: 'networkidle0' });
await page.evaluateHandle('document.fonts.ready');
await page.pdf({ path: outPath, format: 'A4', printBackground: true,
  margin: { top: '14mm', bottom: '14mm', left: '14mm', right: '14mm' } });
await browser.close();
console.log('wrote', outPath);
