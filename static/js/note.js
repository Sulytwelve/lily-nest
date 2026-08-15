    const searchResultsList = document.getElementById('searchResultsList');
    const searchFab = document.getElementById('searchFab');
    const searchOverlay = document.getElementById('searchOverlay');
    const searchInput = document.getElementById('searchInput');

    let publicNotes = [];
    try {
      const dataEl = document.getElementById('notes-data');
      if (dataEl) {
        publicNotes = JSON.parse(dataEl.textContent);
      }
    } catch(e) {
      console.error("Error parsing notes JSON:", e);
    }

    // --- 通用的渲染笔记卡片方法 (用于搜索结果) ---
    function escapeHTML(str) {
      if (!str) return '';
      return String(str).replace(/[&<>'"]/g, tag => ({
          '&': '&amp;',
          '<': '&lt;',
          '>': '&gt;',
          "'": '&#39;',
          '"': '&quot;'
      }[tag] || tag));
    }

    // 统一的 KaTeX 渲染入口（供初始渲染与搜索结果复用）
    function renderMath(container) {
      if (!window.renderMathInElement) return;
      try {
        renderMathInElement(container, {
          delimiters: [
            {left: '$$', right: '$$', display: true},
            {left: '$', right: '$', display: false},
            {left: '\\(', right: '\\)', display: false},
            {left: '\\[', right: '\\]', display: true}
          ],
          throwOnError: false
        });
      } catch(e) {}
    }

    function renderNotes(notesToRender, containerElement) {
      containerElement.innerHTML = '';
      
      if (notesToRender.length === 0) {
        containerElement.innerHTML = '<div class="empty-state">No notes found.</div>';
        return;
      }

      notesToRender.forEach(note => {
        const card = document.createElement('a');
        card.className = 'note-card';
        card.href = `/note/${encodeURIComponent(note.meta.slug)}`;
        card.style.textDecoration = 'none';
        card.style.color = 'inherit';
        
        let displayDate = note.meta.date;
        try {
          const d = new Date(displayDate);
          if (!isNaN(d.getTime())) {
            const pad = (n) => n.toString().padStart(2, '0');
            displayDate = `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
          }
        } catch(e) {}
        
        const plainText = (note.meta.excerpt || '').replace(/[#*_>\[\]\n`]/g, ' ').substring(0, 120) + '...';
        const tags = note.meta.tags || [];
        const tagsHtml = tags.map(tag => `<span class="tag">#${escapeHTML(tag)}</span>`).join('');

        card.innerHTML = `
          <div class="card-meta">
            <span>${escapeHTML(displayDate)}</span>
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6v2h8.59L5 17.59 6.41 19 16 9.41V18h2V6z"/></svg>
          </div>
          <div class="note-title">${escapeHTML(note.meta.title)}</div>
          <div class="note-excerpt">${escapeHTML(plainText)}</div>
          <div class="card-tags">${tagsHtml}</div>
        `;
        containerElement.appendChild(card);
      });

      renderMath(containerElement);
    }

    // --- 搜索遮罩逻辑 ---
    searchFab.onclick = () => {
      searchOverlay.classList.add('active');
      document.body.classList.add('no-scroll');
      setTimeout(() => searchInput.focus(), 100);
      
      if (searchInput.value.trim() === '') {
        renderNotes(publicNotes, searchResultsList);
      }
    };

    function closeSearch() {
      searchOverlay.classList.remove('active');
      document.body.classList.remove('no-scroll');
    }

    searchOverlay.addEventListener('click', (e) => {
      if (e.target === searchOverlay || e.target.classList.contains('search-results-area') || e.target === searchResultsList) {
        closeSearch();
      }
    });

    searchInput.addEventListener('input', (e) => {
      const query = e.target.value.toLowerCase().trim();
      
      if (!query) {
        renderNotes(publicNotes, searchResultsList);
        return;
      }

      const filtered = publicNotes.filter(note => {
        const titleMatch = note.meta.title.toLowerCase().includes(query);
        const excerptMatch = (note.meta.excerpt || '').toLowerCase().includes(query);
        const tags = note.meta.tags || [];
        const tagMatch = tags.some(tag => tag.toLowerCase().includes(query));
        return titleMatch || excerptMatch || tagMatch;
      });
      renderNotes(filtered, searchResultsList);
    });

    // ESC 键全局关闭逻辑
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        if (searchOverlay.classList.contains('active')) {
          closeSearch();
        }
      }
    });

    // 列表页初始渲染：note.js 是 defer 脚本，按文档顺序在 katex 的 defer 脚本之后执行，
    // 此时 renderMathInElement 已可用，直接对整页渲染一次，让卡片公式初始即显示。
    if (window.renderMathInElement) {
      renderMath(document.body);
    }
