# 梨窝 (lily-nest) 安全审查与 OAuth 迁移报告

> 审查日期：2026-07 ｜ 审查范围：`src/` 全部 Rust 源码、`templates/`、`static/js/`、`config.toml`、`Cargo.toml`
> 结论分级：🔴 高危 ｜ 🟠 中危 ｜ 🟡 低危/建议

---

## 1. 结论摘要

| 项 | 结论 |
|---|---|
| 整体评价 | 架构清晰、防御意识不错（白名单配置编辑、CSP、限流、secret 已 gitignore、EdDSA 修复已合入），但存在 **2 个存储型 XSS**（B1 笔记正文 + B23 `notes_json` 大小写绕过）、**`/admin` 公开页泄露密保问题集**（B22）、**CSP 未覆盖静态资源**（B24）、**1 个 CSP 导致前端功能损坏**（B2）、**限流可被绕过且可被内存放大 DoS**（B3 + B26）、**Agent RSA 验签路径是死代码** 等实质问题。其中 B22–B34 为二轮通读补充发现，已并入本清单 |
| Cloudflare Trace | ✅ **不是漏洞**，是可选辅助验证。但存在可重放、依赖 `cf-connecting-ip` 可信等局限，详见 §3 |
| Health 端点 | ✅ **不需要频繁检查**。现状每访客 60s 轮询已足够；它是 liveness 而非 readiness，建议去掉 version 泄露，详见 §4 |
| JWT → OAuth | 本次只给出迁移方案（未改代码），采用 **Cloudflare Access**，含边缘网关与标准 OIDC 两条路线，详见 §5 |

---

## 2. Bug 清单

> 编号说明：B1–B21 为一审发现，B22–B34 为二轮通读补充发现（均以「二轮审查新增」标记），均按风险等级归类放置，因此编号在分级之间不再严格递增。

### 🔴 高危

#### B1. 笔记正文存在存储型 XSS（markdown 原始 HTML 未消毒）
- **位置**：`src/routes/note.rs:200-210`（pulldown-cmark 渲染）、`note.rs:253`（`{{content}}` 直接插入模板）、`src/note_loader.rs:4-37`
- **问题**：`pulldown_cmark` 默认**原样透传原始 HTML**（`html::push_html` 不做任何消毒）。笔记正文里写 `<script>alert(document.cookie)</script>`、`<img src=x onerror=...>`、`[点我](javascript:alert(1))` 都会原样进 DOM 执行。笔记只能由管理员/Agent 写入，但 **Agent 私钥泄露或管理员会话被窃取 = 全站 XSS**；且笔记是公开页面，任何通过搜索/引用引入的内容都可能被利用。
- **修复**：渲染前用 `ammonia` 等白名单消毒器过滤 `html_output`；或禁用 raw HTML（`pulldown_cmark` 无内置开关，需后处理）；对 `<a href>` 用 `utils::sanitize_url` 同款策略拦截 `javascript:` 等伪协议。

#### B2. CSP 拦截内联脚本 → 笔记页代码高亮 / KaTeX 数学渲染实际失效
- **位置**：`config.toml:39-50`（`script-src 'self'`，无 `'unsafe-inline'`）+ `templates/note_detail.html:23-32`（`onload="renderMathInElement(...)"`）+ `templates/note_detail.html:33-45`（内联 `<script>` 调 `hljs.highlightAll()`）
- **问题**：`security_headers` 中间件（`src/middlewares.rs:44-92`）作用于全站。CSP `script-src 'self'` 会**同时阻止内联 `<script>` 块和内联事件处理器**。因此生产环境下笔记详情页的 `onload` 与内联脚本全部被浏览器丢弃 → **代码高亮与公式渲染静默失效**（README 宣称的表格/数学渲染功能在 Release 下是坏的）。
- **修复**：把 `hljs.highlightAll()` 与 KaTeX 初始化移入外部文件 `note_detail.js`（事件用 `addEventListener`，禁用内联 `onload`）；保留 `script-src 'self'` 不放松。

#### B3. 登录限流可被伪造头绕过（且可误伤同 NAT 用户）
- **位置**：`src/middlewares.rs:102-106`（`client_ip` 直接取 `cf-connecting-ip` / `x-real-ip` / `x-forwarded-for`）、`:109-127`（限流桶按该字符串作 key）
- **问题**：源站**无法区分**请求是否真的来自 Cloudflare。只要源站端口可直连（dev 模式 8880 暴露、防火墙误配、Tunnel 未覆盖的路径），攻击者每次换一个 `X-Forwarded-For` 值即可获得新桶 → **5 次/分钟限制形同虚设**。反向的副作用：公司 NAT 后面所有访客共享同一 IP → 单个攻击者可以**锁死整网段**（DoS）。
- **修复**：① 以 TCP 对端 `SocketAddr` 为准，只有在对端属于 Cloudflare 可信 IP 段时才信任上述头（或由 cloudflared/反向代理统一注入并剥离客户端头）；② 对头值做长度与格式校验（`x-forwarded-for` 只取最右可信一跳）；③ 限流桶建议按「IP+失败次数」计，成功后清零。

#### B4. Agent 公钥验签：RSA 路径是死代码（注释与实现不符）
- **位置**：`src/middlewares.rs:400-421`
- **问题**：`DecodingKey::from_ed_pem(...).or_else(|_| DecodingKey::from_rsa_pem(...))` 试图同时支持 Ed25519 与 RSA，但 `Validation::new(Algorithm::EdDSA)`（`:401`）把算法**硬编码为 EdDSA**。jsonwebtoken 的 `decode` 会先校验 token 头部 `alg ∈ validation.algorithms`，因此 **RS256 签发的 token 永远验签失败**——`README.md:142-170` 宣称的「Ed25519 / RS256」双支持实际只有 Ed25519 生效。
- **修复**：按公钥类型分支（Ed25519→`Validation::new(EdDSA)`，RSA→`Validation::new(RS256)`，并同时设置 `algorithms`），或删掉 RSA 分支并更新文档。

#### B5. 密码/密保答案非恒定时间比较（时序侧信道）
- **位置**：`src/middlewares.rs:165`（`payload.password == *a`）、`:184`（`answers[idx] != ans`）
- **问题**：Rust `String ==` 是短路比较，攻击者可借响应时间差逐字符探测密码（对短密码/网络抖动下虽难，但属应修的卫生问题；配合 B3 限流绕过，暴力破解成本大降）。
- **修复**：引入 `subtle` crate 做恒定时间比较（`subtle::ConstantTimeEq` 对字节串）；密保答案建议改为哈希后比对。

