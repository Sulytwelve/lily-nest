# 🗺️ 梨窝 (lily-nest) 线路图与开发历史 (ROADMAP)

这是一个详细的开发历史回顾与未来演进线路图。已完成的里程碑记录了 `lily-nest` 自开发以来的关键迭代与我们今天共同完成的安全大捷；未来的规划则是系统下一步走向极致通用与高可扩展性的方向。

---

## 🚀 未来演进路线 (Roadmap)

> [!TIP]
> 以下为规划中的架构升级路线，目前仅作为技术路线图记录，不作为立即执行的任务。

- [x] **⚙️ 解耦服务器配置：端口号分离（已完成）**
  - [x] 在 `config.toml` 中新增 `[server]` 配置块，支持 `http_port` 和 `https_port` 字段。
  - [x] 重构 `src/main.rs`，将硬编码的 `8880` (Debug) 和 `8443` (Release) 端口改为从配置中动态加载，使服务支持零编译一键更换运行端口。
  
- [ ] **🎨 解耦呈现逻辑：分离 HTML 渲染片段**
  - [ ] 创建 `templates/project_item.html` 和 `templates/about_item.html` 独立骨架文件。
  - [ ] 重构 `src/render.rs`，将硬编码在 `format!` 宏中的项目卡片与关于我 HTML 片段剥离。
  - [ ] 使程序在运行时动态读取片段模板并执行参数替换，使前端样式修改彻底免除 Rust 源码重编译。

---

## 🏆 已完成的里程碑 (Milestones & Changelog)

### 🔴 v0.2.7 零拷贝缓存与安全披露版 (当前版本)
- [x] **内存级零拷贝预缓存与缓存生命周期解耦 (Zero-Copy HTML Cache & Expiry Decoupling)**
  - [x] 基于引用计数 `bytes::Bytes` 重构首页内存 HTML 缓存渲染系统，做到请求进入时真正的零内存堆分配（Zero-Allocation）与零拷贝（Zero-Copy）直接返还网卡。
  - [x] 将所有页面与资源（HTML、API、JS/CSS、图片、字体）的缓存时长（Cache-Control）从代码中彻底解耦，在 `config.toml` 新增 `[assets]` 块进行集中管理，支持免重启即时调整。
- [x] **安全网关与验证生态增强 (Security & Multi-Engine Verification)**
  - [x] 新增对 RFC 9116 安全披露文件（`security.txt`）的规范化响应与自适应缓存路由支持，为站点引入透明的漏洞报告标准。
  - [x] 设计并上线了针对 `baidu` 搜索引擎的**动态所有权泛解析路由**，基于严格的路径字符白名单阻断目录遍历风险，用户克隆项目后将验证 HTML 直接丢进 `static/` 即可实现免重启、免编译一键上线。
  - [x] 内置了 Bing 搜索的 `BingSiteAuth.xml` 路由验证，并补充了 Google GSC 在 DNS 侧/Meta 标签侧的安全验证支持指引。
- [x] **非标端口缓存与反向代理强制 HTTP 优化 (Cloudflare Cache & Force HTTP)**
  - [x] 成功部署并验证了 **Cloudflare Tunnel (`cloudflared`)** 服务，将非标端口回源升级为 443 标准出站，扫平了 Cloudflare 限制非标端口边缘缓存的阻碍，实现静态资产全球边缘节点 100% 缓存命中（`HIT`）。
  - [x] 新增 `force-http` 特性选项，允许在 Release 模式下彻底跳过 TLS 证书初始化检测，完美兼容 Caddy、Nginx 或 cloudflared 等反向代理前置进行 TLS 终止的架构。

### 🔴 v0.2.4 配置解耦与端口分离版
- [x] **服务器配置解耦 (Server Configuration Decoupling)**
  - [x] 将 HTTP 和 HTTPS 的运行端口彻底从代码硬编码中解耦，移入 `config.toml` 最顶部的 `[server]` 段。
  - [x] 新增 `ServerConfig` 模型和动态解析加载逻辑，支持免编译直接配置服务运行端口。
  - [x] 升级 `Cargo.toml` 版本至 `0.2.4`，升级 `site.toml` 页面版本至 `0.2.4-beta`。

