# ContextLab Release Candidate / ContextLab 发布候选

**Candidate / 候选版本:** `0.1.0-pre.1`
**Prepared / 编制日期:** 2026-09-11
**Governing criteria / 治理条件:** `docs/roadmap/completion-criteria.md`
**Plan backlog / 计划积压:** closed by `docs/roadmap/plan-closure-ledger.md`

This document is the human acceptance checkpoint before the first public publication. It states what
the candidate contains, what was verified on this machine, what it deliberately does not claim, and
what a reviewer should look at.

本文档是首次公开发布前的人工验收检查点。它说明本候选包含什么、在本机验证了什么、刻意不声明什么，以及验收者应当看什么。

## 1. What the candidate contains / 候选内容

A context-first, bilingual, evidence-gated engineering platform with a Rust workspace, an Axum API,
a design-system-first Next.js workspace, a TypeScript SDK pair, 25 PostgreSQL migrations, and the
roadmap/governance/plan corpus that produced it.

一个 Context-first、中英双语、以证据为门的工程平台，包含 Rust workspace、Axum API、design-system-first 的 Next.js 工作台、
两套 TypeScript SDK、25 个 PostgreSQL migration，以及产出它的路线图／治理／计划文档集。

| Area / 领域 | Delivered / 已交付 |
| --- | --- |
| Domain crates / 领域 crate | `context-core`, `versioning`, `diff-engine`, `evaluation`, `graph`, `knowledge`, `memory`, `mcp`, `model-gateway`, `plugin-runtime`, `workflow`, `storage`, `auth` |
| API / 服务 | Axum public router plus a default-off protected router; checked-in OpenAPI contract; GET/POST route catalogs guarded by drift tests |
| Storage / 存储 | In-memory and PostgreSQL adapters behind one contract; 25 forward-only migrations; guarded commit writer; audit-retention governance with a restricted purge executor |
| Security / 安全 | HMAC and OIDC principal authentication, Context authorization, protected-route rate limiting, redaction-by-default error surfaces, fail-closed configuration |
| Web / 前端 | Design-system-first workspace with `data → presenter → screen` separation and same-origin BFF routes that forward only a request-scoped Bearer credential |
| SDK / SDK | Public `@contextlab/ts-sdk` and non-public `@contextlab/local-sdk` with strict fail-closed parsers |
| Other apps / 其他应用 | `apps/cli` read-only inspection, `apps/desktop` Tauri shell |
| Docs / 文档 | Bilingual README, architecture, ADRs, API references, storage notes, contribution and security policy, roadmap receipts |

## 2. Fresh verification on this machine / 本机新鲜验证

All of the following was re-run on 2026-09-11 against this worktree, not copied from earlier
receipts.

以下全部于 2026-09-11 在当前工作树重新运行，均非抄录自旧回执。

| Gate / 门禁 | Command / 命令 | Result / 结果 |
| --- | --- | --- |
| Rust workspace | `cargo test --workspace` | `930 passed, 0 failed, 46 ignored` |
| API crate | `cargo test -p contextlab-api` | `229 passed, 0 failed` |
| Storage crate | `cargo test -p contextlab-storage` | `365 passed, 0 failed, 46 ignored` |
| Format | `cargo fmt --all -- --check` | passed / 通过 |
| Lints | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | passed / 通过 |
| Public SDK | `pnpm check:web` (stage 1) | `15 passed, 0 failed` |
| Local SDK | `pnpm check:web` (stage 2) | `148 passed, 0 failed` |
| Web | `pnpm check:web` (stage 3) | `310 passed, 0 failed` |
| Web build | `pnpm --filter @contextlab/web build` | compiled and prerendered / 编译并预渲染成功 |
| Contract fixtures | `pwsh -NoProfile -File tests/contract/verify-local-contracts.test.ps1` | passed / 通过 |
| Static verifier | `pwsh -NoProfile -File scripts/verify-local-contracts.ps1` | source/DTO/route/GraphDiff checks passed; `overall=unobserved` only because no unified diff was supplied |
| Shell self-tests | the four `scripts/*.test.sh` guards | `4 passed` |
| Sole GraphDiff | `scripts/verify-local-contracts.ps1` | `graph_diff_application=passed count=1` |