#### B22. `/admin` 页面无鉴权，未登录即可读取密保问题清单与 cftrace_url（二轮审查新增）
- **位置**：`src/routes/admin.rs:10-14`（路由没挂任何鉴权中间件）、`src/routes/admin.rs:32-37`（把 `admin_security_questions` 与 `cftrace_url` 注入页面）、`templates/admin.html:13`（`<script id="auth-config-json" ...>{{auth_config_json}}</script>`）
- **问题**：任何访问者 `GET /admin` 都能拿到该页 HTML，其中内嵌 JSON 直接包含服务器配置的**全部密保问题文字**与 `cftrace_url`。`admin.js:71-90` 在前端只用来在登录框里随机展示一道题——但等于把题目集**预先发放给所有未认证访问者**。密保题通常是低熵个人信息（宠物名 / 出生地 / 老师名等），配合 breach search 即可批量命中；且攻击者拿到的是**完整题目清单**，可一次性为每道题准备候选答案。再叠加 B3（可被绕过的限流）与「单题随机」实现，把「N 道题 × 5 次盲猜」的搜索空间压到极小。`cftrace_url` 一同泄露，给伪造 trace 提供了可直接对齐的目标（呼应 §3）。默认 `auth_ext_secq=false` 时不致命，一旦启用即出现「公开题目集 + 单题校验 + 可绕限流」三重弱化叠加。
- **修复**：主路径——不要在 `/admin` 公开页里下发题目集。改为登录提交后由服务端选定一个随机 `question_index` 并仅在该次响应里随题随需下发；或启用 `auth_ext_secq` 时换用 OTP / Cloudflare Access（见 §5）。最低限度：`admin_page_handler` 只输出 `auth_ext_secq / auth_ext_cftrace / 题目数量`（不含题目文字），前端只展示「第 N 题」占位；任何方案都不应在未认证响应里出现 `cftrace_url` 的具体取值。

#### B23. `notes_json` 内 `<script type="application/json">` 逃逸只替换小写 `</script>`，大写即可绕过 → 存储型 XSS（独立于 B1）
- **位置**：`src/routes/note.rs:120-122`、`templates/note.html:88-90`
- **问题**：
  ```rust
  let notes_json = serde_json::to_string(&*notes)
      .unwrap_or_else(|_| "[]".to_string())
      .replace("</script>", "<\\/script>");   // 只匹配字面小写 </script>
  ```
  HTML5 tokenizer 的 `script data end tag name` 状态对 ASCII 字母一律 `to_lower()` 后再比 `script`，因此 `</SCRIPT>` / `</ScRiPt>` 同样能终结外层 script 元素，而此处替换抓不到它们。`serde_json::to_string` 不转义 `<` / `/`，而 `note.meta.title`、`note.meta.excerpt`、`note.meta.tags[]`、`note.filename` 全部来自笔记 frontmatter，且 `create_note` / `update_note` 接受 `payload.title` / `payload.excerpt` / `payload.tags` 时不做任何大小写归一或截断。对比 `src/routes/admin.rs:39` 用的是 `.replace("</", "<\\/")` —— **那才是健壮写法**，统一拦截任意 `</` 序列，绕不开。
  攻击模式：用具备写入笔记权限的 Agent（或被泄露的 Agent 私钥）写入标题为 `</SCRIPT><script>…</SCRIPT>` 的笔记。即便当前 CSP `script-src 'self'` 会拦下内联脚本，`</SCRIPT>` 仍能提前终结 JSON 元素并把 JSON 解析 / 搜索功能整段打挂；一旦 CSP 因任何原因被放宽（如 OAuth 联调临时放开外联脚本），即升级为 XSS。
  **关键差异于 B1**：B1 提议给 `html_output`（笔记正文渲染产物）上 `ammonia` 消毒。但 `notes_json` 走的是 `serde_json::to_string(&*notes)`，frontmatter 字段**并不**经过 pulldown-cmark / ammonia。所以即便按 B1 修了正文，这个标题 / 标签 / excerpt 通道仍会单独成洞。
- **修复**：把 `notes.rs:122` 改成与 `admin.rs:39` 一致的 `.replace("</", "<\\/")`；或更彻底——对插入 `<script>` 内的 JSON 做 HTML 安全的全量转义（`<`→`\u003c`、`>`→`\u003e`、`&`→`\u0026`、并转义 U+2028 / U+2029）。同时对 `meta.title` / `meta.excerpt` / `meta.tags[*]` / `filename` 在写盘或入索引时做长度与字符集校验，封掉 `</` 之外的注入路径。**必须与 B1 一并修，否则修一处漏一处**。

### 🟠 中危

#### B6. 保存 site.toml 后笔记页缓存未失效（重启前一直显示旧页脚/旧 head）
- **位置**：`src/routes/api.rs:124-151`（`save_config` 只刷新 `html_cache`）+ `src/routes/note.rs:146-148`、`src/routes/note_admin.rs:148-151`
- **问题**：`site.toml` 的 `footer_html`/`custom_head` 同时被笔记列表页与详情页使用（`note.rs:124-143`、`:236-245`），但 `save_config` 只重建主页缓存，`note_list_html_cache` 与 `note_html_cache` 保持旧值 → **Release 模式下改页脚/head 后笔记页不生效，必须重启进程**。
- **修复**：`save_config` 中把两个笔记缓存一并置空（`*note_list_html_cache = None; note_html_cache.clear()`）。

#### B7. 内容协商响应缺少 `Vary: Accept`（CDN 错误缓存）
- **位置**：`src/routes/home.rs:74-81`、`src/routes/note.rs:82-86`（缓存命中分支）、`:170-174`（详情页缓存分支）
- **问题**：`/`、`/note`、`/note/{slug}` 都按 `Accept`/`?format=` 提供 HTML 与 Markdown 两个变体，但**缓存命中分支（Release 生产路径）不返回 `Vary: Accept`**（新鲜渲染的分支反而有，见 `note.rs:272-275`）。Cloudflare/CDN 可能把 HTML 变体缓存后直接喂给请求 `Accept: text/markdown` 的客户端（或反之）。
- **修复**：所有变体响应统一加 `Vary: Accept`；主页缓存的 HTML 分支同样要加（当前完全没有）。

