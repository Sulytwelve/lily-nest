/* Admin Page Logic — JWT Auth */
document.addEventListener('DOMContentLoaded', () => {
    const editor = document.getElementById('editor');
    const configList = document.getElementById('config-list');
    const saveBtn = document.getElementById('save-btn');
    const authDialog = document.getElementById('auth-dialog');
    const passwordInput = document.getElementById('password-input');
    const loginConfirmBtn = document.getElementById('login-confirm-btn');
    const setupDialog = document.getElementById('setup-dialog');
    const setupCodeInput = document.getElementById('setup-code-input');
    const setupPasswordInput = document.getElementById('setup-password-input');
    const setupConfirmInput = document.getElementById('setup-confirm-input');
    const setupConfirmBtn = document.getElementById('setup-confirm-btn');
    const setupStatusText = document.getElementById('setup-status-text');
    const currentFilenameDisplay = document.getElementById('current-filename');
    const statusText = document.getElementById('status-text');
    const logoutBtn = document.getElementById('logout-btn');

    const securityQuestionContainer = document.getElementById('security-question-container');
    let securityAnswerInput = null;
    let securityQuestionLabel = null;

    let questionCount = 0;

    // Migrate old token keys to unified keys
    (function migrateTokens() {
        if (sessionStorage.getItem('auth_jwt')) return;
        const oldToken = sessionStorage.getItem('admin_jwt');
        if (oldToken) {
            sessionStorage.setItem('auth_jwt', oldToken);
            sessionStorage.setItem('auth_role', 'admin');
            sessionStorage.setItem('auth_name', '管理员');
            sessionStorage.setItem('auth_expires_at', sessionStorage.getItem('admin_jwt_expires_at') || '0');
            sessionStorage.removeItem('admin_jwt');
            sessionStorage.removeItem('admin_jwt_expires_at');
        }
    })();

    // JWT token 存在 sessionStorage，关闭 tab 即失效（统一 auth 键）
    let jwtToken = sessionStorage.getItem('auth_jwt') || '';
    let jwtExpiresAt = parseInt(sessionStorage.getItem('auth_expires_at') || '0', 10);

    let currentFile = '';
    let rawCfTrace = '';
    let authExtSecqEnabled = false;
    let authExtCftraceEnabled = false;
    let setupRequired = false;

    // 被限流时的时间戳，此时间之前不再发请求
    let rateLimitedUntil = 0;

    // 统一 fetch 封装：默认 15s 超时；调用方已提供 signal 时尊重原 signal
    function fetchWithTimeout(url, options = {}, ms = 15000) {
        const opts = { ...options };
        if (!opts.signal) {
            opts.signal = AbortSignal.timeout(ms);
        }
        return fetch(url, opts);
    }

    function updateStatus(msg, isError = false) {
        statusText.innerText = msg;
        statusText.style.color = isError ? 'var(--md-sys-color-error)' : 'var(--md-sys-color-on-surface-variant)';
    }

    function clearStatus() {
        statusText.innerText = '';
    }

    function setSetupStatus(msg, isError = false) {
        if (!setupStatusText) return;
        setupStatusText.innerText = msg;
        setupStatusText.style.color = isError
            ? 'var(--md-sys-color-error)'
            : 'var(--md-sys-color-on-surface-variant)';
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
        sessionStorage.removeItem('auth_jwt');
        sessionStorage.removeItem('auth_role');
        sessionStorage.removeItem('auth_name');
        sessionStorage.removeItem('auth_expires_at');
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
        questionCount = config.question_count || 0;
        setupRequired = config.setup_required === true;
        if (authExtSecqEnabled) {
            securityQuestionContainer.innerHTML = '';
            const label = document.createElement('p');
            label.id = 'security-question-label';
            label.className = 'md-typescale-body-medium';
            label.style.marginBottom = '8px';
            label.style.color = 'var(--md-sys-color-primary)';
            securityQuestionContainer.appendChild(label);

            const input = document.createElement('md-outlined-text-field');
            input.id = 'security-answer-input';
            input.setAttribute('label', 'Security Answer');
            input.style.width = '100%';
            securityQuestionContainer.appendChild(input);

            securityAnswerInput = input;
            securityQuestionLabel = label;
            securityAnswerInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') loginConfirmBtn.click();
            });
        }
    })();

    let tempQuestionIndex = 0;

    async function refreshQuestion() {
        try {
            const res = await fetchWithTimeout('/api/v1/admin/login/question', { cache: 'no-store' });
            if (!res.ok) {
                updateStatus('Security question unavailable', true);
                throw new Error('Security question unavailable');
            }
            const data = await res.json();
            tempQuestionIndex = data.question_index;
            if (securityQuestionLabel) {
                securityQuestionLabel.innerText = `Security Question: ${data.question}`;
            }
        } catch (e) {
            updateStatus('Security question unavailable', true);
            throw e;
        }
    }

    async function showAuthDialog() {
        if (setupRequired) {
            setupDialog.show();
            return;
        }
        if (authExtSecqEnabled) {
            try {
                await refreshQuestion();
            } catch (e) {
                // 取题失败仍打开对话框；提交时服务端会拒绝并提示
            }
        }
        authDialog.show();
    }

    async function fetchTrace() {
        try {
            const res = await fetchWithTimeout('/cdn-cgi/trace');
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
            response = await fetchWithTimeout('/api/v1/admin/login', {
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
            let message = 'Authentication failed. Check password/answer.';
            try {
                const text = await response.text();
                if (text && text.includes('not configured')) {
                    message = '管理员未初始化，请运行 set-password 或使用 Setup Code 初始化。';
                }
            } catch (e) {
                // 保留默认提示
            }
            updateStatus(message, true);
            throw new Error('Unauthorized');
        }

        if (!response.ok) {
            updateStatus('Server error during login', true);
            throw new Error('Login failed');
        }

        const data = await response.json();
        jwtToken = data.token;
        jwtExpiresAt = data.expires_at;
        // 统一 auth 存储
        sessionStorage.setItem('auth_jwt', jwtToken);
        sessionStorage.setItem('auth_role', data.role || 'admin');
        sessionStorage.setItem('auth_name', data.name || '管理员');
        sessionStorage.setItem('auth_expires_at', String(jwtExpiresAt));
    }

    // 带 JWT 的通用 API 请求
    async function apiFetch(url, options = {}) {
        if (!isTokenValid()) {
            // Token 过期或不存在，弹出登录框
            clearToken();
            await showAuthDialog();
            throw new Error('Unauthorized');
        }

        const isGet = !options.method || options.method === 'GET';
        const headers = {
            'Authorization': `Bearer ${jwtToken}`,
            ...(isGet ? {} : { 'Content-Type': 'application/json' }),
            ...options.headers,
        };

        const fetchOptions = { ...options, headers };
        if (isGet && !('cache' in fetchOptions)) {
            fetchOptions.cache = 'no-store';
        }

        let response;
        try {
            response = await fetchWithTimeout(url, fetchOptions);
        } catch (e) {
            updateStatus('Server not responding', true);
            throw e;
        }

        if (response.status === 401) {
            // Token 被服务端拒绝（如服务器重启后 secret 变更）
            clearToken();
            await showAuthDialog();
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

        // 【关键修复】：必须在任何 await 之前同步禁用按钮，否则连点或回车会导致并发进入
        loginConfirmBtn.disabled = true;

        if (authExtCftraceEnabled && !rawCfTrace) {
            await fetchTrace();
        }

        authDialog.close();
        
        try {
            await doLogin(password, answer, questionIndex, rawCfTrace);
            await loadConfigs();
        } catch (e) {
            if (e.message === 'Unauthorized') {
                // 已在 doLogin 里 updateStatus，重新打开对话框并重新取题
                await showAuthDialog();
            }
        } finally {
            loginConfirmBtn.disabled = false;
        }
    });

    passwordInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') loginConfirmBtn.click();
    });

    setupConfirmBtn.addEventListener('click', async () => {
        if (setupConfirmBtn.disabled) return;

        const code = setupCodeInput.value.trim();
        const password = setupPasswordInput.value;
        const confirm = setupConfirmInput.value;

        if (!code || !password) {
            setSetupStatus('请填写 Setup Code 和新密码', true);
            return;
        }
        if (password.length < 8) {
            setSetupStatus('新密码至少 8 个字符', true);
            return;
        }
        if (password !== confirm) {
            setSetupStatus('两次输入的密码不一致', true);
            return;
        }

        setupConfirmBtn.disabled = true;
        setSetupStatus('正在初始化...');
        try {
            const res = await fetchWithTimeout('/api/v1/admin/setup', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ setup_code: code, password }),
            });

            if (res.status === 204) {
                window.location.reload();
                return;
            }
            if (res.status === 409) {
                setSetupStatus('管理员已初始化，正在刷新...', true);
                setTimeout(() => window.location.reload(), 1000);
                return;
            }
            if (res.status === 403) {
                setSetupStatus('Setup Code 错误或已过期', true);
            } else if (res.status === 400) {
                setSetupStatus('密码至少 8 个字符', true);
            } else {
                setSetupStatus('初始化失败，请稍后重试', true);
            }
        } catch (e) {
            setSetupStatus('服务器无响应', true);
        } finally {
            setupConfirmBtn.disabled = false;
        }
    });

    setupCodeInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') setupConfirmBtn.click();
    });
    setupPasswordInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') setupConfirmBtn.click();
    });
    setupConfirmInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') setupConfirmBtn.click();
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
                const div = document.createElement('div');
                div.setAttribute('slot', 'headline');
                div.textContent = cfg.name;
                item.appendChild(div);
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
            const res = await apiFetch(`/api/v1/admin/configs/${encodeURIComponent(name)}`);
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
            const res = await apiFetch(`/api/v1/admin/configs/${encodeURIComponent(currentFile)}`, {
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

    logoutBtn.addEventListener('click', async () => {
        if (jwtToken) {
            try {
                await fetchWithTimeout('/api/v1/admin/logout', {
                    method: 'POST',
                    headers: { 'Authorization': `Bearer ${jwtToken}` },
                });
            } catch (e) {
                console.warn('Admin: logout request failed', e);
            }
        }
        clearToken();
        window.location.reload();
    });

    // Initial load
    if (setupRequired) {
        clearToken();
        setTimeout(() => setupDialog.show(), 100);
    } else if (isTokenValid()) {
        loadConfigs();
    } else {
        clearToken();
        setTimeout(() => {
            if (authExtSecqEnabled) {
                refreshQuestion().catch(() => {}).finally(() => authDialog.show());
            } else {
                authDialog.show();
            }
        }, 100);
    }

    // ====== Navigation & Tabs ======
    const viewConfig = document.getElementById('viewConfig');
    const viewNote = document.getElementById('viewNote');
    let notesLoaded = false;

    function switchTab(tabName) {
        if (tabName === 'config') {
            viewConfig.style.display = 'block';
            viewNote.style.display = 'none';
            replaceWithTonal('navIndexBtn');
            replaceWithText('navNoteBtn');
        } else if (tabName === 'note') {
            viewConfig.style.display = 'none';
            viewNote.style.display = 'block';
            replaceWithTonal('navNoteBtn');
            replaceWithText('navIndexBtn');
            if (!notesLoaded) loadNotes();
        }
    }

    function replaceWithTonal(id) {
        const el = document.getElementById(id);
        if (el.tagName.toLowerCase() === 'md-filled-tonal-button') return;
        const newEl = document.createElement('md-filled-tonal-button');
        newEl.id = id;
        newEl.innerHTML = el.innerHTML;
        el.parentNode.replaceChild(newEl, el);
        bindNavEvents();
    }

    function replaceWithText(id) {
        const el = document.getElementById(id);
        if (el.tagName.toLowerCase() === 'md-text-button') return;
        const newEl = document.createElement('md-text-button');
        newEl.id = id;
        newEl.innerHTML = el.innerHTML;
        el.parentNode.replaceChild(newEl, el);
        bindNavEvents();
    }

    function bindNavEvents() {
        document.getElementById('navIndexBtn').addEventListener('click', () => switchTab('config'));
        document.getElementById('navNoteBtn').addEventListener('click', () => switchTab('note'));
    }
    bindNavEvents();

    // ====== Notes Management ======
    const notesContainer = document.getElementById('notesContainer');
    const addNoteFab = document.getElementById('addNoteFab');
    const editorOverlay = document.getElementById('editorOverlay');
    const closeEditorBtn = document.getElementById('closeEditorBtn');
    const saveNoteBtn = document.getElementById('saveNoteBtn');
    const editTitle = document.getElementById('editTitle');
    const tagInput = document.getElementById('tagInput');
    const selectedTagsSet = document.getElementById('selectedTagsSet');
    const availableTagsSet = document.getElementById('availableTagsSet');
    const editExcerpt = document.getElementById('editExcerpt');
    const editContent = document.getElementById('editContent');
    const editorTitleText = document.getElementById('editorTitleText');
    const noteEditorStatus = document.getElementById('noteEditorStatus');

    let currentEditingSlug = null;
    let currentTags = [];
    let allAvailableTags = new Set();

    function setNoteEditorStatus(msg, isError = false) {
        if (!noteEditorStatus) return;
        noteEditorStatus.textContent = msg;
        noteEditorStatus.style.color = isError
            ? 'var(--md-sys-color-error)'
            : 'var(--md-sys-color-on-surface-variant)';
    }

    function insertAtCursor(textarea, text) {
        const start = textarea.selectionStart ?? textarea.value.length;
        const end = textarea.selectionEnd ?? textarea.value.length;
        textarea.value = textarea.value.slice(0, start) + text + textarea.value.slice(end);
        textarea.selectionStart = textarea.selectionEnd = start + text.length;
        textarea.focus();
    }

    async function uploadNoteImage(file) {
        if (!isTokenValid()) {
            clearToken();
            await showAuthDialog();
            throw new Error('Unauthorized');
        }

        const headers = { 'Authorization': `Bearer ${jwtToken}` };
        if (file.type) headers['Content-Type'] = file.type;

        let response;
        try {
            response = await fetchWithTimeout('/admin/notes/images', {
                method: 'POST',
                headers,
                body: file,
            }, 30000);
        } catch (e) {
            setNoteEditorStatus('图片上传失败：网络错误', true);
            throw e;
        }

        if (response.status === 401) {
            clearToken();
            await showAuthDialog();
            throw new Error('Unauthorized');
        }

        if (!response.ok) {
            const msg = response.status === 413
                ? '图片上传失败：超过 5MB 限制'
                : '图片上传失败：仅支持 PNG / JPG / GIF / WebP';
            setNoteEditorStatus(msg, true);
            throw new Error(msg);
        }

        const data = await response.json();
        setNoteEditorStatus('图片已上传：' + data.url);
        return data;
    }

    async function loadNotes() {
        try {
            const res = await apiFetch('/admin/notes');
            const notes = await res.json();
            notesLoaded = true;
            renderNotes(notes);
        } catch (e) {
            console.warn('Failed to load notes', e);
        }
    }

    function renderNotes(notes) {
        if (!notesContainer) return;
        notesContainer.innerHTML = '';
        allAvailableTags.clear();

        notes.forEach(note => {
            (note.meta.tags || []).forEach(t => allAvailableTags.add(t));
            
            const card = document.createElement('div');
            card.className = 'admin-note-card';
            
            const title = document.createElement('h3');
            title.textContent = note.meta.title;
            
            const slug = document.createElement('div');
            slug.style.fontSize = '0.85rem';
            slug.style.color = 'var(--md-sys-color-outline)';
            slug.textContent = note.meta.slug;

            const actions = document.createElement('div');
            actions.className = 'admin-note-actions';
            
            const editBtn = document.createElement('md-filled-tonal-button');
            editBtn.textContent = '编辑';
            editBtn.addEventListener('click', () => openEditor(note.meta.slug));
            
            const delBtn = document.createElement('md-outlined-button');
            delBtn.textContent = '删除';
            delBtn.className = 'delete-btn';
            delBtn.addEventListener('click', () => deleteNote(note.meta.slug));

            actions.appendChild(delBtn);
            actions.appendChild(editBtn);
            
            card.appendChild(title);
            card.appendChild(slug);
            card.appendChild(actions);
            notesContainer.appendChild(card);
        });
    }

    async function deleteNote(slug) {
        if (!confirm(`确定要删除 ${slug} 吗？`)) return;
        try {
            await apiFetch(`/admin/notes/${encodeURIComponent(slug)}`, { method: 'DELETE' });
            loadNotes();
        } catch (e) {
            alert('删除出错');
        }
    }

    async function openEditor(slug = null) {
        currentEditingSlug = slug;
        
        editTitle.value = '';
        tagInput.value = '';
        currentTags = [];
        editExcerpt.value = '';
        editContent.value = '';
        
        if (slug) {
            editorTitleText.textContent = '编辑梨记';
            try {
                const res = await apiFetch(`/admin/notes/${encodeURIComponent(slug)}`);
                const data = await res.json();
                editTitle.value = data.title || '';
                currentTags = data.tags || [];
                editExcerpt.value = data.excerpt || '';
                editContent.value = data.content || '';
            } catch (e) {
                alert('加载出错');
                return;
            }
        } else {
            editorTitleText.textContent = '新建梨记';
        }
        
        renderSelectedTags();
        renderAvailableTags();
        
        editorOverlay.classList.add('active');
        document.body.style.overflow = 'hidden';
    }

    function renderSelectedTags() {
        selectedTagsSet.innerHTML = '';
        currentTags.forEach(tag => {
            const chip = document.createElement('md-input-chip');
            chip.label = tag;
            chip.addEventListener('remove', () => {
                currentTags = currentTags.filter(t => t !== tag);
                renderSelectedTags();
            });
            selectedTagsSet.appendChild(chip);
        });
    }

    function renderAvailableTags() {
        availableTagsSet.innerHTML = '';
        allAvailableTags.forEach(tag => {
            const chip = document.createElement('md-assist-chip');
            chip.label = tag;
            chip.addEventListener('click', () => {
                if (!currentTags.includes(tag)) {
                    currentTags.push(tag);
                    renderSelectedTags();
                }
            });
            availableTagsSet.appendChild(chip);
        });
    }

    if (tagInput) {
        tagInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                const tag = tagInput.value.trim();
                if (tag && !currentTags.includes(tag)) {
                    currentTags.push(tag);
                    renderSelectedTags();
                }
                tagInput.value = '';
            }
        });
    }

    if (editContent) {
        editContent.addEventListener('paste', async (e) => {
            const items = e.clipboardData && e.clipboardData.items;
            if (!items) return;

            let imageFile = null;
            for (let i = 0; i < items.length; i++) {
                const item = items[i];
                if (item.kind === 'file' && item.type.startsWith('image/')) {
                    imageFile = item.getAsFile();
                    break;
                }
            }
            if (!imageFile) return;

            e.preventDefault();
            setNoteEditorStatus('正在上传图片...');
            try {
                const data = await uploadNoteImage(imageFile);
                const alt = (imageFile.name || 'image').replace(/\.[^.]+$/, '');
                insertAtCursor(editContent, `![${alt}](${data.url})`);
            } catch (err) {
                console.warn('Admin: image paste failed', err);
            }
        });
    }

    function closeEditor() {
        editorOverlay.classList.remove('active');
        document.body.style.overflow = '';
    }

    if (addNoteFab) addNoteFab.addEventListener('click', () => openEditor(null));
    if (closeEditorBtn) closeEditorBtn.addEventListener('click', closeEditor);

    if (saveNoteBtn) {
        saveNoteBtn.addEventListener('click', async () => {
            saveNoteBtn.disabled = true;
            
            const payload = {
                title: editTitle.value,
                tags: currentTags,
                excerpt: editExcerpt.value.trim() ? editExcerpt.value.trim() : null,
                content: editContent.value
            };
            
            const method = currentEditingSlug ? 'PUT' : 'POST';
            const url = currentEditingSlug ? `/admin/notes/${encodeURIComponent(currentEditingSlug)}` : '/admin/notes';
            
            try {
                await apiFetch(url, {
                    method,
                    body: JSON.stringify(payload)
                });
                closeEditor();
                loadNotes();
            } catch (e) {
                alert('保存出错: ' + e.message);
            } finally {
                saveNoteBtn.disabled = false;
            }
        });
    }
});
