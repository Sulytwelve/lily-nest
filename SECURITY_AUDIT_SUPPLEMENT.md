# 梨窝 (lily-nest) 安全审查补充报告（第三轮：审计之外的新发现）

> 审查日期：2026-08 ｜ 前置文档：`SECURITY_AUDIT.md`（B1–B34、I1–I7）
> 本文只收录**原审计未覆盖**的漏洞（N 系列）与更优代码改进（M 系列），并附对原审计的勘误与补强。
> 结论分级：🔴 高危 ｜ 🟠 中危 ｜ 🟡 低危/建议

---

## 1. 结论摘要

原审计已覆盖 34 个 B 类问题与 7 个 I 类改进。本轮逐行复核 `src/**/*.rs`、全部模板、`static/js/*.js`、`config.toml`、`Cargo.toml`、`.gitignore` 与 git 跟踪状态后，新增发现：

| 项 | 结论 |
|---|---|
| 二阶模板注入（N1） | 🔴 **新增独立存储型注入通道**。顺序 `str::replace` 会重扫已插入的用户数据，使 `title/excerpt/tags = "{{content}}"` 之类的载荷把未消毒的 `html_output` / `notes_json` 注入 `<h1>`、`<meta content>`、笔记卡片等错误上下文，绕过 `html_escape`。即使按 B1/B23 最小方案修复，该通道仍独立存在 |
| CF Trace 绑定校验可被缺字段绕过（N2） | 🟠 `ip` / `h` 缺失时整段校验被 `if let Some` 静默跳过，攻击者手搓 `loc=CN\nwarp=on\ngateway=on` 即可通过该“第二因子” |
| 占位密保答案未硬拒绝（N3） | 🟠 与占位密码 `CHANGE_YOUR_PASSWORD` 不同，`default1/default2/default3` 没有等价防护，启用 `auth_ext_secq` 后第二因子可被直接猜中 |
| 可编辑配置文件黑名单（N4） | 🟠 目录扫描 + 大小写敏感黑名单。Windows/macOS 上 `Config.toml` 变体可把实时 `config.toml` 变成后台可编辑对象；任何新增 `.toml` 会自动进入可编辑列表 |
| 仓库跟踪真实 TLS 私钥（N5） | 🟡 `certs/example.com.key` 是完整 RSA 私钥且被 git 跟踪 |
| 管理页/管理 API 缺 no-store（N6） | 🟡 `/admin` HTML（内含 B22 的题目集）与 `/api/v1/admin/configs*` 无缓存控制，可被缓存/索引 |
| CSP 与 Cloudflare Analytics 矛盾（N7） | 🟡 `script-src 'self'` 拦截 `static.cloudflareinsights.com` beacon，`connect-src` 也未放行 `cloudflareinsights.com`，功能静默失效 |
| CSP 配置非法时静默缺失（N8） | 🟡 `HeaderValue::try_from` 失败时直接不发 CSP 头，fail-open 且无日志 |
| 登录端点 CORS 任意源可读 JWT（N9） | 🟡 默认 `allow_origins = ["*"]` 时任意网页可脚本化登录并读取响应中的 JWT |
| 登录 body 无读取超时（N10） | 🟡 慢速连接可占满限流窗口，放大 B3/B26 的锁定攻击 |

---

## 2. 新增漏洞清单（N 系列）

### N1. 🔴 顺序 `String::replace` 造成的“二阶模板注入”

- **位置**：`src/routes/note.rs:136-143`、`src/routes/note.rs:247-255`；同类模式 `src/render.rs:93-138`
- **机理**：`String::replace` 逐占位符替换，后一轮替换会**再次扫描前几轮刚插入的用户数据**。`utils::html_escape` 不转义 `{` `}`，因此：
  - `title = "{{content}}"` → 先被转义插入，随后 `.replace("{{content}}", html_output)` 把未经消毒的 `html_output` 原样注入 `<title>` 与 `<h1>`。
  - `excerpt = "{{content}}"` 或 `tags = ["{{content}}"]` → raw HTML 被注入 `<meta name="description/keywords" content="...">`，可用 `"` 打破属性逃逸到 `<head>`。
  - 列表页 `title = "{{notes_json}}"` → `notes_html` 先插入，随后 `{{notes_json}}` 替换把未做 HTML 转义的 JSON（`serde_json` 不转义 `<`）注入卡片 DOM；结合其它笔记 tag 中的 `<img ...>` 等 payload 即形成页面破坏/潜在 XSS。