#### B8. 静态资源处理器每次请求同步读 config.toml（阻塞运行时线程）
- **位置**：`src/routes/static_assets.rs:147-201`（`serve_robots`/`serve_sitemap`/`serve_favicon`/`serve_security_txt`/`serve_baidu_verify` 每个请求都调 `load_assets_config()`）
- **问题**：`load_config_section`（`src/config.rs:34-53`）每次 `fs::read_to_string("config.toml")` + TOML 解析，且是**同步 IO 跑在 async 处理器里**，会短暂占死一个 tokio worker 线程。robots/sitemap/favicon 是高频请求，白做大量磁盘 IO。
- **修复**：与其它路由一致，启动时把 `AssetsConfig` 放进 `AppState` 复用；或对这几个 handler 用 `OnceLock` 缓存。

#### B9. JWT secret 读取失败的静默降级 + 文件权限
- **位置**：`src/app.rs:20-37`
- **问题**：① 若 `.jwt_secret` 存在但读取失败（权限），会**静默生成新随机 secret** 且 `let _ =` 吞掉写回失败 → 每次重启所有会话失效且无任何日志；② `std::fs::write` 在 Unix 上默认 0644，多用户服务器上同机其他用户可读 secret。
- **修复**：读失败/写失败至少 `warn!` 并 fail-fast；写入后 `chmod 0600`；文档建议优先 `LILY_JWT_SECRET` 环境变量。

#### B10. `list_notes` 每次克隆全量笔记内容（内存浪费）
- **位置**：`src/routes/note_admin.rs:28-31` + `src/model.rs:341-347`
- **问题**：`NoteSummary.content: String` 虽 `skip_serializing`（不出现在响应里），但 `index.clone()` 把**每篇笔记全文**都克隆一遍。笔记多了之后每次列表请求都是无谓的复制+分配。
- **修复**：管理端列表接口返回不含 `content` 的轻量结构（`#[serde(skip)]` 是序列化层的，clone 不受影响），或把 `content` 改为 `Arc<str>`。

#### B11. 笔记详情页 `LINK` alternate 指向错误（根路径而非当前笔记）
- **位置**：`src/routes/note.rs:277-279`
- **问题**：`</?format=markdown>` 以 `/` 开头，是**站点根**的 markdown 变体，而当前页面是 `/note/{slug}`。搜索/UA 拿到的 alternate 是主页而非本笔记。
- **修复**：改为相对链接 `<?format=markdown>`（同列表页 `note.rs:58` 的做法一致）。

#### B24. 静态资源路由完全缺失 `security_headers`（CSP / nosniff / HSTS 都不在静态资源上，二轮审查新增）
- **位置**：`src/app.rs:74-81`
  ```rust
  let app_routes = /* home / admin / note / note_admin */
      .layer(middleware::from_fn_with_state(state, security_headers));
  app_routes.merge(routes::static_assets::router())
  ```
- **问题**：`.layer(...)` 只包裹「当时已经合并进来的路由」；之后再 `.merge(static_assets::router())` 进来的新路由**不继承**这层。结果 `/css/*`、`/js/*`、`/fonts/*`、`/images/*`、`/robots.txt`、`/sitemap.xml`、`/favicon.ico`、`/.well-known/security.txt` 全部没有 CSP、`X-Content-Type-Options: nosniff`、`Strict-Transport-Security`、`X-Frame-Options`、`Referrer-Policy`。B2 在结论里假设「`security_headers` 作用于全站」——这只对 app_routes 成立，static 不在其中。缺失 `nosniff` 是经典卫生缺口：若 `static/` 出现非预期 MIME 文件（运维误放），浏览器嗅探成 HTML / executable 会成 XSS / 反序列化平面；CSP 也不在静态资源上，等于这一支一旦有缺陷就完全没有纵深防御。
- **修复**：把 `.layer(security_headers)` 移到 `app_routes.merge(static_assets::router())` 之后再调用；或在 `static_assets::router()` 内自己挂等效的 `SetResponseHeaderLayer` 链，至少补上 `nosniff` 与 `HSTS`。

#### B25. `/api/v1/admin/login` 响应缺 `Cache-Control: no-store`（二轮审查新增）
- **位置**：`src/middlewares.rs:313-318`（`Json(AdminLoginResponse {...}).into_response()`）；同理 `src/routes/api.rs:36-45` 的所有 public `Json` 响应都无缓存头。
- **问题**：`axum::Json::into_response()` 只设 `Content-Type: application/json`，不会带缓存头。登录成功响应体里就是**新的 JWT 明文**。浏览器历史 / 前进后退缓存、Cloudflare / 反向代理对 5xx 的误缓存、或某些受信中间层缓存 POST 响应，都可能让 token 被回放。配合 B19（无吊销），8 小时内还能用。
- **修复**：登录成功响应显式 `Cache-Control: no-store`、`Pragma: no-cache`、`Expires: 0`；其余敏感 JSON（`get_home_profile` 影响较小）也建议 `private, no-store`。

#### B26. 限流 HashMap 可被无界放大 + 长时间锁住运行时（Memory / Lock DoS，叠加 B3，二轮审查新增）
- **位置**：`src/middlewares.rs:109-127`、`src/app.rs:62`（`auth_rate_limiter: Mutex<HashMap<String, Vec<Instant>>>`）
- **问题**：
  - `retain` 只在该 IP 窗口清空时才删条目，窗口长 60s。攻击者每次换一个伪造 `X-Forwarded-For`（与 B3 同源），60s 内所有伪造 IP 都会成为独立 HashMap 条目（每条至少含一个 `Instant`）。按 5 req/s × 60s 的伪造头洪流即可制造数十万条目。
  - 同时 `state.auth_rate_limiter.lock().await` 是**全登录请求串行**的锁；每次登录还要在锁内做整张 map 的 `retain` —— **每个登录请求的等待时间正比于 map 大小**。洪流即放大锁竞争，让合法登录也一起被卡死，等价一次成本极低的 DoS。
- **修复**：把 IP 来源换为可信对端 TCP `SocketAddr`（先修 B3），限流 key 才稳定可信；改成「令牌桶 + 全局上限」或 `DashMap` / `governor`（已有成熟 `tower-governor`），不再在热路径做整 map `retain`；给单个 IP 的条目设上限，超过阈值拒绝追加或改用近似计数（如 CountMin）。

