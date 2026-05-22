# 梨窝（lily-nest）
[![zread](https://img.shields.io/badge/Ask_Zread-_.svg?style=flat&color=00b0aa&labelColor=000000&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQuOTYxNTYgMS42MDAxSDIuMjQxNTZDMS44ODgxIDEuNjAwMSAxLjYwMTU2IDEuODg2NjQgMS42MDE1NiAyLjI0MDFWNC45NjAxQzEuNjAxNTYgNS4zMTM1NiAxLjg4ODEgNS42MDAxIDIuMjQxNTYgNS42MDAxSDQuOTYxNTZDNS4zMTUwMiA1LjYwMDEgNS42MDE1NiA1LjMxMzU2IDUuNjAxNTYgNC45NjAxVjIuMjQwMUM1LjYwMTU2IDEuODg2NjQgNS4zMTUwMiAxLjYwMDEgNC45NjE1NiAxLjYwMDFaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00Ljk2MTU2IDEwLjM5OTlIMi4yNDE1NkMxLjg4ODEgMTAuMzk5OSAxLjYwMTU2IDEwLjY4NjQgMS42MDE1NiAxMS4wMzk5VjEzLjc1OTlDMS42MDE1NiAxNC4xMTM0IDEuODg4MSAxNC4zOTk5IDIuMjQxNTYgMTQuMzk5OUg0Ljk2MTU2QzUuMzE1MDIgMTQuMzk5OSA1LjYwMTU2IDE0LjExMzQgNS42MDE1NiAxMy43NTk5VjExLjAzOTlDNS42MDE1NiAxMC42ODY0IDUuMzE1MDIgMTAuMzk5OSA0Ljk2MTU2IDEwLjM5OTlaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik0xMy43NTg0IDEuNjAwMUgxMS4wMzg0QzEwLjY4NSAxLjYwMDEgMTAuMzk4NCAxLjg4NjY0IDEwLjM5ODQgMi4yNDAxVjQuOTYwMUMxMC4zOTg0IDUuMzEzNTYgMTAuNjg1IDUuNjAwMSAxMS4wMzg0IDUuNjAwMUgxMy43NTg0QzE0LjExMTkgNS42MDAxIDE0LjM5ODQgNS4zMTM1NiAxNC4zOTg0IDQuOTYwMVYyLjI0MDFDMTQuMzk4NCAxLjg4NjY0IDE0LjExMTkgMS42MDAxIDEzLjc1ODQgMS42MDAxWiIgZmlsbD0iI2ZmZiIvPgo8cGF0aCBkPSJNNCAxMkwxMiA0TDQgMTJaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00IDEyTDEyIDQiIHN0cm9rZT0iI2ZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPgo8L3N2Zz4K&logoColor=ffffff)](https://zread.ai/Sulytwelve/lily-nest)
> 梨梨的个人网站：项目展示、博客与技术分享

## 项目预览
- www.sulyhub.cn

## 项目简介
梨窝是一个基于 Rust + Axum 的个人主页/作品集网站，支持项目动态加载、团队成员展示、深浅色主题等功能，界面采用 Material You 风格，支持响应式设计。

当前版本已添加更明确的资源路由与应用路由定义，进一步强化静态资源服务与安全策略。

## 技术栈
- Rust 2024
- [Axum](https://github.com/tokio-rs/axum) Web 框架
- Tokio 异步运行时
- axum-server + rustls（TLS 支持）
- Serde / TOML 配置
- Tower HTTP 静态资源服务
- tracing + tracing-subscriber 日志
- 前端：原生 Material Design、@material/web 组件、本地 Material 3 主题 CSS

## 主要功能
- 首页动态渲染（项目、团队成员、关于我）
- **后台管理面板（/admin）：支持在线编辑 TOML 配置文件（site.toml, projects.toml, about.toml 等）**
- 配置文件驱动（config.toml、site.toml、projects.toml、about.toml）
- config.toml 中的 [security] 块支持动态 CORS、CSP、Permissions Policy 及 **管理员密码** 配置
- RESTful API（/api/v1/health、/api/v1/home/profile、/api/v1/admin/*）
- 静态资源服务（图片、CSS、JS、robots.txt、sitemap.xml 等）
- 静态资源预压缩（Gzip、Brotli、Zstd）
- 深色主题跟随系统（纯 CSS，无 JS 闪烁）
- HTML 页面缓存（release 模式，5 分钟）
- HTTP 安全头（CSP、HSTS、X-Frame-Options、Permissions-Policy 等）
- 安全配置加载增强：release 模式缓存 security 配置，debug 模式仍会热加载；非法 origin 会跳过并记录错误日志
- release 模式强制 HTTPS，无证书直接拒绝启动
- 应用路由与资源路由已拆分：
  - App router 提供 `/`、`/index.html` 重定向、`/admin`、`/api/v1/*` 和安全头中间件
  - 静态资源 router 提供 `/robots.txt`、`/favicon.ico`、`/css/*`、`/js/*` 等

## 项目结构
```
lily-nest/
├── Cargo.toml
├── config.toml               # 站点基础配置（证书、安全策略、管理员密码）
├── site.toml                 # 站点基础信息配置
├── projects.toml             # 项目列表配置
├── about.toml                # 关于我列表配置
├── MWC/                      # Material Web Components 构建环境
├── certs/                    # SSL 证书目录
├── src/
│   ├── app.rs                # 应用路由与中间件
│   ├── routes/
│   │   ├── admin.rs          # 后台管理路由逻辑
│   │   ├── api.rs            # API 路由
│   │   └── home.rs           # 首页渲染
│   └── ...
├── static/
│   ├── css/
│   │   ├── admin.css         # 后台自定义样式
│   │   └── ...
│   ├── js/
│   │   ├── admin.js          # 后台交互逻辑
│   │   ├── MaterialWeb.js    # 核心组件库（本地构建）
│   │   └── ...
└── ...
```

## 管理后台 (/admin)
项目内置了一个基于 Material Design 3 的管理后台，允许管理员直接在浏览器中修改站点内容：
- **安全验证**：通过请求头 `X-Admin-Password` 进行验证。
- **密码设置**：在 `config.toml` 的 `[security]` 块中设置 `admin_password`。
- **文件限制**：仅允许编辑内容相关的 `.toml` 文件，自动排除 `config.toml` 和 `Cargo.toml` 以保系统安全。

> **安全提示**：在 Debug 模式（HTTP）下，密码以明文传输，仅建议在本地开发环境使用。在生产环境（Release 模式）下，必须配置 HTTPS 以确保传输加密。

## 启动方式

1. 安装 Rust（建议最新稳定版）
2. 克隆本仓库并进入目录
3. **开发模式（HTTP，无需证书）：**
   ```bash
   cargo run
   ```
   访问 [http://[::1]:8880](http://[::1]:8880)

4. **生产模式（HTTPS，必须配置证书）：**
   - 将证书与私钥放入 `certs/` 目录
   - 在 `config.toml` 中配置证书路径
   ```bash
   cargo run --release
   ```
   访问 [https://[::1]:8443](https://[::1]:8443)

> **注意：** release 模式下若未配置证书，程序会直接 panic 拒绝启动。

## 配置说明
- `config.toml`：TLS 证书路径、[security] 安全策略（CORS、CSP、Permissions Policy）
- `site.toml`：站点基础信息，使用 `[site]` 表配置
- `projects.toml`：项目列表
- `about.toml`：关于我
- `static/`：静态资源（图片、CSS、JS、robots.txt 等）

## 安全特性
- URL 协议校验：仅允许 `/` 和 `http://` 以及 `https://` 开头的链接，防止 `javascript:` XSS 注入
- HTML 转义：所有配置内容插入页面前均转义
- HTTP 安全响应头：CSP、HSTS、X-Content-Type-Options、X-Frame-Options、Referrer-Policy、Permissions-Policy
- 安全配置解析错误会记录为错误日志；release 模式使用缓存的 security config，debug 模式支持热加载
- release 模式强制 TLS，不支持 HTTP 回退

## 亮点与注意事项
- debug 模式每次请求重新渲染页面，方便开发调试
- release 模式使用内存缓存，首页渲染结果复用（5 分钟 Cache-Control）
- 静态资源预压缩：支持 Gzip 和 Brotli 压缩，自动检测文件更新并重新压缩，提高网站性能
- 深色主题完全由 CSS `@media (prefers-color-scheme: dark)` 驱动，无 JS 依赖，无闪烁
- 前端资源基于 Material Design 3 规范，使用 `@material/web` 组件库本地构建
- 项目部署于 Cloudflare，开放 8443（HTTPS）和 8880（HTTP dev）端口
- 仅供个人学习/展示用途，欢迎二次开发

## License
MIT
