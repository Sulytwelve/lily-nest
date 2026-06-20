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
      const res = await fetch(url, { headers: { 'Accept': 'text/markdown' } });
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