#### B27. `create_note` 的 check-then-write 是 TOCTOU，同秒并发会让两个 Agent 静默丢数据（B12 的真实机理，二轮审查新增）
- **位置**：`src/routes/note_admin.rs:66-108`
- **问题**：两个 Agent / admin 在**同一秒**并发 `POST /admin/notes`，两侧 `Local::now()` 截到同一秒 → 同一 `slug` 与同一 `filename`。两侧 `metadata` 检查都断言「文件不存在」→ 两侧都写入，第二个写无声覆盖第一个，**没有任何 409、没有日志**，资产直接丢。两侧随后都 `load_all_notes` 重载，索引里只剩后写那篇。
  **与 B12 的区别**：B12 写的是「撞名 409、报错不友好」。真相是这个 409 在并发下**根本不会出现**，因为 lazy 检查只对串行请求有效；并发下是直接覆盖。
- **修复**：用 `OpenOptions::new().create_new(true).write(true)` 原子地独占创建文件，失败再 409；同时 `create`/`update` 都改用毫秒 + 随机后缀的 slug（双保险，一并满足 B12）。

#### B28. 写索引时长时间持有 `note_index` 写锁并阻塞在异步 IO（二轮审查新增）
- **位置**：`src/routes/note_admin.rs:99-102`（create）、`:144-147`（update）、`:172-173`（delete）、`src/routes/note.rs:28-33`（`reload_index_in_debug`）
- **问题**（典型）：
  ```rust
  {
      let mut index = state.note_index.write().await;       // 拿写锁
      *index = crate::note_loader::load_all_notes().await; // 持锁期间扫整盘 + 读每篇内容
  }
  ```
  `load_all_notes` 内做 `read_dir` + 逐文件 `read_to_string`（含正文），**整个 IO 都在 `write()` 持锁期间**。期间所有 `/note`、`/note/{slug}`、`/admin/notes` 的读请求都会被串行阻塞（`state.note_index.read().await`）。Agent 高频创建等于给限流 + 文件锁 + 索引写锁三处同时施压。
- **修复**：在锁外先加载 `let new_index = load_all_notes().await;`，再进入 `write().await` 做短期指针替换 `*index = new_index;`，把持锁时间从「一次全盘 IO」降到「一次赋值」。

#### B29. `/admin/notes*` 与 `/api/v1/notes*` 未挂 CORS 层（与 B16 形成"错误的安全感"，二轮审查新增）
- **位置**：`src/app.rs:72`（只有 `api_routes` 上 `.layer(cors)`）对比 `src/app.rs:78`（`note_admin::router` 在 cors 层之外被 merge）
- **问题**：`config.toml` 默认 `allow_origins = ["*"]` 时，CORS 层只覆盖 `/api/v1/admin/login`、`/api/v1/admin/configs`、`/api/v1/health`、`/api/v1/home/profile`。`/api/v1/notes*` 与 `/admin/notes*` 不在 cors 层内 —— 跨域浏览器对这些路由会按浏览器默认「无 ACAO 头 → 禁止读取」。结果：① B16 说「`*` 会暴露 token 响应」，但这其实**不适用** notes 路由；② 反过来如果未来希望从管理子站直接拉 notes 计数等数据，又会发现 CORS 缺失，再放一层容易越改越乱。
- **修复**：要么明确把 notes 路由纳入 cors 层（与 `api_routes` 合并或独立加层），要么在勘察时把它们排除并加一条注释说明「机器调用 / 不经 CORS」。

### 🟡 低危 / 建议