### 🔴 v0.2.3 安全与架构加固版
- [x] **安全性加固 (Security Hardening)**
  - [x] 在 `src/routes/admin.rs` 中将配置 JSON 序列化后的 `</` 替换为 `<\/`，彻底预防潜在的闭合脚本标签 XSS 注入。
  - [x] 将全局 `cors` 中间件层精确下沉至 `/api/v1/*` 接口路由，防止首页 `/` 与 `/admin` 页面被注入 `Vary: Origin` 头部，为 Cloudflare 边缘缓存扫清协议障碍。
  - [x] 在管理员验证中间件 (`src/middlewares.rs`) 中实现 Trace host 与 IP 的强一致性阻断校验，并为 5 个关键校验失败点增加 `warn!` 日志警报。
- [x] **健壮性与日志规范化 (Robustness & Logging)**
  - [x] 规范 `routes/home.rs` 和 `routes/api.rs` 中 `render_index` 在 `spawn_blocking` 中可能发生 panic 时的错误捕获，添加详细的 `error!` 级别的日志追踪。
- [x] **路由与渲染调优 (Routing & Rendering)**
  - [x] 实现了根据配置文件条件化加载预压缩静态资源的逻辑。
  - [x] 通过在 `<script type="application/json">` 标签内放置配置，完美绕过 CSP 行内脚本块的安全限制。
  - [x] 修复后台管理面板中编辑器容器对齐的样式问题，同步前后端管理渲染版本至 `0.2.3`。

### 🟡 v0.2.1 ~ v0.2.2 缓存与压缩优化版
- [x] 引入静态资源预压缩开关，解决预压缩工具潜在的 panic 风险。
- [x] 优化本地开发环境下的 Cache-Control 缓存判定逻辑，提升开发体验。
- [x] 重构后台保存配置 `save_config` 接口，支持异步安全写入磁盘并瞬时刷新内存 HTML 缓存，实现免重启即时生效。
- [x] 实现 Vary 请求头仅在请求静态资源时有条件添加，避免污染页面响应。

### 🟢 v0.1.8 ~ v0.2.0 项目结构重构版
- [x] 彻底重构项目骨架，进行模块化分离，将 `admin.rs` 成功移入 `routes` 文件夹。
- [x] 将原有大杂烩的 `site.toml` 精细拆分为 `site.toml` (SEO 元数据 `[site]` 与个人名片 `[profile]`)、`projects.toml` 和 `about.toml`。
- [x] 修复跨域预检请求 (CORS preflight) 与云端遥测 CSP 通配符安全冲突问题。

### 🔵 v0.1.4 ~ v0.1.7 动态配置与安全网关版
- [x] 引入混合式 Cloudflare Zero Trust 头部校验与二级二次 Trace (WARP/Gateway) 验证机制。
- [x] 推出完全由 `config.toml` 配置驱动的动态管理后台 UI 与管理员随机密保问题鉴权系统。
- [x] 完美支持管理员密码安全强制锁定——对未配置、为空或使用默认 `"CHANGE_YOUR_PASSWORD"` 的凭证直接断开登录，并在控制台抛出高危 `error!` 日志。
- [x] 引入 `assets` 静态资源预压缩功能，原生支持 `gzip`、`brotli` 压缩。

### 🟣 v0.1.3 早期奠基版
- [x] 规范系统基础路由与静态资源 ServeDir 的定义。
- [x] 搭建首个支持条件请求（`If-Modified-Since` -> `304`）的首页内存缓存渲染系统。
- [x] 规划站点整体安全响应头（CSP、HSTS、X-Content-Type-Options 等）。

---

> _"以极致的资源克制，在螺蛳壳里做道场；在几十个 commit 的锤炼下，终让老旧硬件绽放现代安全的曙光。"— lily-nest 开发者志_