- **与 B1/B23 的关系**：B1 只消毒 `html_output`，B23 只修 `</script>`。本通道绕过两者：即使 B1/B23 按最小方案修完，已消毒 HTML（含引号）仍会被打进属性/标题上下文，JSON 仍会被打进 body 上下文。当前 CSP 能压制内联脚本，但 `<meta http-equiv=refresh>` 可造成开放跳转/钓鱼，DOM 被彻底打乱；CSP 一旦放宽即为完整 XSS。
- **修复**：单遍渲染——只扫描原始模板，绝不重扫已插入值；所有用户值在单遍拼接前各自完成转义。`src/render.rs` 主页渲染同样处理。补回归测试。

### N2. 🟠 CF Trace 的 `ip` / `h` 一致性校验是“可选”的

- **位置**：`src/middlewares.rs:242-247`、`:274-279`
- **问题**：`if let Some(ref tip) = trace_ip` 与 `if let Some(ref th) = trace_host` 在字段缺失时**跳过**绑定校验。只要 `cf_trace` 文本包含 `loc=CN`、`warp=on`、`gateway=on` 三行，不含 `ip`/`h` 也通过。此外 `:253` 优先信任客户端可伪造的 `x-forwarded-host` 进行 host 比对。原审计 §3 已指出 trace 可伪造/可重放，但未指出这两处比对是“有则查、无则免”。
- **修复**：`auth_ext_cftrace` 开启时强制要求 `ip` 与 `h` 字段，缺失即 401；host 只信任服务端权威配置，不信任 `x-forwarded-host`。根本方案是按 §5 退役该机制。

### N3. 🟠 默认占位密保答案未像占位密码一样被服务端硬拒绝

- **位置**：`config.toml:33-34`、`src/model.rs:173-182`、`src/middlewares.rs:161-164`（有密码占位检查）对比 `:174-194`（无答案占位检查）
- **问题**：`admin_security_answers = ["default1","default2","default3"]` 是仓库与 `SecurityConfig::default()` 的默认值。启用 `auth_ext_secq` 而未改答案时，攻击者提交 `question_index=0&answer=default1` 即可通过第二因子。
- **修复**：启用 `auth_ext_secq` 时拒绝任何占位答案/题目（与占位密码同等策略），启动时 `error!` 告警；答案按 B5 建议哈希存储。

### N4. 🟠 “可编辑配置白名单”实为黑名单，且大小写比较在 Windows 上可绕过

- **位置**：`src/config.rs:119-130`、`src/routes/api.rs:54-67`
- **问题**：
  ```rust
  if name.ends_with(".toml") && name != "config.toml" && name != "Cargo.toml" { ... }
  ```
  ① 目录扫描 + 黑名单：任何新增 `.toml`（如 `secrets.toml`）自动成为后台可读可写文件；② 比较大小写敏感：在 Windows/macOS 大小写不敏感文件系统上，部署文件名为 `Config.toml` 时，`load_config_section` 读 `"config.toml"` 仍命中该文件，而 `"Config.toml" != "config.toml"` → 实时安全配置（`admin_password`/CSP/限流开关）变成后台可编辑对象，突破“config.toml 不可编辑”边界。
- **修复**：静态显式白名单（`site.toml`、`projects.toml`、`about.toml`、`changelog.toml`、`sitemap.xml`），并做大小写不敏感比较；名单外一律 403。

### N5. 🟡 仓库跟踪真实 RSA 私钥

- **位置**：`certs/example.com.key`（`git ls-files` 确认被跟踪；`.gitignore:15-16` 还以 `!certs/example*` 放行）
- **问题**：完整 `BEGIN PRIVATE KEY` 的 RSA 私钥进入公开仓库。README 声明仅本地测试，但 secret scanner 必报、fork 者可能误用、历史中永久留存。
- **修复**：从仓库与 git 历史移除；开发模式改为启动时自动生成一次性自签证书。

### N6. 🟡 `/admin` 页面与管理 API 缺 `Cache-Control: no-store` / `noindex`