| # | 位置 | 问题 | 建议 |
|---|---|---|---|
| B12 | `src/routes/note_admin.rs:73` | 新建笔记 slug 用秒级时间戳，同一秒创建第二条 → 撞名 409，且报错信息不友好 | slug 追加随机后缀或毫秒 |
| B13 | `static/js/admin.js:299` | `item.innerHTML = ...${cfg.name}` 直接插入文件名 | 用 `textContent`（或 `escapeHTML`），防止服务器目录里出现恶意命名的 `.toml` 文件时 XSS |
| B14 | `src/model.rs:355`、`src/routes/note_admin.rs` | `original_slug` 字段从未被读取，纯死代码，误导维护者以为支持改 slug | 实现 slug 重命名或删除该字段 |
| B15 | `src/middlewares.rs:336,351` | 401 响应未带 `WWW-Authenticate: Bearer` 头 | 补上，符合 RFC 6750 |
| B16 | `config.toml:31`、`src/middlewares.rs:26-27` | 默认 `allow_origins = ["*"]`；token 在 header 里风险尚可控，但配合 B1（XSS）可被任意站点读取 API 响应 | 收紧为真实站点源（OAuth 改 cookie 后**必须**收紧，见 §5） |
| B17 | `src/routes/api.rs:40-45`、`src/model.rs:18-22` | `/api/v1/health` 公开暴露 `version` 字段 | 去掉或仅内网/`?verbose` 返回，防 CVE 定向打击 |
| B18 | `src/routes/note_admin.rs:66-108` | 笔记内容无大小上限，Agent 可写超大文件占满磁盘 | 加 `DefaultBodyLimit` + 内容长度校验 |
| B19 | `src/app.rs:51-67` | 会话完全无状态：登出仅前端删 sessionStorage，token 被盗后 8 小时内无法吊销、无 `iat`/`jti`、未绑定 IP | OAuth 迁移后改为服务端会话（见 §5.3） |
| B20 | `templates/note.html:24-33` | 同 B2：KaTeX `onload` 内联处理器在 CSP 下失效（列表页搜索卡片里的公式同样不渲染） | 随 B2 一并移入外部 JS |
| B21 | `src/routes/note.rs` 各分支 | 笔记页完全没有 `Cache-Control`/`Last-Modified`（列表与详情、缓存与非缓存分支均无） | 列表页可给短 TTL（如 300s），详情页给 `public, max-age=...` 并配 `Vary: Accept` |
| B22 | `src/routes/admin.rs:32-37`、`templates/admin.html:13` | `/admin` 页面无鉴权即向未登录访问者下发完整密保问题列表与 `cftrace_url`，配合 B3/B26 可低成本弱化 `auth_ext_secq` | 不在公开页下发题目集；改为登录端点按需随机下发，或迁 Cloudflare Access（§5） |
| B23 | `src/routes/note.rs:120-122` | `notes_json` 仅小写 `</script>` 替换，`</SCRIPT>` 大小写不敏感即可逃逸 JSON `<script>` 区，形成独立于 B1 的存储型 XSS / 页面破坏 | 改为 `.replace("</", "<\\/")`，并对 frontmatter 字段做长度与字符集校验；必须与 B1 一并修 |
| B24 | `src/app.rs:74-81` | `.layer(security_headers)` 未覆盖后 merge 的 static_assets 路由 → `/css /js /fonts /images` 等无 CSP/nosniff/HSTS | 把该 layer 移到 merge 之后再调用，或在 static_assets 内自行挂 `SetResponseHeaderLayer` |
| B25 | `src/middlewares.rs:313-318` | 登录成功响应无 `Cache-Control: no-store`，JWT 可被浏览器/代理缓存回放 | 显式 `no-store` / `Pragma: no-cache` / `Expires: 0` |
| B26 | `src/middlewares.rs:109-127`、`src/app.rs:62` | 限流 `HashMap` + `Mutex` + 全 map `retain`，可被伪造头无界放大并锁住运行时 | 可信对端 IP + `tower-governor`/`DashMap` 令牌桶 + 单 IP 条目上限 |
| B27 | `src/routes/note_admin.rs:66-108` | `create_note` check-then-write 为 TOCTOU，同秒并发无 409 直接覆盖丢数据 | `OpenOptions::create_new(true)` 原子独占创建 + 毫秒/随机后缀 slug（兼修 B12） |
| B28 | `src/routes/note_admin.rs:99-102` 等 | 持 `note_index` 写锁期间做整盘 IO，期间全站笔记读被串行阻塞 | 锁外先 `load_all_notes`，再短期持锁做指针替换 |
| B29 | `src/app.rs:72` 对比 `:78` | `/admin/notes*` 与 `/api/v1/notes*` 在 cors 层之外，B16 的"`*` 暴露 token 响应"其实不适用 notes 路由 | 明确这些路由是否纳入 cors，并加注释说明"机器调用 / 不经 CORS" |
| B30 | `src/routes/static_assets.rs:41, 171-191` | `/{baidu_verify_codeva}` 兜底路由对任意不存在的单段 URL 返回 **400 BAD_REQUEST** 而非 404，破坏"未知路径→404"与扫描器体感 | 不符合 `baidu_` 形状直接 404，或改更精准的 `/baidu_{code}.html` 静态路由 |
| B31 | `src/routes/note_admin.rs:59-62` | `build_note_file_content` 中 `toml::to_string(meta).unwrap_or_default()` 失败时静默落盘为 `---\n---\n\n{content}`，frontmatter 全丢且无 5xx/无日志 | 改返回 `Result`，写盘前失败即 500 + `error!` 标注非法字段 |
| B32 | `src/model.rs:210-243` 对比 `config.toml:9-23` | `AssetsConfig::default()` 与线上 `config.toml` 漂移：`compression_types`、`brotli_quality`、`gzip_level`、`assets_dirs` 都不一致，维护者会按默认值调试半天 | 把 `Default` 对齐 config.toml，或下沉到 `load_assets_config` 的 fallback 并注明"线上以 config.toml 为准" |
| B33 | `src/routes/note.rs:184-198` | `/note/{slug}?format=markdown` 直接 `read_to_string` 返回**整篇原始 .md**（含 frontmatter 与 `updated_at`），对爬虫/聚合器泄露 Agent 编辑时间戳 | 剥掉 frontmatter 返回纯净正文，或在 README 明确"markdown 变体含元数据" |
| B34 | `static/js/admin.js:104-113` | 用 `innerHTML =` 拼装含自定义元素的 DOM，依赖运行时 upgrade 顺序，与未来 CSP 收紧相性差 | 改用 `document.createElement` + `appendChild` 命令式构造 |

### ✅ 做得好的点（保留）
- 配置文件编辑用**白名单**（`get_editable_configs`），杜绝路径穿越（`src/config.rs:119-132`、`src/routes/api.rs:54-67`）；
- sitemap 写入前校验 XML 前缀、TOML 写入前校验语法+schema（`api.rs:83-119`）；
- `html_escape`/`sanitize_url` 在主页/项目/关于等渲染路径全覆盖（`src/render.rs`、`src/utils.rs`）；
- `.jwt_secret`、`.agent*` 已加入 `.gitignore`，且默认占位密码 `CHANGE_YOUR_PASSWORD` 被服务端硬拒绝（`middlewares.rs:161-164`）；
- JWT 算法白名单（HS256 / EdDSA 严格区分），无 alg 混淆面；CF Beacon token 有字符白名单（`render.rs:114-123`）；
- 预压缩、304 缓存、note HTML 缓存淘汰策略（`note.rs:261-267`）设计合理。

---

## 3. Cloudflare Trace：不是漏洞，是辅助验证

**结论：`cf_trace`（`auth_ext_cftrace`）机制本身不是漏洞**，它属于可选的第二层辅助验证，且默认关闭（`config.toml:36`）。审查时不应将其列为高危。但要清楚它的真实边界：

1. **它不认证身份，只证明环境**：`h/loc/warp/gateway/ip` 是「浏览器→Cloudflare 边缘」这一跳的连接状态快照（对照 `/cdn-cgi/trace` 输出），与「谁是管理员」无关——真正的凭据仍然是密码。
2. **可重放/可伪造**：trace 文本由浏览器 fetch 后原样 POST（`static/js/admin.js:127-157`），服务端无法验证其时效性。攻击者拿一次合法 trace 即可反复使用（换 IP 也不行，见下条）。
3. **依赖 `cf-connecting-ip` 可信链**：`middlewares.rs:242-247` 的「trace ip == 客户端 ip」校验建立在客户端传来的 `cf-connecting-ip` 头上。**只要源站可被直连，这个头就可伪造**，整条链失效。当前部署（cloudflared + 防火墙只放 443）已缓解，但应把「只信任来自 Cloudflare 的流量」固化为部署约束，而不是写在代码假设里。
4. **可用性副作用**：`warp=on`/`gateway=on`/`loc` 白名单会锁死非 WARP 网络（移动网、企业网）与海外访问者。作为可选开关尚可，不建议强制。

