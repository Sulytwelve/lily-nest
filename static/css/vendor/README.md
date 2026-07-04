# 🎨 Vendor CSS 资源目录 (static/css/vendor)

本目录用于存放第三方样式表文件（如 Highlight.js 代码主题、KaTeX 公式样式等）。

## 💡 使用说明

1. **路由回退**：前端请求 `/css/xxx.css` 时，若主目录不存在，系统会自动回退寻址至本目录。
2. **预压缩**：本目录下所有 `.css` 均受到 `compressor.rs` 支持，开启后自动生成 `.br`、`.zst` 和 `.gz`。
3. **放置建议**：
   - **代码语法高亮**：下载 `atom-one-dark.min.css` 等样式放置至 `static/css/vendor/atom-one-dark.min.css`。
   - **KaTeX 公式**：下载 `katex.min.css` 放置于此。
