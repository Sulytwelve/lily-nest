/* Admin Page Logic — JWT Auth */
document.addEventListener('DOMContentLoaded', () => {
    const editor = document.getElementById('editor');
    const configList = document.getElementById('config-list');
    const saveBtn = document.getElementById('save-btn');
    const authDialog = document.getElementById('auth-dialog');
    const passwordInput = document.getElementById('password-input');
    const loginConfirmBtn = document.getElementById('login-confirm-btn');
    const currentFilenameDisplay = document.getElementById('current-filename');
    const statusText = document.getElementById('status-text');
    const logoutBtn = document.getElementById('logout-btn');

    const securityQuestionContainer = document.getElementById('security-question-container');
    let securityAnswerInput = null;
    let securityQuestionLabel = null;

    let securityQuestions = [];

    // JWT token 存在 sessionStorage，关闭 tab 即失效
    let jwtToken = sessionStorage.getItem('admin_jwt') || '';
    let jwtExpiresAt = parseInt(sessionStorage.getItem('admin_jwt_expires_at') || '0', 10);

    let currentFile = '';
    let rawCfTrace = '';
    let authExtSecqEnabled = false;
    let authExtCftraceEnabled = false;
    let cftraceUrl = 'https://cloudflare.com/cdn-cgi/trace';

    // 被限流时的时间戳，此时间之前不再发请求
    let rateLimitedUntil = 0;

    function updateStatus(msg, isError = false) {
        statusText.innerText = msg;
        statusText.style.color = isError ? 'var(--md-sys-color-error)' : 'var(--md-sys-color-on-surface-variant)';
    }

    function clearStatus() {
        statusText.innerText = '';
    }

    function isTokenValid() {
        if (!jwtToken) return false;
        const nowSec = Math.floor(Date.now() / 1000);
        // 提前 30 秒视为过期，避免边界条件
        return jwtExpiresAt > nowSec + 30;
    }

    function clearToken() {
        jwtToken = '';
        jwtExpiresAt = 0;
        sessionStorage.removeItem('admin_jwt');
        sessionStorage.removeItem('admin_jwt_expires_at');
    }

    (function initAuthConfig() {
        const configElement = document.getElementById('auth-config-json');
        if (!configElement) {
            console.warn('Admin: auth config element not found in page');
            updateStatus('Server not responding', true);
            return;
        }
        let config;
        try {
            config = JSON.parse(configElement.textContent);
        } catch (e) {
            console.warn('Admin: failed to parse auth config');
            updateStatus('Server not responding', true);
            return;
        }
        authExtSecqEnabled = config.auth_ext_secq;
        authExtCftraceEnabled = config.auth_ext_cftrace;
        if (config.security_questions) {
            securityQuestions = config.security_questions;
        }
        if (config.cftrace_url && config.cftrace_url.trim() !== '') {
            let url = config.cftrace_url.trim();
            if (url.startsWith('http://') || url.startsWith('https://') || url.startsWith('/')) {
                cftraceUrl = url;
            } else {
                if (url === window.location.hostname || url === window.location.host) {
                    cftraceUrl = '/cdn-cgi/trace';
                } else {
                    cftraceUrl = `https://${url}/cdn-cgi/trace`;
                }
            }
        }
        if (authExtSecqEnabled) {
            securityQuestionContainer.innerHTML = `
                <p id="security-question-label" class="md-typescale-body-medium" style="margin-bottom: 8px; color: var(--md-sys-color-primary);"></p>
                <md-outlined-text-field id="security-answer-input" label="Security Answer" style="width: 100%;"></md-outlined-text-field>
            `;
            securityAnswerInput = document.getElementById('security-answer-input');
            securityQuestionLabel = document.getElementById('security-question-label');
            securityAnswerInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') loginConfirmBtn.click();
            });
        }
    })();

    function pickRandomQuestion() {
        if (!authExtSecqEnabled || !securityQuestions || securityQuestions.length === 0) return 0;
        const index = Math.floor(Math.random() * securityQuestions.length);
        if (securityQuestionLabel) {
            securityQuestionLabel.innerText = `Security Question: ${securityQuestions[index]}`;
        }
        return index;
    }

    let tempQuestionIndex = 0;

    async function fetchTrace() {
        try {
            let url = cftraceUrl;
            if (!cftraceUrl.startsWith('/')) {
                let traceHost = cftraceUrl.trim().toLowerCase();
                if (traceHost.startsWith('http://')) traceHost = traceHost.substring(7);
                else if (traceHost.startsWith('https://')) traceHost = traceHost.substring(8);
                traceHost = traceHost.split('/')[0];

                let traceHostname = traceHost;
                if (traceHost.startsWith('[')) {
                    const idx = traceHost.indexOf(']');
                    if (idx !== -1) traceHostname = traceHost.substring(0, idx + 1);
                } else {
                    traceHostname = traceHost.split(':')[0];
                }

                const currentHost = window.location.hostname.toLowerCase();
                const normalize = dom => dom.startsWith('www.') ? dom.substring(4) : dom;
                const useRelative = normalize(currentHost) === normalize(traceHostname);
                url = useRelative ? '/cdn-cgi/trace' : (cftraceUrl.startsWith('http') ? cftraceUrl : `https://${cftraceUrl}`);
            }

            const res = await fetch(url);
            if (res.ok) {
                rawCfTrace = await res.text();
            }
        } catch (e) {
            console.warn('Admin: trace fetch failed');
        }
    }

    // 登录：向 /api/v1/admin/login 发送凭据，换取 JWT
    async function doLogin(password, answer, questionIndex, cfTrace) {
        if (Date.now() < rateLimitedUntil) {
            updateStatus('Too many attempts, please wait.', true);
            throw new Error('Rate limited');
        }

        const body = { password };
        if (authExtSecqEnabled) {
            body.answer = answer;
            body.question_index = questionIndex;
        }
        if (authExtCftraceEnabled) {
            body.cf_trace = cfTrace;
        }

        let response;
        try {
            response = await fetch('/api/v1/admin/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(body),
            });
        } catch (e) {
            updateStatus('Server not responding', true);
            throw e;
        }

        if (response.status === 429) {
            rateLimitedUntil = Date.now() + 60000;
            updateStatus('Too many attempts, please wait 60s.', true);
            throw new Error('Rate limited');
        }

        if (response.status === 401) {
            updateStatus('Authentication failed. Check password/answer.', true);
            throw new Error('Unauthorized');
        }

        if (!response.ok) {
            updateStatus('Server error during login', true);
            throw new Error('Login failed');
        }

        const data = await response.json();
        jwtToken = data.token;
        jwtExpiresAt = data.expires_at;
        sessionStorage.setItem('admin_jwt', jwtToken);
        sessionStorage.setItem('admin_jwt_expires_at', String(jwtExpiresAt));
    }

    // 带 JWT 的通用 API 请求
    async function apiFetch(url, options = {}) {
        if (!isTokenValid()) {
            // Token 过期或不存在，弹出登录框
            clearToken();
            tempQuestionIndex = pickRandomQuestion();
            authDialog.show();
            throw new Error('Unauthorized');
        }

        const isGet = !options.method || options.method === 'GET';
        const headers = {
            'Authorization': `Bearer ${jwtToken}`,
            ...(isGet ? {} : { 'Content-Type': 'application/json' }),
            ...options.headers,
        };

        let response;
        try {
            response = await fetch(url, { ...options, headers });
        } catch (e) {
            updateStatus('Server not responding', true);
            throw e;
        }

        if (response.status === 401) {
            // Token 被服务端拒绝（如服务器重启后 secret 变更）
            clearToken();
            tempQuestionIndex = pickRandomQuestion();
            authDialog.show();
            throw new Error('Unauthorized');
        }

        return response;
    }

    loginConfirmBtn.addEventListener('click', async () => {
        if (loginConfirmBtn.disabled) return;

        const password = passwordInput.value;
        if (!password || (authExtSecqEnabled && !securityAnswerInput?.value)) {
            updateStatus('Required fields missing', true);
            return;
        }

        const answer = authExtSecqEnabled ? securityAnswerInput.value : undefined;
        const questionIndex = authExtSecqEnabled ? tempQuestionIndex : undefined;

        if (authExtCftraceEnabled && !rawCfTrace) {
            await fetchTrace();
        }

        authDialog.close();
        loginConfirmBtn.disabled = true;
        try {
            await doLogin(password, answer, questionIndex, rawCfTrace);
            await loadConfigs();
        } catch (e) {
            if (e.message === 'Unauthorized') {
                // 已在 doLogin 里 updateStatus，重新打开对话框
                tempQuestionIndex = pickRandomQuestion();
                authDialog.show();
            }
        } finally {
            loginConfirmBtn.disabled = false;
        }
    });

    passwordInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') loginConfirmBtn.click();
    });

    async function loadConfigs() {
        try {
            updateStatus('Loading files...');
            const res = await apiFetch('/api/v1/admin/configs');
            if (!res.ok) throw new Error('Request failed');
            const configs = await res.json();

            configList.innerHTML = '';
            configs.forEach(cfg => {
                const item = document.createElement('md-list-item');
                item.type = 'button';
                item.innerHTML = `<div slot="headline">${cfg.name}</div>`;
                item.addEventListener('click', () => loadFile(cfg.name));
                configList.appendChild(item);
            });
            clearStatus();
        } catch (e) {
            if (e.message !== 'Unauthorized' && e.message !== 'Rate limited') {
                updateStatus('Server not responding', true);
            }
            console.warn('Admin: load configs failed');
        }
    }

    async function loadFile(name) {
        currentFile = name;
        currentFilenameDisplay.innerText = name;
        updateStatus(`Loading ${name}...`);
        try {
            const res = await apiFetch(`/api/v1/admin/configs/${name}`);
            if (res.ok) {
                editor.value = await res.text();
                saveBtn.disabled = false;
                clearStatus();
            } else {
                updateStatus('Server not responding', true);
            }
        } catch (e) {
            if (e.message !== 'Unauthorized' && e.message !== 'Rate limited') {
                updateStatus('Server not responding', true);
            }
            console.warn('Admin: load file failed');
        }
    }

    saveBtn.addEventListener('click', async () => {
        if (!currentFile) return;
        saveBtn.disabled = true;
        updateStatus('Saving...');
        try {
            const res = await apiFetch(`/api/v1/admin/configs/${currentFile}`, {
                method: 'POST',
                body: JSON.stringify({ content: editor.value }),
            });
            if (res.ok) {
                updateStatus('Saved');
                setTimeout(() => clearStatus(), 3000);
            } else {
                updateStatus('Server not responding', true);
            }
        } catch (e) {
            if (e.message !== 'Unauthorized' && e.message !== 'Rate limited') {
                updateStatus('Server not responding', true);
            }
        } finally {
            saveBtn.disabled = false;
        }
    });

    logoutBtn.addEventListener('click', () => {
        clearToken();
        window.location.reload();
    });

    // Initial load
    if (isTokenValid()) {
        loadConfigs();
    } else {
        clearToken();
        setTimeout(() => {
            tempQuestionIndex = pickRandomQuestion();
            authDialog.show();
        }, 100);
    }
});