**处置建议**：保留为可选辅助（默认关闭即可）；不要把它当成主要安全控制；密码认证本身将在 OAuth 迁移（§5）中被 Cloudflare Access 取代，届时 `auth_ext_cftrace`、`auth_ext_secq`、`admin_password` 全部可以退役。

---

## 4. Health 端点：不需要频繁检查

**结论：不需要。** 现状与数据：

- 前端行为：首页头像状态点**每个访客每 60s** 轮询一次 `/api/v1/health`（`static/js/user.js:17-60`），失败只改 CSS 类，无副作用。
- 服务端成本：`health_handler` 是 O(1) 静态 JSON，无 IO、无锁、无日志（`src/routes/api.rs:40-45`）。README 压测显示单机可到数十万 QPS，60s 一次的轮询压力可以忽略。

**建议**：
1. 外部监控（UptimeRobot/自建探活）间隔 **30–60s** 即可，无需亚秒级；告警阈值建议「连续 2~3 次失败」再触发，避免单次瞬断误报。
2. 认清它是 **liveness（进程活着）而非 readiness（服务可用）**：现在它永远返回 `ok`，即使 `notes/` 目录不可写、TOML 配置损坏、TLS 证书过期也不会有任何反映。若想要真 readiness，可让它顺带检查「notes/ 可写 + 配置可解析」，失败返回 503（注意保持 O(1) 与无缓存）。
3. 从公开响应中移除 `version`（B17），或仅通过内部头/参数返回。
4. 若担心轮询流量，可给响应加 `Cache-Control: no-store`（现状无缓存头，但 JSON 默认不被浏览器缓存，问题不大）。

---

## 5. 简易 JWT → OAuth（Cloudflare Access）迁移方案

### 5.1 目标与总体思路

- 目标：管理后台（`/admin`、`/api/v1/admin/*`、`/admin/notes/*`）**不再使用「密码 + 密保 + cf-trace」换取 JWT**，改为 Cloudflare Access 身份认证；浏览器端不再持有可被 XSS 窃取的长期 token。
- 现状铺垫：站点已部署在 cloudflared 之后（README §部署），这使 Cloudflare Access 的接入成本极低。
- 两条路线，可组合：
  - **路线 A（边缘网关，改动最小，1 小时级）**：Access 直接在边缘挡住 `/admin*`，应用信任边缘注入的身份；
  - **路线 B（应用内标准 OAuth2/OIDC，推荐，1 天级）**：应用实现授权码 + PKCE 流程，Cloudflare Access 作为 OIDC Provider，应用自己发会话。

### 5.2 路线 A：Cloudflare Access 边缘网关（最快落地）

1. Cloudflare Dashboard → Zero Trust → Access → Applications，新建 **Self-hosted** 应用，策略覆盖：
   - `https://<你的域名>/admin`
   - `https://<你的域名>/admin/*`
   - `https://<你的域名>/api/v1/admin/*`（含 `/admin/notes*`，注意 Access 路径匹配是按 URL 前缀）
   - 身份策略：只允许你自己的邮箱（可叠加 One-time PIN / 硬件密钥 MFA）。
2. 应用内新增 `access_auth_middleware`（替换 `admin_auth_middleware` 的 JWT 校验）：
   - 校验请求头 `Cf-Access-Jwt-Assertion`：用 JWKS（`https://<team>.cloudflareaccess.com/cdn-cgi/access/certs`，EdDSA）验签，检查 `aud == 应用的 AUD tag`、`exp`、`iss`；
   - 兜底校验 `Cf-Access-Authenticated-User-Email` 是否在配置白名单（**仅当**对端确属 Cloudflare 时可信，见 B3 的修复原则）；
   - 命中即放行，并把 email 写入请求扩展/日志。
3. 前端：登录对话框删除；`/admin` 未认证用户会被边缘重定向到 Access 登录页，登录后原路返回。`admin.js` 里 `apiFetch` 改为**不带任何 token**（或保留旧 JWT 逻辑作为降级开关）。
4. 边界与坑：
   - 只有「被 Access 策略覆盖的路径」受保护；确保没有漏网的 `/api/v1/admin/*` 变体（如带尾斜杠、大小写）。
   - 若源站可被直连，Access 会被绕过——依赖 cloudflared 与防火墙收紧（现状已满足）。
   - Agent 的 `/api/v1/notes*` 不走 Access（机器调用），保持公钥 JWT 专线（先修 B4）。

### 5.3 路线 B：应用内标准 OAuth2/OIDC 授权码 + PKCE（推荐，与「改 oauth」语义最吻合）

Cloudflare Access 是合规 OIDC Provider，端点（team 域固定，如 `https://sulyhub.cloudflareaccess.com`）：

| 端点 | URL |
|---|---|
| OIDC Discovery | `https://<team>.cloudflareaccess.com/.well-known/openid-configuration` |
| Authorize | `https://<team>.cloudflareaccess.com/cdn-cgi/access/authorize` |
| Token | `https://<team>.cloudflareaccess.com/cdn-cgi/access/token` |
| UserInfo | `https://<team>.cloudflareaccess.com/cdn-cgi/access/userinfo` |
| JWKS | `https://<team>.cloudflareaccess.com/cdn-cgi/access/certs` |

**新流程**：

```
浏览器                    lily-nest                       Cloudflare Access (OIDC)
  │  GET /admin (无会话)     │                                  │
  │ ◄── 302 /auth/login ────┤                                  │
  │  GET /auth/login ──────►│ 生成 state+nonce+PKCE verifier    │
  │ ◄── 302 authorize?... ──┼───► code_challenge=S256 ────────►│
  │ ◄──────────────────────── 登录/授权(邮箱+MFA) ◄─────────────┤
  │  GET /auth/callback?code&state ──► 校验 state               │
  │                              │ code+verifier+secret ────► token 端点
  │                              │ ◄── access_token + id_token
  │                              │ 验签 id_token(JWKS, aud/iss/exp/nonce)
  │                              │ userinfo 取 email，白名单校验
  │                              │ 签发 HttpOnly 会话 cookie
  │  GET /admin ────────────► session_auth_middleware 放行
```

**服务端设计要点**：

