const shareBtn = document.body.querySelector('#shareBtn');

if (shareBtn) {
  shareBtn.addEventListener('click', async () => {
    // 1. 尝试触发系统原生分享 API
    if (navigator.share) {
      try {
        await navigator.share({
          title: document.title,
          url: window.location.href
        });
        return;
      } catch (err) {
        if (err.name !== 'AbortError') {
          console.log('分享出错:', err);
        }
        return; // 用户主动取消或者报错，终止执行
      }
    } 
    
    // 2. 退化方案：如果不支持原生分享 (如 PC 端或非 HTTPS 局域网访问)，则复制链接
    try {
      if (navigator.clipboard && window.isSecureContext) {
        // HTTPS 或 localhost 下的现代 API
        await navigator.clipboard.writeText(window.location.href);
      } else {
        // 兼容非 HTTPS 局域网访问的传统 API
        const textArea = document.createElement("textarea");
        textArea.value = window.location.href;
        textArea.style.position = "fixed";
        textArea.style.left = "-9999px";
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
      }
      alert('文章链接已复制到剪贴板！');
    } catch (err) {
      alert('无法自动复制，请手动复制浏览器地址栏。');
    }
  });
}

const copyMdBtn = document.body.querySelector('#copyMdBtn');
if (copyMdBtn) {
  copyMdBtn.addEventListener('click', async () => {
    try {
      const url = window.location.pathname + '?format=markdown';
      const fetchOptions = { headers: { 'Accept': 'text/markdown' } };
      if (!fetchOptions.signal) {
        fetchOptions.signal = AbortSignal.timeout(15000);
      }
      const res = await fetch(url, fetchOptions);
      if (!res.ok) throw new Error('Network response was not ok');
      const mdContent = await res.text();

      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(mdContent);
      } else {
        const textArea = document.createElement("textarea");
        textArea.value = mdContent;
        textArea.style.position = "fixed";
        textArea.style.left = "-9999px";
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
      }
      alert('Markdown 原文 (包含元数据) 已复制到剪贴板！');
    } catch (err) {
      console.error(err);
      alert('获取或复制 Markdown 失败。');
    }
  });
}

// KaTeX / Highlight.js 初始化（CSP 合规：不再依赖内联 onload / 内联 script）
document.addEventListener('DOMContentLoaded', () => {
  if (window.hljs) {
    try {
      hljs.highlightAll();
    } catch (e) {
      console.warn('Highlight.js init failed:', e);
    }
  }

  if (window.renderMathInElement) {
    try {
      renderMathInElement(document.body, {
        delimiters: [
          {left: '$$', right: '$$', display: true},
          {left: '$', right: '$', display: false},
          {left: '\\(', right: '\\)', display: false},
          {left: '\\[', right: '\\]', display: true}
        ],
        throwOnError: false
      });
    } catch (e) {}
  } else if (window.katex) {
    document.querySelectorAll('.math-inline').forEach(el => {
      try { katex.render(el.textContent, el, { displayMode: false, throwOnError: false }); } catch(e){}
    });
    document.querySelectorAll('.math-display').forEach(el => {
      try { katex.render(el.textContent, el, { displayMode: true, throwOnError: false }); } catch(e){}
    });
  }
});
