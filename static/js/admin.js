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

    function updateStatus(msg, isError = false) {
        statusText.innerText = msg;
        statusText.style.color = isError ? 'var(--md-sys-color-error)' : 'var(--md-sys-color-on-surface-variant)';
    }

    async function initAuthConfig() {
        try {
            const res = await fetch('/api/v1/admin/auth_config');
            if (res.ok) {
                const config = await res.json();
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
            }
        } catch (e) {
            console.error('Failed to fetch auth config:', e);
        }
    }

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
            const res = await fetch(cftraceUrl);
            if (res.ok) {
                rawCfTrace = await res.text();
            }
        } catch (e) {
            console.warn('Failed to fetch Cloudflare trace:', e);
        }
    }

    async function apiFetch(url, options = {}) {
        if (authExtCftraceEnabled && !rawCfTrace) {
            await fetchTrace();
        }

        const headers = {
            'X-Admin-Password': encodeURIComponent(currentPassword),
            'X-Admin-Answer': authExtSecqEnabled ? encodeURIComponent(currentAnswer) : '',
            'X-Admin-Question-Index': authExtSecqEnabled ? currentQuestionIndex.toString() : '0',
            'X-Admin-Trace': encodeURIComponent(rawCfTrace),
            'Content-Type': 'application/json',
            ...options.headers
        };
        try {
            const response = await fetch(url, { ...options, headers });
            if (response.status === 401) {
                tempQuestionIndex = pickRandomQuestion();
                authDialog.show();
                throw new Error('Unauthorized');
            }
            return response;
        } catch (e) {
            if (e.message !== 'Unauthorized') {
                updateStatus(`Error: ${e.message}`, true);
            }
            throw e;
        }
    }

    loginConfirmBtn.addEventListener('click', () => {
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
        loadConfigs();
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
            if (!res.ok) throw new Error('Failed to load configs');
            const configs = await res.json();
            
            configList.innerHTML = '';
            configs.forEach(cfg => {
                const item = document.createElement('md-list-item');
                item.type = 'button';
                item.innerHTML = `<div slot="headline">${cfg.name}</div>`;
                item.addEventListener('click', () => loadFile(cfg.name));
                configList.appendChild(item);
            });
            updateStatus('Ready');
        } catch (e) {
            console.error(e);
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
                updateStatus('File loaded.');
            } else {
                updateStatus(`Error: ${res.statusText}`, true);
            }
        } catch (e) {
            console.error(e);
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
                updateStatus('Saved successfully.');
                setTimeout(() => updateStatus('Ready'), 3000);
            } else {
                const text = await res.text();
                updateStatus(`Save Error: ${text}`, true);
            }
        } catch (e) {
            updateStatus(`Error: ${e.message}`, true);
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
    initAuthConfig().then(() => {
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
});