The 46 ignored Rust tests are the named PostgreSQL cases. They are not skipped by accident: each one
declares that it requires an empty disposable database through `CONTEXTLAB_TEST_DATABASE_URL`, and
they run in CI via `scripts/verify-disposable-postgres-storage.sh`.

46 个 ignored Rust 测试即具名 PostgreSQL case。它们不是被意外跳过：每一个都声明自己需要通过
`CONTEXTLAB_TEST_DATABASE_URL` 提供空 disposable database，并会在 CI 中通过
`scripts/verify-disposable-postgres-storage.sh` 运行。

## 3. Environment repairs made for this candidate / 为本候选修复的环境问题

1. `rust-toolchain.toml` pinned `channel = "stable"`, while every recorded verification receipt was
   produced with Rust `1.85.0` — the declared MSRV. The local `stable` toolchain was also broken
   (missing `rustc.exe`), which made a bare `cargo` invocation hang and spin a repair install. The
   pin is now `1.85.0`, matching `rust-version`, the receipts, and the CI workflow, which was pinned
   the same way.
2. `.env.example` pointed the OpenAI-compatible base URL at a third-party relay. It now points at a
   neutral local placeholder, because a public template should not recommend an unaffiliated
   gateway.
3. `.gitignore` now excludes local agent workspaces (`.agents/`, `.codex/`) and stray `*.rmeta`
   build artifacts.

1. `rust-toolchain.toml` 固定 `channel = "stable"`，但所有已验证回执都是用 Rust `1.85.0`（即声明的 MSRV）产出的；
   本机的 `stable` 工具链同时已损坏（缺 `rustc.exe`），导致裸 `cargo` 调用挂起并反复触发修复安装。现固定为
   `1.85.0`，与 `rust-version`、既有回执以及同样被固定的 CI workflow 三者一致。
2. `.env.example` 的 OpenAI-compatible base URL 指向第三方中转。现改为中性的本地占位，因为公开模板不应推荐无关联的 gateway。
3. `.gitignore` 现排除本地 agent 工作区（`.agents/`、`.codex/`）与零散 `*.rmeta` 构建产物。

## 4. What this candidate does not claim / 本候选不声明的范围

- **Production readiness.** The protected route, write paths, and deployment topology have local
  and CI-oriented tests, not production operation.
- **Live PostgreSQL runtime on this machine.** Docker Desktop cannot reach a Linux engine here, no
  PostgreSQL binaries are installed, and WSL has no usable distribution. The PostgreSQL evidence is
  therefore `unobserved` locally and runs in CI instead.
- **Authenticated browser flow.** The browser smoke exercises preview mode. A browser session that
  authenticates through the BFF into the protected Axum runtime has not been observed here.
- **Public write surface.** Promoting the protected commit route to public REST/OpenAPI/SDK remains a
  deferred decision with external prerequisites.
- **A perfect or finished product.** ContextLab is an early platform. Many roadmap areas — workflow
  execution, knowledge ingestion, embedding search, plugin loading, marketplace, and much of the
  Desktop and CLI experience — are scaffolded, partially delivered, or not started.

- **Production readiness。** protected route、写入路径与部署拓扑只有本地与面向 CI 的测试，没有生产运行经验。
- **本机的 live PostgreSQL runtime。** 本机 Docker Desktop 无法连接 Linux engine，未安装 PostgreSQL 二进制，WSL 也没有可用发行版。
  因此 PostgreSQL 证据在本机为 `unobserved`，改由 CI 运行。
- **Authenticated browser flow。** 浏览器 smoke 走的是 preview mode。经 BFF 认证进入 protected Axum runtime 的浏览器会话尚未在本机观测。
- **Public write surface。** 把 protected commit route 提升为 public REST/OpenAPI/SDK 仍是有外部前置的延期决策。
- **"完美"或"完成"的产品。** ContextLab 仍是早期平台。workflow execution、knowledge ingestion、embedding search、plugin loading、
  marketplace，以及 Desktop 与 CLI 的大部分体验，处于搭骨架、部分交付或尚未开始的状态。

## 5. Acceptance checklist / 验收清单

Reviewer, please confirm on the running local application:

验收者请在本地运行的应用上确认：

