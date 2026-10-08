// Integration test with a temporary site and generated keys; never uses deployment keys.
// Run: cargo build --offline && node scripts/test-app-publishing.mjs
import assert from 'node:assert/strict';
import { generateKeyPairSync, sign, createHmac } from 'node:crypto';
import { mkdtemp, mkdir, writeFile, cp, rm, access } from 'node:fs/promises';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { connect } from 'node:http2';
import { once } from 'node:events';
import { makeToken } from './publish-app.mjs';

const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const target = join(project, 'target');
await mkdir(target, { recursive: true });
const fixture = await mkdtemp(join(target, 'app-integration-'));
const executable = join(target, 'debug', process.platform === 'win32' ? 'lily-nest.exe' : 'lily-nest');
const keyPair = () => generateKeyPairSync('ed25519', {
    publicKeyEncoding: { type: 'spki', format: 'pem' },
    privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
});
const [first, second, legacy] = [keyPair(), keyPair(), keyPair()];
const secret = 'test-only-secret-with-more-than-32-bytes';
let server;
let output = '';
let assertions = 0;
const keep = process.argv.includes('--keep');
const probe = createServer();
probe.listen(0, '::1');
await once(probe, 'listening');
const port = probe.address().port;
await new Promise(resolve => probe.close(resolve));
const site = `http://[::1]:${port}`;
const token1 = makeToken(first.privateKey, 'first');
const token2 = makeToken(second.privateKey, 'second');

async function start() {
    output = '';
    const env = { ...process.env, LILY_JWT_SECRET: secret, LILY_AGENT_PUB_KEY: legacy.publicKey,
        LILY_AGENT_KEYS_FILE: join(fixture, '.agent-keys.toml'), RUST_LOG: 'warn' };
    delete env.LILY_ADMIN_PASSWORD;
    server = spawn(executable, [], { cwd: fixture, env, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
    server.stdout.on('data', chunk => { output += chunk; });
    server.stderr.on('data', chunk => { output += chunk; });
    let spawnError;
    server.on('error', error => { spawnError = error; });
    for (let i = 0; i < 100; i++) {
        if (spawnError) throw spawnError;
        if (server.exitCode !== null) throw new Error(`Test server exited: ${output}`);
        try { if ((await fetch(`${site}/api/v1/health`)).ok) return; } catch {}
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    throw new Error(`Test server failed to start: ${output}`);
}
async function stop() {
    if (server && server.exitCode === null) {
        const exited = once(server, 'exit');
        server.kill();
        await exited;
    }
}
async function request(path, { token = token1, method = 'GET', body, expected = 200 } = {}) {
    const response = await fetch(`${site}${path}`, { method, redirect: 'manual',
        headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}), ...(body ? { 'Content-Type': 'application/json' } : {}) },
        ...(body ? { body: JSON.stringify(body) } : {}) });
    assert.equal(response.status, expected, `${method} ${path}: ${await response.clone().text()}`);
    assertions++;
    return response;
}
const textFile = (path, content) => ({ path, content, encoding: 'text' });
const browserFiles = [
    textFile('index.html', '<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="./css/style.css"></head><body><h1 id="result">Loading</h1><a href="./other.html">Other page</a><script src="./js/classic.js"></script><script type="module" src="./js/main.mjs"></script></body></html>'),
    textFile('css/style.css', '#result { color: rgb(12, 120, 40); }'),
    textFile('js/classic.js', 'document.body.dataset.classic = "loaded";'),
    textFile('js/main.mjs', 'import { message } from "./helper.mjs"; const data = await (await fetch("./data.json")).json(); document.querySelector("#result").textContent = message + " " + data.value + " " + document.body.dataset.classic;'),
    textFile('js/helper.mjs', 'export const message = "Module";'),
    textFile('data.json', '{"value":"JSON"}'),
    textFile('other.html', '<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="./css/style.css"></head><body><h1 id="result">Second page</h1><a href="./">Home</a></body></html>'),
];
const keyConfig = enabled => [
    ['first', first, true], ['second', second, enabled],
].map(([id, pair, on]) => `[[keys]]\nid = "${id}"\nname = "${id} Agent"\nenabled = ${on}\npublic_key = ${JSON.stringify(pair.publicKey)}\n`).join('\n');
function customToken(pair, kid, { role = 'agent', exp = Math.floor(Date.now() / 1000) + 300 } = {}) {
    const data = [{ alg: 'EdDSA', ...(kid === undefined ? {} : { kid }) }, { sub: 'test', name: 'test', role, exp }]
        .map(value => Buffer.from(JSON.stringify(value)).toString('base64url')).join('.');
    return `${data}.${sign(null, Buffer.from(data), pair.privateKey).toString('base64url')}`;
}