- **位置**：`src/routes/admin.rs:16-41`、`src/routes/api.rs:47-70`、`templates/admin.html`
- **问题**：B25 只覆盖登录响应。`GET /admin` 内嵌 `auth_config_json`（B22），却无缓存控制，可被 CDN/浏览器缓存并被搜索引擎索引；管理 API 的 GET 响应同样无缓存头。
- **修复**：所有 `/admin*`、`/api/v1/admin/*` 响应加 `Cache-Control: no-store, no-cache, must-revalidate` + `Pragma: no-cache`；`/admin` 加 `X-Robots-Tag: noindex, nofollow` 与 `<meta name="robots">`。

### N7. 🟡 CSP 与 Cloudflare Web Analytics 自相矛盾

- **位置**：`src/render.rs:6,114-124` 对比 `config.toml:39-50`
- **问题**：启用 token 时注入 `https://static.cloudflareinsights.com/beacon.min.js`，但 `script-src 'self'` 拦截该外链；`connect-src` 只放行 `cloudflare.com`/`*.cloudflare.com`，不含 `cloudflareinsights.com`。Release 环境下 Web Analytics 完全失效且无提示。
- **修复**：仅当配置了合法 token 时向 CSP 动态追加 `script-src https://static.cloudflareinsights.com` 与 `connect-src https://cloudflareinsights.com`；或本地 vendor 化 beacon。

### N8. 🟡 CSP / Permissions-Policy 配置非法时静默缺失（fail-open）

- **位置**：`src/middlewares.rs:81-89`
- **问题**：`HeaderValue::try_from` 失败或字符串为空时直接不发头，且无日志。
- **修复**：失败时 `error!` 并回退到内置默认 CSP / Permissions-Policy（fail-closed）。

### N9. 🟡 登录端点可被任意源跨域调用并读取 JWT 响应

- **位置**：`src/app.rs:72`（CORS 只包 `api_routes`）、`src/middlewares.rs:21-42`、`config.toml:31`
- **问题**：`allow_origins = ["*"]` 时 `/api/v1/admin/login` 响应带 `Access-Control-Allow-Origin: *`。任意恶意网页可发 POST 并**读取响应体中的 JWT**：钓鱼页可在受害者浏览器内完成登录并回传 token；也可借访客 IP 轮换做分布式口令猜测。
- **修复**：登录端点排除出 CORS 通配层（同源专用），校验 `Origin`/`Referer`；`allow_origins` 按 §5 收紧为真实站点。

### N10. 🟡 登录请求体读取无超时

- **位置**：`src/middlewares.rs:130`
- **问题**：`to_bytes(..., 64*1024)` 无读取超时。攻击者可用 5 条只发头不发体的慢连接占满某 IP 的限流窗口 60 秒；直连部署下配合伪造头可精准锁定目标 IP。
- **修复**：用 `tokio::time::timeout`（无需新依赖）或 `tower-http::timeout::TimeoutLayer` 给登录 body 读取加 10–15s 上限，配合 B3 的可信 IP 修复。

---

## 3. 对原审计的勘误与补强

1. **B4 的 README 引用已过时**：当前 `README.md:146-157` 只宣称 EdDSA，已无“RS256 双支持”表述。B4 代码问题仍成立；修复方向为删除 `src/middlewares.rs:403-404` 的 RSA 分支或真正实现分支，并同步修正注释。
2. **B18 表述不准确**：axum 0.8 的 `Json<T>` 提取器自带 2MB 默认 body limit，笔记接口并非“完全无上限”；但缺少业务级上限/配额（2MB × 无限数量仍可打满磁盘），应加显式单篇上限与写入频率配额。
3. **B30 行为描述有误**：`src/routes/static_assets.rs:176-189` 对任意非 `baidu_` 单段路径返回 404；只有 `baidu_*.html` 且校验码含非法字符才返回 400。本质问题仍是宽泛的动态兜底路由。
4. **B23 最小修复仍不够**：`.replace("</", "<\\/")` 之外，JSON 中的 `<!--` 会使 script 解析器进入 escaped 状态，仍可破坏 JSON 区解析。推荐全量转义：`<`→`\u003c`、`>`→`\u003e`、`&`→`\u0026`、U+2028/U+2029。
5. **B7 补漏**：主页 markdown 分支（`home.rs:39-46`）、主页 debug HTML 分支（`home.rs:48-63`）与所有 `check_304` 304 响应（`home.rs:119-135`）同样缺少 `Vary: Accept`。

---

## 4. 更优的代码改进（M 系列）

