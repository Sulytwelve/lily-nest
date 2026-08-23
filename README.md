# 梨窝（lily-nest）
[![zread](https://img.shields.io/badge/Ask_Zread-_.svg?style=flat&color=00b0aa&labelColor=000000&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQuOTYxNTYgMS42MDAxSDIuMjQxNTZDMS44ODgxIDEuNjAwMSAxLjYwMTU2IDEuODg2NjQgMS42MDE1NiAyLjI0MDFWNC45NjAxQzEuNjAxNTYgNS4zMTM1NiAxLjg4ODEgNS42MDAxIDIuMjQxNTYgNS42MDAxSDQuOTYxNTZDNS4zMTUwMiA1LjYwMDEgNS42MDE1NiA1LjMxMzU2IDUuNjAxNTYgNC45NjAxVjIuMjQwMUM1LjYwMTU2IDEuODg2NjQgNS4zMTUwMiAxLjYwMDEgNC45NjE1NiAxLjYwMDFaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00Ljk2MTU2IDEwLjM5OTlIMi4yNDE1NkMxLjg4ODEgMTAuMzk5OSAxLjYwMTU2IDEwLjY4NjQgMS42MDE1NiAxMS4wMzk5VjEzLjc1OTlDMS42MDE1NiAxNC4xMTM0IDEuODg4MSAxNC4zOTk5IDIuMjQxNTYgMTQuMzk5OUg0Ljk2MTU2QzUuMzE1MDIgMTQuMzk5OSA1LjYwMTU2IDE0LjExMzQgNS42MDE1NiAxMy43NTk5VjExLjAzOTlDNS42MDE1NiAxMC42ODY0IDUuMzE1MDIgMTAuMzk5OSA0Ljk2MTU2IDEwLjM5OTlaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik0xMy43NTg0IDEuNjAwMUgxMS4wMzg0QzEwLjY4NSAxLjYwMDEgMTAuMzk4NCAxLjg4NjY0IDEwLjM5ODQgMi4yNDAxVjQuOTYwMUMxMC4zOTg0IDUuMzEzNTYgMTAuNjg1IDUuNjAwMSAxMS4wMzg0IDUuNjAwMUgxMy43NTg0QzE0LjExMTkgNS42MDAxIDE0LjM5ODQgNS4zMTM1NiAxNC4zOTg0IDQuOTYwMVYyLjI0MDFDMTQuMzk4NCAxLjg4NjY0IDE0LjExMTkgMS42MDAxIDEzLjc1ODQgMS42MDAxWiIgZmlsbD0iI2ZmZiIvPgo8cGF0aCBkPSJNNCAxMkwxMiA0TDQgMTJaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00IDEyTDEyIDQiIHN0cm9rZT0iI2ZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPgo8L3N2Zz4K&logoColor=ffffff)](https://zread.ai/Sulytwelve/lily-nest)
> 梨梨的个人网站：项目展示、博客与技术分享

## 项目预览
- www.sulyhub.cn

## 项目简介
梨窝是一个基于 Rust + Axum 的个人主页与技术分享网站。它不仅支持项目展示、团队成员和动态更新日志，还在 v0.3.0 引入了**「梨记」(lily-note)**——一个完全由纯 Markdown 文件驱动的轻量级个人笔记模块。
界面采用 Material You 风格，原生支持深浅色系统主题与全响应式设计。

当前版本进一步强化了纯文件管理的理念，通过无数据库的架构实现了极简运维，同时保留了极高的性能与安全防护。

## 技术指标
在极低配置的嵌入式设备（如玩客云、香橙派等 armv7l 架构单板电脑）上，该项目展现出极致的系统开销与运行效率：

### 内存占用（v0.2.8-beta 实测，v0.3.0 待测）

- v0.2.9 冷启动实测仅需 **684 KB** 内存（峰值 **1.4 MB**）；v0.2.8 在玩客云（armv7l）上持续运行 5 天、经历多次后台配置写入后，常驻内存稳定在 **3.3 MB**（峰值 **4.3 MB**）。

### 编译体积

| 版本 | 编译器优化 | 二进制 | tar.xz |
|:---|:---|:---|:---|
| v0.2.8-beta | LTO + strip | ~5.0 MB | ~2.1MB |
| v0.3.0 | LTO + strip + panic=abort + codegen-units=1 | 6.9 MB | 2.4 MB |

> v0.3.0 因引入「梨记」笔记模块（pulldown-cmark、chrono、jsonwebtoken 等），体积略有增长。后续可通过进一步裁剪依赖或 feature flag 优化。

### 性能压测报告（v0.2.8-beta，v0.3.0 待测）
环境：Ryzen 7 7840HS + DDR5 32G 5600MT + Arch Linux，release + LTO + force-http 模式

| 场景 | 工具 | 协议 | Req/s | 延迟 P50 | 延迟 P99 | 吞吐 |
|:---|:---|:---|:---|:---|:---|:---|
| X270 → 工作站（LAN） | `wrk -t4 -c64` | HTTP/1.1 | 12,180 | — | — | 109 MB/s |
| X270 → 工作站 `/health` | `wrk -t4 -c64` | HTTP/1.1 | 7,309 | 7.78ms | 24.63ms | 5.43 MB/s |
| X270 → 工作站 `/` | `wrk -t4 -c64` | HTTP/1.1 | 884 | 64.06ms | 182.26ms | 7.99 MB/s |
| 本地回路 `/health` | `wrk -t8 -c64` | HTTP/1.1 | **656,781** | 0.32ms | 1.36ms | — |
| 本地回路 `/` | `wrk -t8 -c64` | HTTP/1.1 | **523,439** | 0.41ms | 1.40ms | — |
| 本地回路 `/health` | `oha -c256` | HTTP/2 | 410,882 | 0.54ms | 1.98ms | — |
| 本地回路 `/` | `oha -c256` | HTTP/2 | 298,334 | 0.79ms | 2.22ms | — |
| Go/fasthttp `/health` (对比参考) | `wrk -t8 -c256`| HTTP/1.1 | 844,286 | 0.21ms | 2.57ms | — |
| Go/fasthttp `/` (对比参考) | `wrk -t8 -c256`| HTTP/1.1 | 562,155 | 0.32ms | 4.14ms | — |

Go版本是CodeX翻译当前Rust版本得来，只做参考，非生产产品。
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
- **「梨记」轻量级笔记模块 (NEW!)**：无需数据库，文章全部以 `.md` 纯文本文件存储于 `notes/` 目录。
- **笔记全文瞬时检索**：前端原生 JavaScript 提供零网络请求的标题、摘要与 `#标签` 秒级过滤。
- **「梨记」图片上传**：后台编辑笔记时可直接粘贴/上传 PNG/JPG/GIF/WebP 图片，按魔数严格校验、原子写入 `static/images/notes/`，无需手动管理资源路径。
- 首页动态渲染（项目、团队成员、关于我、更新日志）
- **HTML 片段模块化渲染**：采用 `templates/fragments/` 独立骨架片段，运行时动态拼装，免除 Rust 源码重编译
- **强大的后台管理面板 (/admin)**：不仅支持在线编辑各项 TOML 配置，还内置了**在线 Markdown 编辑器**用于新建、修改、删除笔记文件，保存操作 100% 异步落盘。
- **命令行运维子命令**（无需启动服务器）：
  - `lily-nest set-password` —— 初始化或修改管理员密码（支持 `--generate` 自动生成 16 位随机密码），写入 `secrets.toml`（加盐哈希）。
  - `lily-nest set-security-answers` —— 交互式读取 `config.toml` 的题目，逐题隐藏输入答案，加盐哈希后写入 `secrets.toml`，保留已存在的密码哈希。
- 配置文件驱动（config.toml、site.toml、projects.toml、about.toml、changelog.toml）
- `config.toml` 的 `[security]` 块支持动态 CORS、CSP、Permissions Policy、可信代理 IP 列表、密保题目与可选的安全问题 / Cloudflare Trace 二次校验开关
- **认证秘密已迁移到 `secrets.toml`**：管理员密码与密保答案统一以 `sha256$<salt_hex>$<hash_hex>` 格式持久化（Unix 下自动 `chmod 0600`）。`config.toml` 里相关字段已被注释、仅作为旧配置回退迁移路径。
- **首次 Web 初始化**：未设置管理员密码时，服务端启动日志会一次性输出 10 分钟有效的 Setup Code，`/api/v1/admin/setup` 可凭该码在浏览器里完成首次密码初始化。
- RESTful API（/api/v1/health、/api/v1/home/profile、/api/v1/notes*、/api/v1/admin/*、/api/v1/admin/login 登录接口、/api/v1/admin/setup*、/api/v1/admin/password 在线改密、/api/v1/admin/logout 服务端吊销）
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
  - App router 提供 `/`、`/index.html` 重定向、`/admin`、`/admin/notes*`、`/api/v1/*` 和安全头中间件
  - 静态资源 router 提供 `/robots.txt`、`/favicon.ico`、`/css/*`、`/js/*` 等

## 项目结构
```
lily-nest/
├── Cargo.toml
├── config.toml               # 站点基础配置（证书、安全策略、密保题目）
├── secrets.toml              # （不入库，首次运行后生成）管理员密码 / 密保答案的加盐哈希
├── site.toml                 # 站点基础信息配置（SEO元数据 [site] 及 个人主页 [profile]）
├── projects.toml             # 项目列表配置
├── about.toml                # 关于我列表配置
├── changelog.toml            # 更新日志配置
├── notes/                    # 「梨记」Markdown 笔记存储目录 (NEW!)
├── .jwt_secret               # （不入库，自动生成，0600）JWT 签名密钥
├── .agent.pub                # （不入库，可选）Agent Ed25519 公钥，用于发文专线
├── certs/                    # SSL 证书目录
├── src/
│   ├── main.rs               # 应用入口（含 set-password / set-security-answers CLI）
│   ├── app.rs                # 路由编织与全局共享状态
│   ├── state.rs              # 全局共享状态 AppState
│   ├── routes/
│   │   ├── mod.rs            
│   │   ├── home.rs           # 首页路由（含 304 支持）
│   │   ├── note.rs           # 「梨记」前台路由 (列表与详情)
│   │   ├── note_admin.rs     # 「梨记」后台与 Agent REST API 路由 (CRUD + 图片上传)
│   │   ├── api.rs            # 公开 RESTful API 与认证、配置保存
│   │   ├── admin.rs          # 后台管理页面渲染路由
│   │   └── static_assets.rs  # 静态资源路由
│   ├── secrets.rs            # 认证秘密加盐哈希与 secrets.toml 持久化
│   ├── utils.rs              # 跨模块工具函数（HTTP日期、URL清洗、HTML转义）
│   ├── render.rs             # HTML 模板拼接渲染
│   ├── note_loader.rs        # 异步纯文件 Markdown 加载与 TOML 提取引擎
│   ├── middlewares.rs        # 全局中间件（CORS、安全头、JWT 与 Agent 鉴权）
│   ├── config.rs             # 配置文件解析
│   ├── model.rs              # 数据模型定义
│   ├── compressor.rs         # 静态资源预压缩工具
├── static/
│   ├── css/
│   │   ├── admin.css
│   │   ├── user-theme.css
│   │   └── note.css          # 梨记样式表
│   ├── js/
│   │   ├── admin.js
│   │   ├── MaterialWeb.js
│   │   ├── note.js           # 梨记列表与前端检索逻辑
│   │   └── note_detail.js    # 梨记详情页逻辑
│   ├── images/
│   │   └── notes/            # 「梨记」上传的图片（gitignore，不入库）
├── templates/
│   ├── index.html            # 主页模板
│   ├── admin.html            # 后台管理页面模板
│   ├── note.html             # 梨记列表页模板
│   ├── note_detail.html      # 梨记详情页模板
│   └── fragments/            # HTML 渲染片段模板目录
│       └── ...
└── ...
```

## 管理后台 (/admin)
项目内置了一个基于 Material Design 3 的管理后台，允许管理员直接在浏览器中修改站点内容。

### 1. 凭据存储：哈希 + 加盐，不再明文落盘

- 管理员密码与密保答案已从 `config.toml` 迁出，统一写入 `secrets.toml`（不入库，gitignored），格式：
  ```toml
  admin_password_hash = "sha256$<salt_hex>$<hash_hex>"
  admin_security_answer_hashes = ["sha256$<salt_hex>$<hash_hex>", ...]
  ```
  运行时仅与内存中的哈希比对，永远不回显明文。
- 容器化 / CI 场景可通过环境变量注入，优先级高于 `secrets.toml`：
  - `LILY_ADMIN_PASSWORD_HASH` —— 已哈希的管理员密码
  - `LILY_ADMIN_SECURITY_ANSWER_HASHES` —— 已哈希的密保答案列表（逗号分隔）
- `config.toml` 里旧字段 `admin_password` / `admin_security_answers` 已被注释，仅作为一次性迁移回退；启动时若检测到明文会自动就地哈希、并在日志发出 `warn!` 提醒尽快迁移到 `secrets.toml`。
- Unix 系统下 `secrets.toml` 在写入时会自动 `chmod 0600`。

### 2. 首次初始化（Web 端）

1. 首次启动时若未配置密码，服务端日志会以 `warn!` 级别打印一次 **Setup Code**（8 字节随机十六进制，10 分钟内有效）：
   ```
   [WARN] 管理员未初始化。请运行 lily-nest set-password 或访问 /admin 输入 Setup Code: aabbccdd（10 分钟内有效）
   ```
2. 浏览器访问 `/admin`，前端会调用 `GET /api/v1/admin/setup/status` 判断是否进入初始化流程。
3. 用户填入 Setup Code + 新密码，提交到 `POST /api/v1/admin/setup`，服务端校验通过后写入 `secrets.toml` 并清空 Setup Code。
4. 若直接运行 `lily-nest set-password` 初始化，则跳过 Web 端的 Setup Code 流程（CLI 路径见下）。

### 3. CLI 子命令（不启动服务器）

| 命令 | 作用 |
|---|---|
| `lily-nest set-password` | 交互式设置 / 修改管理员密码（两次输入确认）。支持 `--generate` 直接生成 16 位随机密码并打印到 stdout。 |
| `lily-nest set-security-answers` | 从 `config.toml` 的 `admin_security_questions` 读取题目，交互式逐题隐藏输入答案（rpassword），加盐哈希后写入 `secrets.toml`，保留已存在的密码哈希。题目必须是真实题目，配置占位符 `default1/default2/default3` 时会被拒绝。 |

两个 CLI 写入都是 **原子落盘**（临时文件 + flush + fsync + rename）。

### 4. 在线改密 / 登出

登录后可调用以下后台端点（需 Bearer JWT）：

| 方法 | 路径 | 说明 |
|---|---|---|
| `POST` | `/api/v1/admin/password` | 携带 `old_password` / `new_password`，成功后服务端原子写 `secrets.toml` 并吊销当前 token |
| `POST` | `/api/v1/admin/logout` | 服务端将当前 JWT 的 `jti` 写入吊销表，10 分钟自动清理 |

### 5. 安全防护

- **无状态 JWT 鉴权**：登录成功后颁发 HS256 JWT，签名密钥优先使用 `LILY_JWT_SECRET`，缺省时持久化到 `.jwt_secret`（gitignored、Unix 下 `chmod 0600`），保证重启后会话不丢失。
- **服务端吊销表**：JWT 内置 `jti`，登出 / 改密后写入内存吊销表，所有后续校验都会拦截已被吊销的 token。
- **登录限流**：按客户端 IP（`X-Forwarded-For` / `CF-Connecting-IP` 经可信代理白名单校正后）分片限流，60 秒窗口内最多 5 次。
- **多层二次校验**：可选启用 `auth_ext_secq`（随机抽一道密保题）与 `auth_ext_cftrace`（校验 Cloudflare Trace 地区）。
- **密码强制锁定**：未设置、为空或保留 `"CHANGE_YOUR_PASSWORD"` 时，登录接口会直接拒绝并记录 `error!` 级安全警告，防止站点被爆破。
- **文件限制**：仅允许编辑内容相关的 `.toml` 文件，自动排除 `config.toml` 和 `Cargo.toml` 以保系统安全。

> **安全提示**：在 Debug 模式（HTTP）下，密码以明文传输，仅建议在本地开发环境使用。在生产环境（Release 模式）下，必须配置 HTTPS 以确保传输加密。

## 「梨记」自动化发文与 Agent 接口 (`/api/v1/notes`)

项目预留了一套完全脱离浏览器端「密码 + 密保 + cf-trace」的 **非对称公私钥鉴权专线**，供自动化程序、AI 助手或 CI/CD 脚本直接发布与管理博客文章。

### 1. 密钥生成与部署

生成 Ed25519 密钥对（PEM 格式），服务端仅接受 **EdDSA（Ed25519）** 算法：

```bash
openssl genpkey -algorithm ed25519 -out agent.key
openssl pkey -in agent.key -pubout -out agent.pub
```

将 `agent.pub` 改名为 `.agent.pub` 放到项目根目录（或通过环境变量 `LILY_AGENT_PUB_KEY` 配置）。`agent.key` 由 Agent 脚本持有，用于签发 JWT。

### 2. 认证方式

请求头携带 EdDSA 签名的 JWT：

```
Authorization: Bearer <JWT>
```

JWT 载荷需包含以下声明：

| 字段 | 值 | 说明 |
|------|----|------|
| `sub` | 任意标识 | 主题（如 `"agent"`） |
| `name` | 显示名称 | 昵称 |
| `role` | `"agent"` | 必须为 `agent` 或 `admin` |
| `exp` | Unix 时间戳 | 建议 5 分钟有效期 |

> `/api/v1/notes*` 鉴权中间件 `note_auth_middleware` 优先按本地 HS256 密钥校验，若失败再尝试 Ed25519 公钥校验。Web 端用 admin 角色签发的 HS256 JWT 与 Agent 用 EdDSA 签发的 JWT 均可访问。

### 3. 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/v1/notes` | 获取全部文章列表（同样暴露为 `/admin/notes` 供后台使用） |
| `GET` | `/api/v1/notes/{slug}` | 获取指定文章的详情与编辑结构（同样暴露为 `/admin/notes/{slug}`） |
| `POST` | `/api/v1/notes` | 创建新文章（同样暴露为 `/admin/notes`） |
| `PUT` | `/api/v1/notes/{slug}` | 更新指定文章（同样暴露为 `/admin/notes/{slug}`） |
| `DELETE` | `/api/v1/notes/{slug}` | 删除指定文章（同样暴露为 `/admin/notes/{slug}`） |
| `POST` | `/api/v1/notes/images` | 上传图片到 `static/images/notes/`，返回 Markdown 可用 URL（同样暴露为 `/admin/notes/images`，限 5MB） |

`POST` / `PUT` 请求体格式：

```json
{"title": "文章标题", "tags": ["标签1"], "excerpt": "摘要", "content": "Markdown正文"}
```

> 所有增删改操作会自动触发服务端内存缓存与搜索索引重载，无需重启服务；图片上传会按文件魔数（PNG/JPEG/GIF/WebP）严格校验，避免「改 Content-Type 就传任意文件」的绕过。

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

## 管理员初始化（首次必读）

第一次部署后，必须先初始化管理员密码再启动 Web 服务，否则 `/admin` 与 `/api/v1/admin/*` 都会被拒。

### 方式 A：CLI（推荐，适合服务器无人值守）

```bash
# 1. 设置密码（两次输入确认，或 --generate 随机生成）
lily-nest set-password
# 或
lily-nest set-password --generate    # 自动生成 16 位随机密码并打印到 stdout

# 2. （可选）启用安全问题时，设置密保答案
#    前提：config.toml 的 [security] admin_security_questions 已是真实题目
lily-nest set-security-answers

# 3. 启动服务
cargo run --release    # 或 cargo run（开发模式）
```

两个 CLI 都以加盐哈希（`sha256$<salt_hex>$<hash_hex>`）写入 `secrets.toml`，Unix 下自动 `chmod 0600`。

### 方式 B：Web 端 Setup Code（适合本地/容器临时启动）

1. 临时启动服务（任何一种启动方式都行）：
   ```bash
   cargo run
   ```
2. 服务端会在日志里以 `warn!` 级别打印一行：
   ```
   [WARN] 管理员未初始化。请运行 lily-nest set-password 或访问 /admin 输入 Setup Code: aabbccdd（10 分钟内有效）
   ```
3. 浏览器打开 `/admin`，填入 Setup Code + 新密码提交，服务端校验通过后写入 `secrets.toml` 并清空 Setup Code。

### 容器化 / CI 部署

无需落盘，可直接通过环境变量注入已哈希的凭据（优先级最高，会覆盖 `secrets.toml` 与 `config.toml` 明文）：

| 变量 | 说明 |
|---|---|
| `LILY_ADMIN_PASSWORD_HASH` | `sha256$<salt_hex>$<hash_hex>` 格式的管理员密码哈希 |
| `LILY_ADMIN_SECURITY_ANSWER_HASHES` | 逗号分隔的多个 `sha256$<salt_hex>$<hash_hex>`，顺序对应 `admin_security_questions` |
| `LILY_JWT_SECRET` | ≥ 32 字节的 JWT 签名密钥；缺省时持久化到 `.jwt_secret` |
| `LILY_AGENT_PUB_KEY` | Agent Ed25519 公钥（PEM）；缺省时尝试 `.agent.pub` |

## 配置说明
- `config.toml`：TLS 证书路径、`[security]` 安全策略（CORS、CSP、Permissions Policy、可信代理 IP、密保题目、安全问题 / Cloudflare Trace 二次校验开关等）、`[assets]` 预压缩配置与各类别（HTML、API、JS/CSS、图片、字体等）缓存时长解耦配置。**管理员密码与密保答案已迁出**，此处对应字段已注释。
- `secrets.toml`（不入库，首次运行后由 CLI / Setup Code 自动生成）：管理员密码与密保答案的加盐哈希，格式详见上文「管理后台」一节。可通过环境变量 `LILY_ADMIN_PASSWORD_HASH` / `LILY_ADMIN_SECURITY_ANSWER_HASHES` 覆盖注入。
- `site.toml`：站点配置与展示内容，已拆分为两部分：
  - `[site]`：页面元数据（标题 `index_title` 与 `meta_desc` 元描述信息）
  - `[profile]`：个人主页卡片内容（包括头像、背景、团队成员、自我介绍等）
- `projects.toml`：项目列表
- `about.toml`：关于我
- `changelog.toml`：更新日志配置，用于首页时间轴展示
- `static/`：静态资源（图片、CSS、JS、robots.txt 等）；`static/images/notes/` 为「梨记」上传图片目录（gitignored）
- `templates/`：前端页面及片段骨架 HTML 模板
- `.jwt_secret`（不入库，自动生成，Unix 下 `chmod 0600`）：HS256 JWT 签名密钥；可用 `LILY_JWT_SECRET` 覆盖
- `.agent.pub`（不入库，可选）：Agent Ed25519 公钥；可用 `LILY_AGENT_PUB_KEY` 覆盖

## 安全特性
- **HTTP/2 协议层防御**：全局限制 `max_header_list_size` 为 32KB，从底层阻断针对 HTTP/2 HPACK 的压缩炸弹攻击（如 CVE-2026-49975），拒绝异常内存消耗，实际`hyper`和`h2`底层是`“免疫”`这种攻击的，当前项目增加也是纵深防御。
- **认证秘密加盐哈希存储**：管理员密码与密保答案一律以 `sha256$<salt_hex>$<hash_hex>` 写入 `secrets.toml`，Unix 下 `chmod 0600`；容器化部署可走环境变量注入，源代码 / 配置文件里不再出现明文。
- **HS256 + EdDSA 双轨 JWT**：
  - Web 管理后台：HS256 JWT，密钥来自 `LILY_JWT_SECRET` 或持久化的 `.jwt_secret`（0600）。
  - Agent 发文专线：Ed25519（EdDSA）JWT，公钥来自 `LILY_AGENT_PUB_KEY` 或 `.agent.pub`，完全免密码 / cf-trace。
  - 笔记 API 中间件会按 HS256 → EdDSA 顺序自动尝试两者。
- **服务端 JWT 吊销表**：每个 JWT 携带 `jti`，登出 / 改密后服务端将 `jti` 写入内存吊销表，所有后续校验都会拦截已被吊销的 token；过期项按 `exp` 自动清理。
- **登录限流**：按客户端 IP 分片限流（B26），60 秒窗口内最多 5 次；可信代理白名单 `trusted_proxy_ips` 校正 `X-Forwarded-For` / `CF-Connecting-IP`。
- **Setup Code 首次初始化**：未初始化时启动日志打印 10 分钟有效的一次性 Setup Code，避免空密码上线即被爆破。
- **多层二次校验**：可选启用 `auth_ext_secq`（随机抽一道密保题，要求 `answer_hashes.len() == questions.len()` 且非占位符）与 `auth_ext_cftrace`（校验 Cloudflare Trace 地区）。
- **默认密码安全锁定**：强制过滤并禁用未设置、为空或为默认占位符 `"CHANGE_YOUR_PASSWORD"` 的管理员密码，并在发生登录尝试时在服务器控制台抛出 `error!` 级别警报日志，防止站点被爆破。
- **原子写入**：配置文件、`secrets.toml`、笔记文件均采用「临时文件 + flush + fsync + rename」落盘，崩溃后不会留下半截文件。
- URL 协议校验：仅允许 `/` 和 `http://` 以及 `https://` 开头的链接，防止 `javascript:` XSS 注入
- HTML 转义：所有配置内容插入页面前均转义
- HTTP 安全响应头：CSP、HSTS、X-Content-Type-Options、X-Frame-Options、Referrer-Policy、Permissions-Policy
- 安全配置解析错误会记录为错误日志；release 模式使用缓存的 security config，debug 模式支持热加载
- release 模式强制 TLS，不支持 HTTP 回退
- 「梨记」图片上传按文件魔数（PNG/JPEG/GIF/WebP）严格校验，禁绝「改 Content-Type 就传任意文件」的绕过

## 亮点与注意事项
- debug 模式每次请求重新渲染页面，方便开发调试
- release 模式使用内存缓存，首页渲染结果复用（5 分钟 Cache-Control）
- 静态资源预压缩（默认关闭）：在 `config.toml` 的 `[assets]` 中设置 `precompress = true` 以开启，支持 Gzip、Brotli 和 Zstd 压缩，自动检测文件更新并重新压缩
- 深色主题完全由 CSS `@media (prefers-color-scheme: dark)` 驱动，无 JS 依赖，无闪烁
- 前端资源基于 Material Design 3，使用 `@material/web` 组件库本地构建
- 项目部署于 Cloudflare，开放 8443（HTTPS）和 8880（HTTP dev）端口
  > **关于 Cloudflare 边缘缓存的重要提示：**  
  > Cloudflare 默认对部分非标准端口（包括 `8443`、`8880` 等，注意 `8080` 虽为非标端口但支持缓存）**硬性禁用了边缘缓存**（Free/Pro 计划会一直返回 `cf-cache-status: DYNAMIC`）。  
  >   
  > **🎉 缓存难题解决：** 在版本 [`3ad622c`](https://github.com/Sulytwelve/lily-nest/commit/3ad622c710db8421868019bd183f9c6d376510f8) 之后，梨梨（Sulytwelve）已将 `个人服务器` 原有的 `8443` 非标端口回源方案改为了 **Cloudflare Tunnel (`cloudflared`)** 方案，将 `lily-nest` 的端口设置为 `https_port = 443` 并允许防火墙 `443` 端口出站。在完成此次变更后，网站已成功实现了所有静态资源（如 JS/CSS 等）在 Cloudflare 边缘节点的 `HIT`（缓存命中）！
- 仅供个人学习/展示用途，欢迎二次开发

## 设计哲学与开发历程
项目最初是梨梨在 2026 年初在试验场（`suly-nexus`）启动的。在经历了一番技术选型的纠结、因为个人事务短暂沉寂，以及后来的高效重构后，终于诞生了如今的 `lily-nest` 虽然还是很简陋。

当时为了快速验证想法，梨梨只花了 3 天时间写完核心 MVP，总计 5 天就把项目成功上线到 Cloudflare。以下是具体的开发历程：

- **2026-02-16**：装好 Neovim (AstroNvim) 开发环境，开始折腾 Dioxus、Leptos 及 Tauri 等流行前端模板框架。最初梨梨特别仰慕这些“新技术”的，但在实际使用中发现它们普遍需要在宏里书写极其繁琐且难以维护的代码。梨梨是自学过来的，不想把自己绕进那些繁琐的宏里，综合考虑后，决定避开复杂的框架，选择最纯粹的手写 HTML/JS/CSS，配合后端极简的占位符拼接。
- **2026-02-23**：对项目砍了很多功能，正式创建 `lily-nest` 并初始化骨架，开始核心 MVP 开发。
- **2026-02-26**：搞定了后端核心数据模型、路由逻辑拼装以及前端页面设计，完成首个可用版本打包备份（当时还是裸 HTTP 方案）。
- **2026-02-28**：将项目顺利打包部署并上线至 Cloudflare 生产环境。
- **2026-03-19**：当前 GitHub 仓库的第一个公开 Commit。其实在 2 月 28 日项目上线后到 3 月 19 日这段时间，因为处理梨梨的一些个人事务，项目沉寂了一段时间。发布前，梨梨对代码进行了安全脱敏（清理敏感路径与测试私钥信息），确保代码能以最干净、规范的工程形态开源。
- **2026-03-22**：回归后立即启动高频迭代，首先实现了 **HTTPS / TLS 证书动态加载** 支持。
- **2026-03-28**：全面采用 **Material Web Components (MWC)** 重构前端页面模板与后端渲染架构，现代的 Material You 风格基调。
- **2026-03-29 至 04-01**：重构并加入了一系列核心安全特性，包括 **URL 协议清洗（防范 XSS 注入）、CORS 跨域预检、严格的 CSP 内容安全策略、管理员动态安全密码管理**等。
- **2026-04-08**：静态资源全面支持 **Gz, Br, Zstd预压缩**，可压榨静态资产传输开销。
- **2026-05-20 至 05-22**：迎来**安全策略与管理面板重磅升级**，支持多重安全提问校验、Cloudflare Trace 可信源二次校验，以及动态编辑 site/about/projects 的后台 TOML 管理面板。
- **2026-05-30 至今**：引入页面内存级**零拷贝（Zero-Copy）预缓存响应机制**与 `security.txt` 披露支持，并坚守服务端渲染文本以对 AI 采集器与爬虫完美兼容。
- **2026-07-05**：接入**持久化 JWT** 与 **Agent 公私钥发文专线**（Ed25519 EdDSA），密钥缺省时自动生成 `.jwt_secret` / 读取 `.agent.pub`，并支持环境变量 `LILY_JWT_SECRET` / `LILY_AGENT_PUB_KEY` 注入。
- **2026-08-15**：根据第三方安全审计报告完成**多轮安全硬化**：HTML 转义 / 模板注入防护、Rate Limit 分片加固、密保题不再通过页面整集下发、密保答案哈希、CSP 与 HSTS 收严、笔记存储与渲染多层防御。
- **2026-08-23**：管理员凭据正式迁出 `config.toml`，统一写入 `secrets.toml`（加盐哈希、Unix 下 `chmod 0600`）；同时上线 `lily-nest set-password` 与 `lily-nest set-security-answers` 两个 CLI，并提供 `/api/v1/admin/setup` 一次性 Setup Code 的 Web 端首次初始化路径。

## 梨梨的悄悄话：这个项目是怎么被雕琢出来的？
这个项目，是梨梨用 Rust 一点点抠出来的。可能有人觉得它只是个普通的个人主页，但在你看不到的底层，梨梨花了很多心思。

梨梨不想把它写得太夸张，但它在服务器上的表现确实让梨梨挺开心的：
* **不爱占地方（v0.2.8-beta 实测）**：v0.2.9 刚启动时，它在玩客云上只吃 **684 KB** 内存（峰值 **1.4 MB**）；v0.2.8 在同一台 ARM 设备上连续跑了 5 天、后台写了好几次配置后，常驻内存稳在 **3.3 MB**（峰值 **4.3 MB**）。比那些动辄几十上百兆的大家伙要老实多了。v0.3.0 待有空时重新测试。
* **零拷贝与零堆分配**：主页渲染完就装在引用计数的 `bytes::Bytes` 里了。每次请求进来，它都是零分配、零拷贝地直接扔回给网卡，几乎没有多余的 CPU 消耗。
* **只在必要时回源**：所有的缓存时间（HTML、API、CSS、JS、图片和字体）全都可以直接在 `config.toml` 里修改。不用重新编译，改完重启就行。
* **搜索引擎验证**：项目已经把 Bing 的 `BingSiteAuth.xml` 路由验证写好了，Google 则是在 DNS 侧注入 TXT 记录或通过 Meta 头进行 GSC 索引认证。至于 Baidu 验证，在你克隆了项目后，把带随机字符的验证文件直接丢进 `static/` 目录就能用，写好了安全的泛路由，不需要重新编译。

梨梨知道自己水平还很差，也经常会自我怀疑，觉得自己没那么厉害。但梨梨一直在艰难的环境里，用自己的方式努力活着、思考着，不会轻易放弃的。
代码里可能有写得不妥的地方，不要怪梨梨，梨梨是自学过来的，要是写错了很怕被指责做得不对。所以如果大家遇到技术或者配置上的疑问，梨梨真的建议自己去查阅对应的资料，而不是直接来找梨梨（梨梨不是想偷懒，只是不想让人在小事上被依赖）但是真要找我也是可以的。总之梨梨真的在很努力地反思和改进，希望让这个小小的地方跑得更安全、更优雅，能让喜欢梨梨的人觉得稍微靠谱一点。

## License
MIT
