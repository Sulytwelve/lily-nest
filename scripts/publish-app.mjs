#!/usr/bin/env node
// Node.js >= 18; no third-party dependencies. Upload only the built static output.
import { readFile, readdir, stat } from 'node:fs/promises';
import { resolve, relative, sep } from 'node:path';
import { createPrivateKey, sign, randomUUID } from 'node:crypto';
import { pathToFileURL } from 'node:url';

const supported = /\.(html?|css|m?js|json|txt|svg|png|jpe?g|gif|webp|ico|woff2?|mp3|wav|ogg|mp4|webm)$/i;

export function makeToken(pem, kid) {
    const key = createPrivateKey(pem);
    if (key.asymmetricKeyType !== 'ed25519') throw new Error('Private key must be Ed25519 PEM.');
    const now = Math.floor(Date.now() / 1000);
    const header = { alg: 'EdDSA', typ: 'JWT', ...(kid ? { kid } : {}) };
    const claims = { sub: kid || 'agent', name: 'App publisher', role: 'agent', iat: now, exp: now + 300, jti: randomUUID() };
    const data = [header, claims].map(value => Buffer.from(JSON.stringify(value)).toString('base64url')).join('.');
    return `${data}.${sign(null, Buffer.from(data), key).toString('base64url')}`;
}

export async function appContent(input) {
    const root = resolve(input);
    const metadata = await stat(root);
    if (metadata.isFile()) {
        if (!/\.html?$/i.test(root) || metadata.size > 2 * 1024 * 1024) throw new Error('Input must be an HTML file <= 2 MiB.');
        return { html: await readFile(root, 'utf8') };
    }
    if (!metadata.isDirectory()) throw new Error('Input must be an HTML file or an app folder.');
    const files = [];
    let total = 0;
    async function walk(directory) {
        for (const entry of await readdir(directory, { withFileTypes: true })) {
            const path = resolve(directory, entry.name);
            if (entry.isSymbolicLink()) throw new Error('App folders must not contain symbolic links.');
            if (entry.isDirectory()) { await walk(path); continue; }
            const name = relative(root, path).split(sep).join('/');
            if (!entry.isFile() || !supported.test(name) || name.split('/').some(part => part.startsWith('.'))) {
                throw new Error(`Unsupported publish file: ${name}. Select only the built static output.`);
            }
            const size = (await stat(path)).size;
            total += size;
            if (size > 2 * 1024 * 1024 || total > 8 * 1024 * 1024 || files.length >= 64) throw new Error('Limit: 64 files, 8 MiB total, 2 MiB per file.');
            files.push({ path: name, content: (await readFile(path)).toString('base64'), encoding: 'base64' });
        }
    }
    await walk(root);
    if (!files.some(file => file.path === 'index.html')) throw new Error('App folder must contain root index.html.');
    return { files };
}

async function main() {
    const options = {};
    const allowed = new Set(['site', 'key', 'kid', 'category', 'slug', 'title', 'description', 'input']);
    const args = process.argv.slice(2);
    for (let i = 0; i < args.length; i++) {
        const name = args[i].replace(/^--/, '');
        if (args[i] === '--update' || args[i] === '--help') { options[name] = true; continue; }
        if (!args[i].startsWith('--') || !allowed.has(name) || !args[i + 1] || args[i + 1].startsWith('--')) throw new Error(`Invalid option: ${args[i]}`);
        options[name] = args[++i];
    }
    if (options.help) {
        console.log('node scripts/publish-app.mjs --site https://your-site --key /path/agent.key --kid desktop-agent --category tools --slug demo --title Demo --input ./dist [--update] [--description text]');
        console.log('Environment alternatives: LILY_SITE_URL, LILY_AGENT_KEY_FILE, LILY_AGENT_KEY_ID; or LILY_AGENT_TOKEN for a pre-signed JWT.');
        return;
    }
    const site = options.site || process.env.LILY_SITE_URL;
    const keyPath = options.key || process.env.LILY_AGENT_KEY_FILE;
    const kid = options.kid || process.env.LILY_AGENT_KEY_ID;
    if (!site || !options.input || !options.title || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(options.slug || '') || !['tools', 'games'].includes(options.category)) {
        throw new Error('Required: --site, --input, --title, --slug, --category tools|games. Use --help.');
    }
    const base = new URL(site);
    if (!['http:', 'https:'].includes(base.protocol) || base.username || base.password || base.search || base.hash || base.pathname !== '/') {
        throw new Error('--site must be an HTTP(S) site origin without credentials, path, query or fragment.');
    }
    if (base.protocol !== 'https:' && !['localhost', '127.0.0.1', '[::1]'].includes(base.hostname)) throw new Error('Use HTTPS for remote publishing.');
    const content = await appContent(options.input);
    const token = keyPath ? makeToken(await readFile(keyPath, 'utf8'), kid) : process.env.LILY_AGENT_TOKEN;
    if (!token) throw new Error('Provide --key, LILY_AGENT_KEY_FILE or LILY_AGENT_TOKEN.');
    const endpoint = options.update ? `/api/v1/apps/${options.category}/${options.slug}` : '/api/v1/apps';
    const response = await fetch(new URL(endpoint, base), {
        method: options.update ? 'PUT' : 'POST', redirect: 'error',
        headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
        body: JSON.stringify({ category: options.category, slug: options.slug, title: options.title, description: options.description || '', ...content }),
    });
    if (!response.ok) throw new Error(`Publish failed (${response.status}): ${await response.text()}`);
    const app = await response.json();
    console.log(`Published: ${new URL(app.url, base).href}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
    main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