| # | 位置 | 改进 |
|---|---|---|
| M1 | `src/routes/note.rs`、`src/render.rs` | 用 N1 的单遍模板渲染替换全部 `str::replace` 链；模板文件启动时预载入 `AppState`，不要每请求 `read_to_string` |
| M2 | `src/app.rs:15-17`、`src/routes/static_assets.rs:14`、`src/main.rs:30,43` | 启动时只解析一次 `config.toml`（当前同一文件被解析 4–5 次），各 section 放 `Arc<AppConfig>`；静态根文件 handler 复用 `AppState.assets_config`（顺带修 B8）；debug 热加载改为按 mtime 判断 |
| M3 | `src/routes/api.rs:121`、`src/routes/note_admin.rs:93,138` | 写盘（配置、笔记、预压缩产物）改 temp + rename + fsync 原子写；`save_config` 对 `changelog.toml`、site.toml 的 `[note]` 段做 schema 校验，sitemap 做真正的 XML 校验 |
| M4 | `src/state.rs`、`src/note_loader.rs`、`src/routes/note.rs:257-267` | 索引改 `HashMap<slug, entry>` + `Arc<str>` 正文（B10/I1 彻底版）；锁外加载（B28）；`note_html_cache` 加代际计数防止并发旧渲染回填陈旧缓存；换 LRU；slug 做 URL 编码；frontmatter 字段做长度/字符集校验；release 下手动放入 `notes/*.md` 支持 mtime 触发重载或提供 reload 端点 |
| M5 | `src/app.rs:20-37`、`src/middlewares.rs:283-287` | `LILY_JWT_SECRET`/文件内容校验 ≥32 字节；`.jwt_secret` 原子生成；`exp` 用 `saturating_add` 并设上限；管理密码/密保答案改哈希存储（B5 延伸） |
| M6 | `src/routes/home.rs`、`src/routes/note.rs`、`src/middlewares.rs` | 补全 `Vary: Accept`（含 304）、笔记页 `Cache-Control`/`ETag`/`Last-Modified`（B21）；未来时间的 `If-Modified-Since` 按现在处理；管理路由 `no-store`（N6）；静态资源挂安全头（B24）；可加 COOP/CORP |
| M7 | `static/js/admin.js`、`static/js/note.js` | 文件名/slug 拼 URL 前 `encodeURIComponent`；`fetch` 统一超时与 `cache: 'no-store'`；内联脚本/onload 全部外移（B2/B20）；`innerHTML` 改 DOM API（B13/B34）；vendor JS/CSS 加 SRI |
| M8 | `src/main.rs`、`src/middlewares.rs` | bind 地址配置（I7 正式化）、连接/body 超时、SIGTERM 优雅关停；启用 `TraceLayer`（依赖已声明未使用）；笔记增删改补审计日志；`EnvFilter` 尊重 `RUST_LOG` |
| M9 | 仓库根目录 | 目前无测试与 `.github` CI。建议 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo audit`/`cargo-deny`（I6），并加 N1/B23 注入回归测试、模板渲染 golden 测试 |
| M10 | `src/model.rs:203,355` | 删除/实现死配置与死字段：`api_cache_seconds` 从未使用、`original_slug` 从未读取（B14）、RSA 验签分支（B4） |

---

## 5. 建议修复顺序

1. **第一批**：N1（单遍模板）+ B1（ammonia）+ B23（全量 JSON 转义）；N4（显式白名单）；B22/B3/B26（可信 IP + governor 限流 + 不下发题目）。
2. **第二批**：N2/N3、N6/N7/N8、B24 + B2/B20（CSP 全站化）、B25/B19。
3. **第三批**：M 系列工程改进 + B27/B28/B29 + OAuth 迁移（§5）。

---

## 6. 本轮已落实 / 待办

| 编号 | 状态 |
|---|---|
| N1 + M1 + 勘误 4 | 待修复 |
| N2 / N3 / N8 / N10 + B15 / B25 | 待修复 |
| N4 / N6 + M3 | 待修复 |
| N7 + B24 | 待修复 |
| B2/B20/B13 + M7 | 待修复 |
| B27/B12/B28/B31 + 字段长度校验 | 待修复 |
| M4/M5/M8/M9 剩余项 | 视后续迭代排期 |

> 审查说明：本环境无网络拉取 crates，`cargo clippy` / `cargo audit` 未能实跑（schannel 证书错误无法更新 crates.io 索引），依赖侧结论以 CI 中的 `cargo audit` 为准。