1. **新增依赖**：当前 `Cargo.toml` **没有任何 HTTP 客户端**（`src/main.rs` 只做服务），OAuth 换码、JWKS 拉取需要新增 `reqwest`（`rustls-tls` 特性）或 `ureq`；JWT 验签复用已有 `jsonwebtoken`（JWKS 是 JSON 形式密钥，用 `DecodingKey::from_jwk`）。
2. **新增 `[oauth]` 配置段**（放 `config.toml`，secret 走环境变量，沿用 `LILY_JWT_SECRET` 模式）：
   ```toml
   [oauth]
   team_domain = "sulyhub"                 # → https://sulyhub.cloudflareaccess.com
   aud = "你的Access应用AUD tag"
   client_id = "你的client_id"
   allowed_emails = ["you@example.com"]
   session_ttl_secs = 3600
   # client_secret 用环境变量 LILY_OAUTH_CLIENT_SECRET 注入
   ```
3. **新增 `src/auth.rs`**：OIDC discovery 缓存、JWKS 拉取与缓存（TTL 1h）、PKCE（`rand` 已有）、`state`/`nonce` 一次性存储（内存 Map + TTL，过期清理，复用 `auth_rate_limiter` 的模式）。
4. **会话替换 JWT**：登录成功后**不再发 JWT**，改为服务端会话：`session_id`（随机 32 字节，hex）→ `AppState.sessions: Mutex<HashMap<session_id, Session{email, expires_at}>>`，cookie 设置 `HttpOnly; Secure; SameSite=Lax; Path=/`。登出 = 服务端删除会话（`/auth/logout`），真正可吊销（修复 B19）。内存会话在单实例下足够，多实例需换成共享存储（如 SQLite/Redis）。
5. **中间件**：`admin_auth_middleware` 改为读 cookie 会话；`note_auth_middleware` 保留（Agent 公钥 JWT 专线），但允许「会话 cookie 或 Agent token」任一通过。
6. **限流**：`/auth/login`（防 authorize 轰炸）与 `/auth/callback`（防换码轰炸）沿用现有 `auth_rate_limiter`，key 用真实对端 IP（修复 B3 后再用）。
7. **安全细节**：`state` 必须校验（防 CSRF 登录）；`nonce` 校验（防重放）；`redirect_uri` 精确匹配配置；ID token 验 `iss == <team>/.well-known/openid-configuration` 的 issuer 与 `aud == AUD`；token 换码一律服务端进行（浏览器不接触 code 之外的任何东西，CSP 无需放开 `connect-src`）。

**前端改造（`templates/admin.html` + `static/js/admin.js`）**：
- 删除密码对话框、密保题、`fetchTrace()` 全部逻辑；改为「使用 Cloudflare Access 登录」按钮 → `location.href = '/auth/login'`；
- `apiFetch` 去掉 `Authorization: Bearer` 头，改用同源 cookie（`credentials: 'same-origin'`，同源下默认携带）；
- 401 时 `location.href = '/auth/login'` 而不是弹窗；
- 删除 sessionStorage 里 `auth_jwt` 相关读写与迁移代码。

**服务端删除项**：
- `handle_admin_login` 的密码/密保/cf-trace 全链路（`middlewares.rs:94-319`）、`AdminLoginRequest.cf_trace/answer/question_index`（`model.rs:268-273`）、`SecurityConfig` 中 `admin_password/admin_security_answers/auth_ext_secq/auth_ext_cftrace/cftrace_url/allowed_locs`（`model.rs:152-159`，可留作兼容但不建议）；
- `admin.js` 对应的 UI 与逻辑；
- 若担心过渡期，可加 `auth_mode = "password" | "oauth"` 配置二选一，默认 oauth，密码模式标记 deprecated。

### 5.4 文件级改动清单（路线 B）

| 文件 | 改动 |
|---|---|
| `Cargo.toml` | + `reqwest(rustls-tls)`（或 `ureq`）、`subtle`（顺带修 B5） |
| `config.toml` | + `[oauth]` 段；`allow_origins` 收紧为真实站点（B16）；CSP 无需变（服务端换码） |
| `src/auth.rs`（新） | OIDC discovery / JWKS 缓存 / PKCE / state+nonce 存储 / session 管理 |
| `src/routes/auth.rs`（新） | `/auth/login`、`/auth/callback`、`/auth/logout` |
| `src/state.rs` | + `sessions: Mutex<HashMap<...>>`、`oauth_config: Arc<...>` |
| `src/middlewares.rs` | 删 `handle_admin_login`；`admin_auth_middleware` 改会话校验；`note_auth_middleware` 修 B4；限流修 B3；密码比较改恒定时间（B5） |
| `src/routes/api.rs` | 删 `/admin/login` 路由；health 去掉 version（B17）；`save_config` 补笔记缓存失效（B6） |
| `src/routes/note.rs` | Vary 修复（B7）、LINK 修复（B11）、XSS 消毒（B1）、缓存头（B21） |
| `templates/note_detail.html`、`templates/note.html` | 内联脚本/onload 移入外部 JS（B2/B20） |
| `templates/admin.html`、`static/js/admin.js` | 登录 UI 改造、cookie 会话、删 JWT 逻辑、`textContent` 修复（B13） |
| `src/routes/static_assets.rs` | 配置缓存化（B8） |
| `README.md` | 更新认证章节（§安全验证、§Agent 专线） |

### 5.5 迁移步骤与回滚

1. **Cloudflare 侧**：建 Access 应用（Self-hosted），拿 AUD tag；在应用的 OIDC 设置里配置 `client_id`/`client_secret`，Redirect URI 填 `https://<域名>/auth/callback`；身份策略限自己邮箱。
2. **代码**：按 §5.4 落地；先合入修复类改动（B1–B11），再合入 OAuth。
3. **灰度**：`auth_mode` 默认 password 跑通 OAuth 后再切 oauth；观察 `/auth/callback` 失败率。
4. **回滚**：保留旧二进制；`auth_mode` 切回 password 即回滚，会话表清空即可。
5. **上线检查**：见 §5.6。

### 5.6 上线检查清单