try {
    for (const dir of ['templates', 'static']) await cp(join(project, dir), join(fixture, dir), { recursive: true });
    for (const file of ['site.toml', 'about.toml', 'projects.toml', 'changelog.toml']) await cp(join(project, file), join(fixture, file));
    await writeFile(join(fixture, 'config.toml'), `[server]\nhttp_port = ${port}\nhttps_port = 8443\n[assets]\nprecompress = false\n`);
    await writeFile(join(fixture, '.agent-keys.toml'), keyConfig(true));
    await mkdir(join(fixture, 'apps', 'tools'), { recursive: true });
    await writeFile(join(fixture, 'apps', 'tools', 'old.html'), '<!doctype html><html><body>Legacy HTML</body></html>');
    await writeFile(join(fixture, 'apps', 'manifest.toml'), '[[apps]]\ncategory = "tools"\nslug = "old"\ntitle = "Old"\nurl = "/tools/old"\ncreated_at = "2026-10-09T00:00:00Z"\nupdated_at = "2026-10-09T00:00:00Z"\n');
    await start();
    await request('/api/v1/apps', { token: null, expected: 401 });
    await request('/api/v1/apps', { token: token2 });
    await request('/api/v1/apps', { token: makeToken(legacy.privateKey) });
    await request('/api/v1/apps', { token: makeToken(legacy.privateKey, 'legacy') });
    for (const token of [makeToken(first.privateKey), makeToken(first.privateKey, 'unknown'), makeToken(second.privateKey, 'first'), customToken(first, 'first', { exp: 1 })]) {
        await request('/api/v1/apps', { token, expected: 401 });
    }
    await request('/api/v1/apps', { token: customToken(first, 'first', { role: 'reader' }), expected: 403 });
    await request('/api/v1/admin/apps', { expected: 401 });
    await request('/api/v1/notes');
    const adminData = [{ alg: 'HS256' }, { sub: 'admin', name: 'admin', role: 'admin', exp: Math.floor(Date.now() / 1000) + 300 }].map(value => Buffer.from(JSON.stringify(value)).toString('base64url')).join('.');
    const adminToken = `${adminData}.${createHmac('sha256', secret).update(adminData).digest('base64url')}`;
    await request('/api/v1/admin/apps', { token: adminToken });
    const old = await request('/tools/old', { token: null });
    assert.match(await old.text(), /Legacy HTML/);
    const app = { category: 'tools', slug: 'demo', title: 'Demo', files: browserFiles };
    const created = await request('/api/v1/apps', { method: 'POST', body: app, expected: 201 });
    assert.equal((await created.json()).url, '/tools/demo/');
    await request('/api/v1/apps', { method: 'POST', body: app, expected: 409 });
    const redirect = await request('/tools/demo', { token: null, expected: 308 });
    assert.equal(redirect.headers.get('location'), '/tools/demo/');
    const entry = await request('/tools/demo/', { token: null });
    assert.match(entry.headers.get('content-security-policy'), /sandbox allow-scripts/);
    assert.ok(!entry.headers.get('content-security-policy').includes('allow-same-origin'));
    const h2 = connect(site);
    const h2Request = h2.request({ ':path': '/tools/demo/', ':authority': `localhost:${port}` });
    const [h2Headers] = await once(h2Request, 'response');
    assert.match(h2Headers['content-security-policy'], new RegExp(`http://localhost:${port}/tools/demo/`));
    h2Request.resume();
    await once(h2Request, 'end');
    h2.close();
    assertions++;
    for (const path of ['css/style.css', 'js/main.mjs', 'js/helper.mjs', 'js/classic.js', 'data.json', 'other.html']) {
        const response = await request(`/tools/demo/${path}`, { token: null });
        assert.equal(response.headers.get('access-control-allow-origin'), '*');
    }
    for (const path of ['missing.js', '%2e%2e%2fmanifest.toml', '.agent.pub', 'js%5cmain.mjs']) await request(`/tools/demo/${path}`, { token: null, expected: 404 });
    for (const files of [[textFile('a.js', 'x')], [textFile('index.html', 'ok'), textFile('../bad.js', 'x')], [textFile('index.html', 'ok'), textFile('a.exe', 'x')], [textFile('index.html', 'ok'), textFile('A.js', 'x'), textFile('a.js', 'x')]]) {
        await request('/api/v1/apps', { method: 'POST', body: { ...app, slug: 'bad', files }, expected: 400 });
    }
    await request('/api/v1/apps', { method: 'POST', body: { ...app, slug: 'bad', html: 'also' }, expected: 400 });
    await request('/api/v1/apps/tools/demo', { method: 'PUT', body: { title: 'Metadata update' } });
    await request('/tools/demo/js/main.mjs', { token: null });
    await request('/api/v1/apps/tools/demo', { method: 'PUT', body: { slug: 'other' }, expected: 400 });
    await request('/api/v1/apps/tools/old', { method: 'PUT', body: { files: browserFiles } });
    await assert.rejects(access(join(fixture, 'apps', 'tools', 'old.html')));
    await request('/tools/old', { token: null, expected: 308 });
    await request('/tools/old/js/helper.mjs', { token: null });
    // Exercise actual CLI signing and file packaging against the temporary server.
    const dist = join(fixture, 'dist');
    await mkdir(dist);
    await writeFile(join(dist, 'index.html'), '<!doctype html><html><body>CLI app</body></html>');
    const privateKeyPath = join(fixture, 'test-private.pem');
    await writeFile(privateKeyPath, first.privateKey);
    const cli = spawn(process.execPath, [join(project, 'scripts', 'publish-app.mjs'), '--site', site, '--key', privateKeyPath, '--kid', 'first', '--category', 'games', '--slug', 'cli-demo', '--title', 'CLI', '--input', dist], { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
    let cliOutput = '';
    cli.stdout.on('data', chunk => { cliOutput += chunk; });
    cli.stderr.on('data', chunk => { cliOutput += chunk; });
    const [code] = await once(cli, 'exit');
    assert.equal(code, 0, cliOutput);
    assert.match(cliOutput, /Published:/);
    await request('/games/cli-demo/', { token: null });
    await request('/api/v1/apps/games/cli-demo', { method: 'PUT', body: { html: '<html><body>Single again</body></html>' } });
    await request('/games/cli-demo', { token: null });
    await request('/games/cli-demo/index.html', { token: null, expected: 404 });
    await request('/api/v1/apps/games/cli-demo', { method: 'DELETE', expected: 204 });
    await request('/games/cli-demo', { token: null, expected: 404 });
    assert.match(await (await request('/apps?format=markdown', { token: null })).text(), /\/tools\/demo\//);
    await stop();
    await writeFile(join(fixture, '.agent-keys.toml'), keyConfig(false));
    await start();
    await request('/api/v1/apps', { token: token2, expected: 401 });
    await request('/api/v1/apps', { token: token1 });
    await request('/api/v1/apps', { token: makeToken(legacy.privateKey) });
    await request('/tools/demo/', { token: null });
    await request('/tools/old/other.html', { token: null });
    console.log(`Passed ${assertions} HTTP checks, CLI publishing, key disable/restart and legacy migration.`);
    if (keep) {
        console.log(`Browser fixture: ${site}/tools/demo/`);
        console.log(`Temporary directory: ${fixture}`);
        // Keeping stdin open lets the runner stop and clean up after browser verification.
        process.stdin.resume();
        await once(process.stdin, 'data');
        process.stdin.pause();
    }
} finally {
    await stop();
    assert.equal(dirname(fixture), target);
    assert.ok(fixture.startsWith(join(target, 'app-integration-')));
    await rm(fixture, { recursive: true, force: true });
}
