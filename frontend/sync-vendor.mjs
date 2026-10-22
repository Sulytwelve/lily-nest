import { transform } from 'esbuild';
import { copyFile, cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontend = dirname(fileURLToPath(import.meta.url));
const root = dirname(frontend);
const modules = join(frontend, 'node_modules');
const js = join(root, 'static/js/vendor');
const css = join(root, 'static/css/vendor');
const licenses = join(js, 'licenses');
await Promise.all([mkdir(js, { recursive: true }), mkdir(css, { recursive: true }), mkdir(licenses, { recursive: true })]);
const bundlePath = join(js, 'MaterialWeb.js');
const compact = await transform(await readFile(bundlePath, 'utf8'), {
  format: 'esm',
  target: 'es2022',
  minify: true,
  supported: { 'template-literal': false },
  legalComments: 'inline',
});
await writeFile(bundlePath, compact.code);

const files = [
  ['katex/dist/katex.min.js', join(js, 'katex.min.js')],
  ['katex/dist/contrib/auto-render.min.js', join(js, 'katex-auto-render.min.js')],
  ['katex/dist/katex.min.css', join(css, 'katex.min.css')],
  ['@highlightjs/cdn-assets/highlight.min.js', join(js, 'highlight.min.js')],
  ['@highlightjs/cdn-assets/styles/atom-one-dark.min.css', join(css, 'atom-one-dark.min.css')],
  ['katex/LICENSE', join(licenses, 'KaTeX.txt')],
  ['@highlightjs/cdn-assets/LICENSE', join(licenses, 'Highlight.js.txt')],
  ['@material/web/LICENSE', join(licenses, 'Material-Web.txt')],
  ['lit/LICENSE', join(licenses, 'Lit.txt')],
  ['lit-html/LICENSE', join(licenses, 'Lit-HTML.txt')],
  ['lit-element/LICENSE', join(licenses, 'Lit-Element.txt')],
  ['@lit/reactive-element/LICENSE', join(licenses, 'Lit-Reactive-Element.txt')],
  ['@lit/context/LICENSE', join(licenses, 'Lit-Context.txt')],
  ['tslib/LICENSE.txt', join(licenses, 'tslib.txt')],
];
for (const [source, destination] of files) await copyFile(join(modules, source), destination);
// KaTeX CSS resolves fonts relative to /css/vendor/katex.min.css.
await cp(join(modules, 'katex/dist/fonts'), join(css, 'fonts'), { recursive: true });
console.log('Updated Material Web, KaTeX, Highlight.js, styles, fonts and licenses.');