- [ ] 直连源站（绕过 cloudflared）时 `/api/v1/admin/*` 返回 401/403，且伪造 `Cf-Access-*` 头无效（B3 修复后）
- [ ] `/auth/callback` 无 `state`/`nonce` 或重复使用时报 400
- [ ] 篡改/重放 ID token（改 `aud`/`iss`/`exp`）被拒
- [ ] 非白名单邮箱登录后被拒，且日志记录
- [ ] 登出后 cookie 服务端失效，重放旧 cookie 返回 401
- [ ] 笔记页代码高亮与公式在 Release 模式正常渲染（B2 修复验证）
- [ ] 笔记正文含 `<script>`/`javascript:` 链接时页面安全（B1 修复验证）
- [ ] 笔记页/主页 304 与 CDN 缓存行为正确（B7 修复验证）
- [ ] Agent 公钥 JWT 专线回归（Ed25519 + RSA 各一例，B4 修复验证）

### 5.7 Agent 专线 API 的处置

- **保留**：`/api/v1/notes*` 的机器认证（`note_auth_middleware` 非对称分支）与人类登录是两回事，OAuth 不覆盖机器调用。**先修 B4** 让 RSA 真正可用，或明确只支持 Ed25519 并更新 README。
- **可选演进**：改用 Cloudflare Access **Service Token**（请求头 `Cf-Access-Client-Id` + `Cf-Access-Client-Secret`，服务端用 JWKS 校验 `aud` 为 Service Token AUD），彻底免去私钥分发；代价是需要维护 token 轮换。

---

## 6. 二轮审查补充：代码改进与合并修复优先级

> 本节为二轮通读在 B1–B21 之外发现的非安全工程改进（I1–I7）与「与 B 系列应一并合并修复」的优先级建议。其中 B22–B34 已并入 §2 的对应分级，下表只总结"哪些finding应该一起改"。

### 6.1 代码改进（非安全）

#### I1. 用 `Arc<str>` 替代克隆正文（呼应 B10 的更彻底做法）
`NoteSummary.content: String` 在 `list_notes` 里被整体 clone（B10 提到的）。更彻底的做法：把 `content: Arc<str>` 存进 `note_index`，`list_notes` 用 `Arc::clone` 而非 `String::clone`，复制开销从 O(n) 降到 O(1)。

#### I2. `handle_admin_login` 主体应改用 `Json<AdminLoginRequest>` 提取器
当前 `src/middlewares.rs:130-137` 手写 `axum::body::to_bytes(...).await` 再 `serde_json::from_slice`，失去 axum 对 `Content-Length` / `Content-Type` 的统一校验，也与 `save_config` / `create_note` 等用 `Json<Payload>` 提取器的风格不一。把限流提前到提取器之前的中间件层，再把 body 解析交回 `Json<T>`。

#### I3. 限流统一到 `tower-governor`
当前 `auth_rate_limiter` + `Vec<Instant>` + `Mutex` 是手写轮子，已暴露 B26 的内存与锁问题。`tower-governor` / `governor` 的 LeakyBucket 已经支持：基于对端 IP 的 key、自动 LRU 回收、异步 sharded 锁。

#### I4. `build_note_file_content` 规整为 `Result`
让 `build_note_file_content` 返回 `Result<String, toml::ser::Error>`，`create_note` / `update_note` 据此返 400 / 500，避免 B31 的静默失败。

#### I5. pulldown-cmark 渲染选项集中成配置
`note.rs:201-208` 硬编码 `ENABLE_TABLES | ENABLE_MATH | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS | ENABLE_FOOTNOTES | ENABLE_SMART_PUNCTUATION`。引入类似 `[markdown] .tables / .math / ...` 的配置位，既便于灰度，也便于单测。

#### I6. 引入 `cargo audit` / `cargo-deny` 到 CI
`Cargo.toml` 锁版本但未提 `cargo-audit` / `cargo-deny`。§7 审查范围也已自陈"未做依赖漏洞库扫描"。建议补一个 step：
```yaml
- uses: rustsec/audit-check@v2
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

#### I7. dev 模式默认绑本机
`ServerConfig::http_port = 8880` 在 debug 模式直接绑 `[::]:8880`。dev 模式 IP 头可被伪造（B3 的根因之一），而默认绑到了所有网卡。建议加 `--bind 127.0.0.1` 启动 flag，或在 dev 时默认 `127.0.0.1`，配合隧道使用。

### 6.2 与 B 系列应「合并修复」的优先级

| 并入 / 关联 | 性质 | 建议修复顺序 |
|---|---|---|
| **B23 + B1** | 两者同为存储型 XSS，但清洗路径独立：B1 修 `html_output`，B23 修 `notes_json`。务必**两处一起修**，否则修一处漏一处 | 第一批 |
| **B22 + B3 + B26** | 三者黄金组合：题目泄露 + 限流可绕 + 内存放大 = 远程低成本弱化或 DoS。先把 IP 可信化做掉，限流 / 速率 / 题目三件一起改（B26 顺带替换为 governor） | 第一批 |
| **B24 + B2/B20** | 要让 CSP 真正全站生效：把内联脚本/onload 全挪到外联 JS（B2/B20），再把 `security_headers` 挂到顶层覆盖静态资源（B24） | 第一批 |
| **B25 + B19** | 登录响应 `no-store` + 会话可吊销（OAuth 迁移），两条都做完才是「token 被动泄露」的完整收敛 | 第二批 |
| **B27 + B12** | slug 改毫秒 + 随机后缀（B12），同时 `OpenOptions::create_new` 原子创建（B27），并发也安全 | 第二批 |
| **B28** | 与 B26 同改：锁外加载 + 短期持锁，限流与索引锁一并消除长尾阻塞 | 第二批 |
| **B29** | 与 §5 OAuth 迁移一并收敛：明确机器调用路由是否纳入 CORS 的策略，写进 README | 第三批 |

---

## 7. 审查范围与方法

- 静态通读全部源码（`src/*.rs`、`src/routes/*.rs` 共 12 个模块 + 4 个模板 + 4 个前端 JS + `config.toml`）；
- 重点核对：认证/授权链路、CSP 与内联脚本的相互作用、缓存一致性、内容协商头、路径遍历面、限流可信边界、依赖版本（`jsonwebtoken 10`、`axum 0.8`、`pulldown-cmark 0.13`）；
- 二轮补充通读（B22–B34、§6）：重点核对 `.layer()` 覆盖边界、`<script>` 内嵌 JSON 的 HTML 逃逸大小写敏感性、async 处理器内的同步 IO 持锁、check-then-write TOCTOU、HashMap 无界放大；
- 未做：动态运行测试、依赖漏洞库扫描（建议 CI 加 `cargo audit`，详见 §6.1 I6）、模糊测试。