1. The workspace renders and the layout holds at desktop and narrow widths. / 工作台能渲染，且桌面与窄屏布局都站得住。
2. Context discovery, commit history, component inventory, and evaluation reads show API-shaped data. / Context discovery、commit history、component inventory 与 evaluation 读取显示 API-shaped 数据。
3. The private inspectors accept a Bearer token in memory only, and fail closed with a redacted message when the upstream is not configured. / 私有 inspector 只在内存中接受 Bearer token；上游未配置时以脱敏信息 fail closed。
4. Error surfaces never echo an upstream diagnostic verbatim. / 错误面从不原样回显上游诊断信息。
5. Nothing on screen exposes a credential, a raw component body, or a real hostname. / 屏幕上不暴露凭据、component 正文或真实主机名。

Record the outcome in this file before publication. A rejection should name the screen and the exact
step that failed so it becomes a normal Necessity Record instead of an undocumented fix.

请在发布前把结论记入本文件。若验收不通过，请指明具体页面与失败步骤，使其按正常 Necessity Record 处理，而不是变成无记录的临时修补。

### Reviewer sign-off / 验收签署

`accepted / 已通过` on 2026-09-17. The reviewer accepted the running candidate and authorized
publication to GitHub. The five checklist items above were reviewed against the live local
application described in the session receipt below.

2026-09-17 标记为 `accepted / 已通过`。验收者接受了运行中的候选版本，并授权发布到 GitHub。上述五项检查清单已对照下方会话回执所述的本地运行应用完成复核。

### Observed acceptance session / 已观测的验收会话

On 2026-09-11 the candidate was served locally and inspected through a real headed browser session,
not through fixtures:

2026-09-11 本候选在本地实际启动，并通过真实的 headed 浏览器会话检查，而非 fixture：

- API: `target/debug/contextlab-api.exe` on `127.0.0.1:3100`, `CONTEXTLAB_API_ROUTE_MODE=public`,
  `CONTEXTLAB_GRAPH_REPOSITORY=memory`.
- Web: `next start` on `127.0.0.1:3000`, with the same-origin BFF pointed at the local API through
  the gitignored `apps/web/.env.local`.
- `GET /` returned `200` with a 152 KB server-rendered page containing the ContextLab shell.
- Rendered sections observed: Context Detail, Context Graph (10 nodes / 10 relationships), Version
  History (`main`, 1 change, `create_context` payload), Component Inventory (5 fingerprinted
  components), Scorecard (2 metrics over 1 run), and the Operations inspector stack.
- Browser console error collection was empty.
- The private branch-head inspector was exercised with a memory-only token. Because the local API
  runs in public mode, the protected route is not mounted, and the UI failed closed with the stable
  bilingual message "Branch-head read failed / 分支 head 读取失败" plus "The protected branch-head
  read failed closed. / 受保护的分支 head 读取已失败关闭。" No upstream diagnostic, credential, or
  hostname appeared in the rendered output.

- API：`target/debug/contextlab-api.exe` 监听 `127.0.0.1:3100`，`CONTEXTLAB_API_ROUTE_MODE=public`、
  `CONTEXTLAB_GRAPH_REPOSITORY=memory`。
- Web：`next start` 监听 `127.0.0.1:3000`，同源 BFF 通过被 gitignore 的 `apps/web/.env.local` 指向本地 API。
- `GET /` 返回 `200`，服务端渲染页面 152 KB，包含 ContextLab 外壳。
- 已观测的渲染区块：Context Detail、Context Graph（10 个节点 / 10 条关系）、Version History（`main`、1 处变更、
  `create_context` payload）、Component Inventory（5 个带 fingerprint 的组件）、Scorecard（1 次运行上的 2 个指标），
  以及 Operations inspector 栈。
- 浏览器 console error 收集为空。
- 私有 branch-head inspector 以仅供内存的 token 实际触发。由于本地 API 运行在 public 模式，protected route 未挂载，
  界面以稳定的双语信息 fail closed："Branch-head read failed / 分支 head 读取失败" 与
  "The protected branch-head read failed closed. / 受保护的分支 head 读取已失败关闭。"渲染输出中没有出现上游诊断、凭据或主机名。

This session is local acceptance evidence for the Web/API composition and for fail-closed redaction.
It is **not** authenticated protected-runtime evidence: no request reached a mounted protected Axum
route, because the local API was started in public mode.

