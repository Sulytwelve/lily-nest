/* Admin Page Logic */
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

    let currentPassword = localStorage.getItem('admin_password') || '';
    let currentAnswer = localStorage.getItem('admin_answer') || '';
    let currentQuestionIndex = localStorage.getItem('admin_question_index') || 0;
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
            if (cftraceUrl.startsWith('/')) {
                url = cftraceUrl;
            } else {
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

    async function apiFetch(url, options = {}) {
        // 被限流期间不发送请求
        if (Date.now() < rateLimitedUntil) {
            updateStatus('Server not responding, please try again later', true);
            throw new Error('Rate limited');
        }

        if (authExtCftraceEnabled && !rawCfTrace) {
            await fetchTrace();
        }

        const isGet = !options.method || options.method === 'GET';
        const headers = {
            'X-Admin-Password': encodeURIComponent(currentPassword),
            'X-Admin-Answer': authExtSecqEnabled ? encodeURIComponent(currentAnswer) : '',
            'X-Admin-Question-Index': authExtSecqEnabled ? currentQuestionIndex.toString() : '0',
            'X-Admin-Trace': encodeURIComponent(rawCfTrace),
            ...(isGet ? {} : { 'Content-Type': 'application/json' }),
            ...options.headers
        };

        let response;
        try {
            response = await fetch(url, { ...options, headers });
        } catch (e) {
            updateStatus('Server not responding', true);
            throw e;
        }

        if (response.status === 429) {
            rateLimitedUntil = Date.now() + 60000;
            updateStatus('Server not responding, please try again later', true);
            console.warn('Admin: rate limited by server');
            throw new Error('Rate limited');
        }

        if (response.status === 401) {
            tempQuestionIndex = pickRandomQuestion();
            authDialog.show();
            throw new Error('Unauthorized');
        }

        return response;
    }

    loginConfirmBtn.addEventListener('click', async () => {
        if (loginConfirmBtn.disabled) return;

        if (!passwordInput.value || (authExtSecqEnabled && !securityAnswerInput.value)) {
            updateStatus('Required fields missing', true);
            return;
        }

        currentPassword = passwordInput.value;
        localStorage.setItem('admin_password', currentPassword);

        if (authExtSecqEnabled) {
            currentAnswer = securityAnswerInput.value;
            currentQuestionIndex = tempQuestionIndex;
            localStorage.setItem('admin_answer', currentAnswer);
            localStorage.setItem('admin_question_index', currentQuestionIndex);
        }

        authDialog.close();
        loginConfirmBtn.disabled = true;
        try {
            await loadConfigs();
        } finally {
            loginConfirmBtn.disabled = false;
        }
    });

    passwordInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            loginConfirmBtn.click();
        }
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
                body: JSON.stringify({ content: editor.value })
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
        localStorage.removeItem('admin_password');
        localStorage.removeItem('admin_answer');
        localStorage.removeItem('admin_question_index');
        window.location.reload();
    });

    // Initial load
    if (currentPassword) {
        loadConfigs();
    } else {
        setTimeout(() => {
            if (!currentPassword) {
                tempQuestionIndex = pickRandomQuestion();
                authDialog.show();
            }
        }, 100);
    }
});
