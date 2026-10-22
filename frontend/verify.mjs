import assert from 'node:assert/strict';
import { access, readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runInNewContext } from 'node:vm';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const js = join(root, 'static/js/vendor');
const module = { exports: {} };
runInNewContext(await readFile(join(js, 'katex.min.js'), 'utf8'), { module, exports: module.exports });
const katex = module.exports;
assert.match(katex.renderToString('x^2 + y^2 = z^2'), /<math/);
// GHSA-238p-pmpm-9mq7: inherited options must not enable trusted rendering.
const inherited = Object.assign(Object.create({ trust: true }), { strict: 'ignore' });
assert.doesNotMatch(katex.renderToString('\\href{javascript:alert(1)}{click}', inherited), /href=/);

const highlight = {};
runInNewContext(await readFile(join(js, 'highlight.min.js'), 'utf8'), highlight);
const code = highlight.hljs.highlight('<script>alert(1)</script>', { language: 'xml' }).value;
assert.doesNotMatch(code, /<script>/);
assert.match(code, /&lt;/);

const cssDir = join(root, 'static/css/vendor');
const css = await readFile(join(cssDir, 'katex.min.css'), 'utf8');
const fonts = [...css.matchAll(/url\(([^)]+)\)/g)].map(match => match[1].replace(/["']/g, ''));
assert.ok(fonts.length > 0);
for (const font of fonts) await access(join(cssDir, font));
console.log(`Vendor checks passed: KaTeX ${katex.version}, prototype trust protection, code escaping, ${fonts.length} font references.`);