本次会话是 Web/API 组合与 fail-closed 脱敏的本地验收证据。它**不是** authenticated protected-runtime 证据：
没有任何请求到达已挂载的 protected Axum route，因为本地 API 以 public 模式启动。

## 6. Publication plan / 发布计划

Publication is gated on the acceptance above. Once accepted:

发布以通过上述验收为前提。验收通过后：

1. ~~Stage every non-ignored file and run `git diff --cached --check` before committing.~~
   Done: 728 files staged, `git diff --cached --check` clean.
2. ~~Create the commit, then create the GitHub repository and push.~~
   Done: published at <https://github.com/XXXXXQ-0206/ContextLab>; the local branch was renamed
   `master` → `main` to match the repository default branch.
3. ~~Watch the first `Verify` run.~~ Done, and it was red twice before it went green. Both failures
   were real product/repository defects, not flakes, and both are fixed:
   - a `#[ignore]`d storage test had gone stale against migration `0017` and failed with a not-null
     violation instead of the cross-Context foreign key it asserted;
   - `actions/upload-artifact@v4` skipped the dot-prefixed evidence directories because
     `include-hidden-files` defaults to `false`, so the evidence upload failed after every check had
     already passed.
4. ~~Record the resulting run URL and artifact names in `docs/roadmap/active-long-term-goal.md`.~~
   Done: run `35181152716` on `main`, conclusion `success`, head commit `0a1c6e9`, with
   `contextlab-ci-evidence` and `contextlab-rehearsal-evidence` retained until 2026-12-16.

1. ~~暂存所有未被 ignore 的文件，并在提交前运行 `git diff --cached --check`。~~
   已完成：728 个文件已暂存，`git diff --cached --check` 干净。
2. ~~创建提交，再创建 GitHub repository 并推送。~~
   已完成：发布在 <https://github.com/XXXXXQ-0206/ContextLab>；本地分支已由 `master` 改名为 `main` 以匹配仓库默认分支。
3. ~~观察首次 `Verify` 运行。~~ 已完成，且经过两次红色才转绿。两次失败都是真实的产品/仓库缺陷，不是抖动，均已修复：
   - 一个 `#[ignore]` 的 storage 测试相对迁移 `0017` 已过时，触发的是 not-null 违规，而不是它所断言的跨 Context 外键；
   - `actions/upload-artifact@v4` 因 `include-hidden-files` 默认为 `false` 而跳过了点开头的证据目录，导致在所有检查都已通过之后，证据上传才失败。
4. ~~把得到的 run URL 与 artifact 名称记入 `docs/roadmap/active-long-term-goal.md`。~~
   已完成：`main` 上的 run `35181152716`，结论 `success`，head commit `0a1c6e9`，
   `contextlab-ci-evidence` 与 `contextlab-rehearsal-evidence` 保留至 2026-12-16。

Remote CI success does not close the long-term goal and does not authorize public protected-write
promotion, release, or production rollout. Those remain separate, deferred decisions with their own
external prerequisites.

远端 CI 成功不关闭长期目标，也不授权 public protected-write promotion、release 或 production rollout。
它们仍是各自带有外部前置的独立延期决策。

### Author identity / 作者身份

Public history carries the repository-local commit identity
`ContextLab Contributors <XXXXXQ-0206@users.noreply.github.com>`, which matches the `authors` field
already declared in `Cargo.toml`. A prior local commit used a personal mail address; it was rewritten
to the same contributor identity before the first push, because nothing had been published yet and
the address would otherwise become permanent public metadata. The pre-rewrite object was preserved
as a local backup ref rather than discarded.

This document deliberately records no personal address. Do not paste one here: a release note that
quotes the identity it is trying to protect becomes the leak it warns about.

公开历史使用 repository-local 提交身份 `ContextLab Contributors <XXXXXQ-0206@users.noreply.github.com>`，
与 `Cargo.toml` 中已声明的 `authors` 字段一致。此前有一个本地提交使用个人邮箱地址；由于当时尚未发布，该提交已在首次 push 前
改写为同一 contributor 身份，否则该地址会成为永久的公开元数据。改写前的对象以本地 backup ref 保留，未被丢弃。

本文档刻意不记录任何个人地址。不要把它粘回来：一份引用它本想保护的身份的发布说明，会变成它所警告的那种泄漏源。
