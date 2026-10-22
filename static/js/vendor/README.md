# 本地前端依赖

本目录保存 Material Web、KaTeX 和 Highlight.js 的浏览器产物。版本、下载来源和完整性由项目根目录 `frontend/package.json` 与 `frontend/package-lock.json` 管理；Material Web 的组件入口在 `frontend/index.js`。

重新生成资源：

```sh
cd frontend
npm ci
npm run build
npm test
```

需要更新依赖时，在 `frontend/` 中更新 npm 版本与锁文件，再运行构建和 `npm audit`。构建同时更新 KaTeX 核心、自动渲染脚本、CSS 和字体，以及 Highlight.js 脚本和主题；不要单独替换其中一个版本。Node.js 和 npm 仅用于更新静态资源，Rust 服务运行时不需要它们。

第三方许可保存在 [licenses/](licenses/)，Material Web 合并产物保留源码许可注释。

## 静态资源服务

- `/js` 支持回退到本目录，也可显式使用 `/js/vendor/`。
- KaTeX 字体位于 `static/css/vendor/fonts/`，相对其 CSS 寻址。
- 启用预压缩后，服务启动时为配置目录中的 JS/CSS 生成 `.br`、`.zst` 和 `.gz`。
- Git Linguist 将本目录标为第三方代码，不计入项目语言统计。
