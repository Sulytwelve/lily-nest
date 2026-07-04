# 📦 Vendor JavaScript 资源目录 (static/js/vendor)

本目录用于存放第三方、非原创或自构建的本地化 JavaScript 静态库文件。

## ✨ 架构说明与核心特性

1. **路由无感回退 (Fallback Routing)**：
   在后端静态资源服务 (`src/routes/static_assets.rs`) 中，已为 `/js` 挂载了本目录作为自动回退寻址路径。
   前端模板中直接引用 `/js/xxx.js`（或显式使用 `/js/vendor/xxx.js`），后端均能自动精准寻址并返回本目录下的对应文件，无需更改任何 HTML 脚本路径。
2. **自动预压缩 (Pre-compression)**：
   本目录已被纳入 `assets_dirs` 扫描范围。只要在 `config.toml` 中开启 `precompress = true`，系统在启动时会自动将本目录下的 `.js` 文件预压缩为 `.br`、`.zst` 和 `.gz` 格式。
3. **屏蔽 Git 统计与 Diff (Linguist Vendored)**：
   在根目录 `.gitattributes` 中已配置 `static/js/vendor/** linguist-vendored`，放入此处的第三方 JS 文件不会干扰 GitHub 仓库的语言统计，且在查看 diff 时会自动忽略。

---

## 🚀 第三方库下载与放置说明

为优化国内访问速度并遵循“够用就好”的纯粹静态化策略，请根据需要自行下载第三方前端库并放置于此目录中：

### 1. Highlight.js（代码语法高亮）⚠️ 重要
如果你想弃用外部 CDN 彻底改为本地加载，请自行下载 Highlight.js 核心脚本并放置于此处：
* **目标位置**：`static/js/vendor/highlight.min.js`
* **推荐下载地址**：[Highlight.js CDN Release (v11.10.0)](https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.10.0/build/highlight.min.js)
* **配套样式**：对应的代码高亮 CSS 样式（如 `atom-one-dark.min.css`），请下载至 `static/css/vendor/atom-one-dark.min.css`。

### 2. KaTeX（数学公式渲染）
如需本地化托管 LaTeX 数学公式渲染引擎，请将核心脚本放入本目录：
* **目标位置**：
  * `static/js/vendor/katex.min.js`
  * `static/js/vendor/katex-auto-render.min.js` (对应 contrib/auto-render.min.js)
* **配套资源**：其 CSS 样式请放至 `static/css/vendor/`，其配套 `woff/woff2` 字体文件请放至 `static/fonts/vendor/`。

### 3. Material Web Components (MWC)
* **目标位置**：`static/js/vendor/MaterialWeb.js`
* **生成方式**：进入项目根目录下的 `MWC/` 目录，执行 `npm run build` 即可自动通过 Rollup 构建并将产物直接输出至本目录。
