# 🔤 Vendor Fonts 资源目录 (static/fonts/vendor)

本目录用于存放第三方字体文件（如 KaTeX 渲染所需的 `woff/woff2/ttf` 字体文件）。

## 💡 使用说明

1. **自动寻址**：配合 `/css/vendor/katex.min.css` 中相对路径 `fonts/KaTeX_...woff2` 的引用，字体会被正确路由加载。
2. **预压缩**：本目录下所有字体文件 (`woff`, `woff2`, `ttf`, `otf`) 在开启预压缩时将自动被压缩加速。
