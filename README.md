# 梨窝（lily-nest）
[![zread](https://img.shields.io/badge/Ask_Zread-_.svg?style=flat&color=00b0aa&labelColor=000000&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQuOTYxNTYgMS42MDAxSDIuMjQxNTZDMS44ODgxIDEuNjAwMSAxLjYwMTU2IDEuODg2NjQgMS42MDE1NiAyLjI0MDFWNC45NjAxQzEuNjAxNTYgNS4zMTM1NiAxLjg4ODEgNS42MDAxIDIuMjQxNTYgNS42MDAxSDQuOTYxNTZDNS4zMTUwMiA1LjYwMDEgNS42MDE1NiA1LjMxMzU2IDUuNjAxNTYgNC45NjAxVjIuMjQwMUM1LjYwMTU2IDEuODg2NjQgNS4zMTUwMiAxLjYwMDEgNC45NjE1NiAxLjYwMDFaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00Ljk2MTU2IDEwLjM5OTlIMi4yNDE1NkMxLjg4ODEgMTAuMzk5OSAxLjYwMTU2IDEwLjY4NjQgMS42MDE1NiAxMS4wMzk5VjEzLjc1OTlDMS42MDE1NiAxNC4xMTM0IDEuODg4MSAxNC4zOTk5IDIuMjQxNTYgMTQuMzk5OUg0Ljk2MTU2QzUuMzE1MDIgMTQuMzk5OSA1LjYwMTU2IDE0LjExMzQgNS42MDE1NiAxMy43NTk5VjExLjAzOTlDNS42MDE1NiAxMC42ODY0IDUuMzE1MDIgMTAuMzk5OSA0Ljk2MTU2IDEwLjM5OTlaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik0xMy43NTg0IDEuNjAwMUgxMS4wMzg0QzEwLjY4NSAxLjYwMDEgMTAuMzk4NCAxLjg4NjY0IDEwLjM5ODQgMi4yNDAxVjQuOTYwMUMxMC4zOTg0IDUuMzEzNTYgMTAuNjg1IDUuNjAwMSAxMS4wMzg0IDUuNjAwMUgxMy43NTg0QzE0LjExMTkgNS42MDAxIDE0LjM5ODQgNS4zMTM1NiAxNC4zOTg0IDQuOTYwMVYyLjI0MDFDMTQuMzk4NCAxLjg4NjY0IDE0LjExMTkgMS42MDAxIDEzLjc1ODQgMS42MDAxWiIgZmlsbD0iI2ZmZiIvPgo8cGF0aCBkPSJNNCAxMkwxMiA0TDQgMTJaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00IDEyTDEyIDQiIHN0cm9rZT0iI2ZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPgo8L3N2Zz4K&logoColor=ffffff)](https://zread.ai/Sulytwelve/lily-nest)
> 梨梨的个人网站：项目展示、博客与技术分享

## 项目预览
- www.sulyhub.cn

## 项目简介
梨窝是一个基于 Rust + Axum 的个人主页/作品集网站，支持项目动态加载、团队成员展示、深浅色主题等功能，界面采用 Material You 风格，支持响应式设计。

当前版本已添加更明确的资源路由与应用路由定义，进一步强化静态资源服务与安全策略。

## 技术指标
在极低配置的嵌入式设备（如玩客云、香橙派等 armv7l 架构单板电脑）上，该项目展现出极致的系统开销与运行效率：
- **极低内存常驻**：冷启动仅需 **600 KB** 内存；在经历多次配置动态加载与高频访问后，常驻内存依然稳定保持在 **2.3 MB** 左右。
- **高并发与零分配**：核心页面与静态资源通过引用计数 `bytes::Bytes` 实现零堆分配、零拷贝（Zero-Copy）渲染，并在 7840HS 环境下实现 QPS 达 11.1 万 (HTTP) / 5.9 万 (HTTPS) 的极致吞吐。
- **三级缓存优化**：通过内存级预编译缓存、HTTP Last-Modified/If-Modified-Since 的 304 协商响应，配合 CDN 缓存，最大化节省服务器与网卡开销（无图本地响应耗时约 500ms+；公网环境下受限于大体积 Web Components JS 与图片资源加载时间，总计耗时约数秒）。
- **高度可控编译体积**：默认采用标准配置编译；在为受限目标环境（如玩客云）定制编译时，可通过手动开启 LTO（Link-Time Optimization）与符号裁剪，将 Release 二进制压缩至约 **5.0 MB**。
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
- HTML 页面缓存（release 模式，默认 1 小时，可在 config.toml 自定义），支持 If-Modified-Since → 304（使用 `bytes::Bytes` 实现零分配与零拷贝，预计算并缓存 HTTP 日期 HeaderValue）
- 静态资源缓存控制：支持根据 config.toml 的 `[assets]` 集中配置 HTML、API、JS/CSS、图片、字体等缓存时间，完美分流与解耦
- 静态资源支持 Last-Modified + 304 条件请求，CDN 友好
- HTTP 安全头（CSP、HSTS、X-Content-Type-Options、X-Frame-Options、Permissions-Policy 等）
- RFC 9116 安全披露文件支持：配置并读取 `/.well-known/security.txt` 自适应缓存与规范化响应。
- 动态 baidu 所有权验证支持：无需修改代码与重编译，直接丢入 `baidu_verify_codeva-*.html` 校验文件即可开箱即用（`0.2.6` 引入）。
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
├── site.toml                 # 站点基础信息配置（SEO元数据 [site] 及 个人主页 [profile]）
├── projects.toml             # 项目列表配置
├── about.toml                # 关于我列表配置
├── MWC/                      # Material Web Components 构建环境
├── certs/                    # SSL 证书目录
├── src/
│   ├── main.rs               # 应用入口
│   ├── app.rs                # 路由编织与全局中间件
│   ├── state.rs              # 全局共享状态 AppState
│   ├── routes/
│   │   ├── mod.rs            # 路由模块定义
│   │   ├── home.rs           # 首页路由（含 304 支持）
│   │   ├── api.rs            # 公开 RESTful API 路由（含 admin auth 中间件）
│   │   ├── admin.rs           # 后台管理页面路由
│   │   └── static_assets.rs  # 静态资源服务路由（含 Last-Modified + 304）
│   ├── render.rs             # 纯 HTML 渲染逻辑与占位符替换
│   ├── middlewares.rs        # 全局 HTTP 中间件（CORS 与安全头）
│   ├── config.rs             # 配置文件解析加载
│   ├── model.rs              # 数据模型定义
│   ├── compressor.rs         # 静态资源预压缩工具
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
- **安全验证**：基于 JWT (JSON Web Token) 的无状态鉴权机制，登录后 token 仅存储于当前标签页 (`sessionStorage`)，关闭即失效。
- **密码设置**：在 `config.toml` 的 `[security]` 块中设置 `admin_password`。
- **密码强制锁定**：若 `admin_password` 未设置、为空或保留默认的 `"CHANGE_YOUR_PASSWORD"`，系统将全面禁用登录功能并在服务器日志中输出 `error!` 级别的安全警告，强制保障管理权限不被任意滥用。
- **多层防护**：内置严格的限流机制（Rate Limit），并支持基于 CF Trace 的深度校验及自定义安全问题二次验证。
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
   > **关于自签测试证书的提示：** 仓库中默认内置了针对 `example.com` 的 5 年自签证书仅用于本地快速开发与测试验证，**严禁**直接作为你自己的生产证书使用，梨梨（Sulytwelve）本人也绝不会在生产中使用该证书。公网部署请务必自行生成并替换你的专属证书！

5. **生产模式（强制 HTTP，无需证书）：**
   - 如果你计划在 Release 模式下运行但不配置 TLS 证书（例如部署在 Caddy、Nginx 或 Cloudflare Tunnel 等反向代理后面做 TLS 终止），可以使用 `force-http` 特性来强制以 HTTP 启动：
   ```bash
   cargo run --release --features force-http
   ```
   > **安全警告：** 强制 HTTP 模式下，管理后台的密码在传输时将以明文流转。请务必确保该服务只运行在受保护的内部局域网，或者前置代理已经启用了合法的 TLS！

> **注意：** 默认情况下，release 模式下若未配置证书，且未指定 `force-http` 特性时，程序会直接 panic 拒绝启动。

## 配置说明
- `config.toml`：TLS 证书路径、`[security]` 安全策略（CORS、CSP、Permissions Policy、管理员密码等）、`[assets]` 预压缩配置与各类别（HTML、API、JS/CSS、图片、字体等）缓存时长解耦配置
- `site.toml`：站点配置与展示内容，已拆分为两部分：
  - `[site]`：页面元数据（标题 `index_title` 与 `meta_desc` 元描述信息）
  - `[profile]`：个人主页卡片内容（包括头像、背景、团队成员、自我介绍等）
- `projects.toml`：项目列表
- `about.toml`：关于我
- `static/`：静态资源（图片、CSS、JS、robots.txt 等）

## 安全特性
- URL 协议校验：仅允许 `/` 和 `http://` 以及 `https://` 开头的链接，防止 `javascript:` XSS 注入
- HTML 转义：所有配置内容插入页面前均转义
- HTTP 安全响应头：CSP、HSTS、X-Content-Type-Options、X-Frame-Options、Referrer-Policy、Permissions-Policy
- 安全配置解析错误会记录为错误日志；release 模式使用缓存的 security config，debug 模式支持热加载
- release 模式强制 TLS，不支持 HTTP 回退
- 默认密码安全锁定：强制过滤并禁用未设置、为空或为默认占位符 `"CHANGE_YOUR_PASSWORD"` 的管理员密码，并在发生登录尝试时在服务器控制台抛出 `error!` 级别警报日志，防止站点被爆破

## 亮点与注意事项
- debug 模式每次请求重新渲染页面，方便开发调试
- release 模式使用内存缓存，首页渲染结果复用（5 分钟 Cache-Control）
- 静态资源预压缩（默认关闭）：在 `config.toml` 的 `[assets]` 中设置 `precompress = true` 以开启，支持 Gzip、Brotli 和 Zstd 压缩，自动检测文件更新并重新压缩
- 深色主题完全由 CSS `@media (prefers-color-scheme: dark)` 驱动，无 JS 依赖，无闪烁
- 前端资源基于 Material Design 3 规范，使用 `@material/web` 组件库本地构建
- 项目部署于 Cloudflare，开放 8443（HTTPS）和 8880（HTTP dev）端口
  > **关于 Cloudflare 边缘缓存的重要提示：**  
  > Cloudflare 默认对部分非标准端口（包括 `8443`、`8880` 等，注意 `8080` 虽为非标端口但支持缓存）**硬性禁用了边缘缓存**（Free/Pro 计划会一直返回 `cf-cache-status: DYNAMIC`）。  
  >   
  > **🎉 缓存难题解决：** 在版本 [`3ad622c`](https://github.com/Sulytwelve/lily-nest/commit/3ad622c710db8421868019bd183f9c6d376510f8) 之后，梨梨（Sulytwelve）已将原有的 `8443` 非标端口回源方案改为了 **Cloudflare Tunnel (`cloudflared`)** 方案，将 `lily-nest` 的端口设置为 `https_port = 443` 并允许防火墙 `443` 端口出站。在完成此次变更后，网站已成功实现了所有静态资源（如 JS/CSS 等）在 Cloudflare 边缘节点的 `HIT`（缓存命中）！
- 仅供个人学习/展示用途，欢迎二次开发

## 设计哲学与开发历程
项目最初是梨梨在 2026 年初在试验场（`suly-nexus`）启动的。在经历了一番技术选型的纠结、因为个人事务短暂沉寂，以及后来的高效重构后，终于诞生了如今的 `lily-nest`。

当时为了快速验证想法，梨梨只花了 3 天时间写完核心 MVP，总计 5 天就把项目成功上线到 Cloudflare。以下是具体的开发历程：

- **2026-02-16**：装好 Neovim (AstroNvim) 开发环境，开始折腾 Dioxus、Leptos 及 Tauri 等流行前端模板框架。最初梨梨特别仰慕这些“新技术”啦，但在实际使用中发现它们普遍需要在宏里书写极其繁琐且难以维护的代码。梨梨是自学过来的，不想把自己绕进那些繁琐的宏里，综合考虑后，决定避开复杂的框架，选择最纯粹的手写 HTML/JS/CSS，配合后端极简的占位符拼接。
- **2026-02-23**：对项目进行“砍功能”的减法提炼，排除过度设计的冗余功能，正式创建 `lily-nest` 并初始化骨架，开始核心 MVP 开发。
- **2026-02-26**：搞定了后端核心数据模型、路由逻辑拼装以及前端页面设计，完成首个可用版本打包备份（当时还是裸 HTTP 方案）。
- **2026-02-28**：将项目顺利打包部署并上线至 Cloudflare 生产环境。
- **2026-03-19**：当前 GitHub 仓库的第一个公开 Commit。其实在 2 月 28 日项目上线后到 3 月 19 日这段时间，因为处理梨梨的一些个人事务，项目沉寂了一段时间。发布前，梨梨对代码进行了彻底的安全脱敏（清理敏感路径与测试私钥信息），确保代码能以最干净、规范的工程形态开源。
- **2026-03-22**：回归后立即启动高频迭代，首先实现了 **HTTPS / TLS 证书动态加载** 支持。
- **2026-03-28**：全面采用 **Material Web Components (MWC)** 重构前端页面模板与后端渲染架构，奠定了极现代的 Material You 风格基调。
- **2026-03-29 至 04-01**：爆发式重构并加入了一系列核心安全特性，包括 **URL 协议清洗（防范 XSS 注入）、CORS 跨域预检、严格的 CSP 内容安全策略、管理员动态安全密码管理**等。
- **2026-04-08**：静态资源全面支持 **Gzip 与 Brotli 预压缩**，大幅压榨静态资产传输开销。
- **2026-05-20 至 05-22**：迎来**安全策略与管理面板重磅升级**，支持多重安全提问校验、Cloudflare Trace 可信源二次校验，以及动态编辑 site/about/projects 的后台 TOML 管理面板。
- **2026-05-30 至今**：引入页面内存级**零拷贝（Zero-Copy）预缓存响应机制**与 RFC 9116 规范的 `security.txt` 披露支持，并坚守服务端渲染文本以对 AI 采集器与爬虫完美兼容。

## 梨梨的悄悄话：这个项目是怎么被雕琢出来的？
这个项目，是梨梨用 Rust 一点点抠出来的。可能有人觉得它只是个普通的个人主页，但在你看不到的底层，梨梨花了很多心思。

梨梨不想把它写得太夸张，但它在服务器上的表现确实让梨梨挺开心的：
* **不爱占地方**：冷启动时，它在服务器上只吃 **600 KB** 内存；在玩客云这样羸弱的 ARM 设备上跑了快两天、后台存了多次配置后，常驻内存也稳稳地守在 **2.3 MB** 左右。比那些动辄几十上百兆的大家伙要老实多了。
* **零拷贝与零堆分配**：主页渲染完就装在引用计数的 `bytes::Bytes` 里了。每次请求进来，它都是零分配、零拷贝地直接扔回给网卡，几乎没有多余的 CPU 消耗。
* **只在必要时回源**：所有的缓存时间（HTML、API、CSS、JS、图片和字体）全都可以直接在 `config.toml` 里修改。不用重新编译，改完重启就行。
* **搜索引擎验证**：项目已经把 Bing 的 `BingSiteAuth.xml` 路由验证写好了，Google 则是在 DNS 侧注入 TXT 记录或通过 Meta 头进行 GSC 索引认证。至于 Baidu 验证，在你克隆了项目后，把带随机字符的验证文件直接丢进 `static/` 目录就能用，写好了安全的泛路由，不需要重新编译。

梨梨知道自己水平还很差，其实也经常会自我怀疑，觉得自己没那么厉害。但梨梨会一直在艰难的环境里，用自己的方式努力活着、思考着，不会轻易放弃的。
代码里可能有写得不妥帖的地方，毕竟梨梨一直是自学过来的，要是写错了很怕被赖上说梨梨说得不对。所以如果大家遇到技术或者配置上的疑问，梨梨真的特别建议先自己去查阅对应的资料，而不是直接来找梨梨（梨梨不是想偷懒，只是不想让人在小事上被依赖啦）。但梨梨真的在很努力地反思和改进，希望让这个小小的地方跑得更安全、更优雅，能让喜欢梨梨的人觉得稍微靠谱一点。

## License
MIT
