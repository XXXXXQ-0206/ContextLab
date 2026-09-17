# Private Branch-Head Error Redaction Admission / 私有 Branch-Head 错误脱敏准入

## Necessity Record / 必要性记录

### Named criterion and charter principle / 对应完成条件与宪章原则

This increment directly serves Criterion 4, secure local version and graph reads. The exact
branch-head selection workflow must preserve a typed, fail-closed, bilingual error contract from
the protected local route through the local SDK, Web data adapter, presenter, and screen. The
Context-first security principle forbids exposing upstream diagnostics or private payloads while a
user selects a version for replay or graph review.

本增量直接服务条件 4“安全的本地版本与图读取”。精确 branch-head 选择 workflow 必须从 protected local route 经过 local SDK、Web data adapter、presenter 与 screen，保持类型化、fail-closed、双语错误 contract。Context-first 安全原则禁止用户为 replay 或 graph review 选择版本时暴露 upstream diagnostic 或 private payload。

### Gap, dependencies, and evidence / 缺口、依赖与证据

The protected branch-head route and local SDK already normalize errors, but
`apps/web/src/app/local-branch-heads-data.ts` copies a structured upstream `message` into the
Web proxy error. The shared presenter can render that untrusted message. The route, parser,
presenter, screen, and focused tests already exist, so the smallest dependency-ready repair is a
Web adapter regression and stable local message normalization. No external receipt is required.

受保护 branch-head route 与 local SDK 已经规范化 error，但 `apps/web/src/app/local-branch-heads-data.ts` 仍会把 structured upstream `message` 复制到 Web proxy error，shared presenter 可能渲染该不可信内容。route、parser、presenter、screen 与 focused tests 均已存在，因此最小依赖就绪修复是增加 Web adapter regression 并规范化为稳定本地 message；不依赖任何 external receipt。

### Why now / 为何现在优先

Graph-diff and persisted Context-diff adapters have just closed the same diagnostic-leak class,
and branch-head discovery feeds the exact version selection used by the existing graph-review
bridge. Closing this symmetric boundary is smaller and more directly tied to Criterion 4 than
adding another graph, benchmark, or public API surface.

Graph-diff 与 persisted Context-diff adapter 刚收束同类 diagnostic leak；branch-head discovery 为现有 graph-review bridge 提供 exact version selection。收束这一对称边界比新增 graph、benchmark 或 public API surface 更小，也更直接服务条件 4。

### Explicit non-goals / 明确非目标

- No Rust domain/storage change, route, OpenAPI operation, public SDK method, write, migration,
  provider, operator transport, or second `GraphDiff` calculator.
- No change to branch-head scope validation, capability states, retry behavior, or existing
  bilingual screen copy beyond stable error normalization.
- No secrets, Docker/PostgreSQL runtime, browser, Git, remote CI, operator rehearsal, release, or
  production evidence claim. External release conditions remain deferred and outside the queue.

- 不修改 Rust domain/storage，不新增 route、OpenAPI operation、public SDK method、write、migration、provider、operator transport 或第二个 `GraphDiff` calculator。
- 除稳定 error normalization 外，不改变 branch-head scope validation、capability state、retry behavior 或既有双语 screen copy。
- 不读取 secrets，不声称 Docker/PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release 或 production 证据。外部 release 条件继续延期且不进入当前队列。

### Smallest boundary and ownership / 最小边界与所有权

- Luna worker, if available, owns only `apps/web/src/app/local-branch-heads-data.ts` and
  `apps/web/src/app/local-branch-heads.test.tsx`; it must return changed paths and real test output.
- Integration Lead owns this plan, bilingual roadmap receipts, and final verification. The worker
  must not edit shared presenter/screen or any Rust/API/SDK file.

- Luna worker 若可用，仅负责 `apps/web/src/app/local-branch-heads-data.ts` 与 `apps/web/src/app/local-branch-heads.test.tsx`，并返回实际改动路径与测试输出。
- Integration Lead 负责本计划、双语路线图回执与最终验证。worker 不得编辑 shared presenter/screen 或任何 Rust/API/SDK 文件。

### Fresh verification required / 所需新鲜验证

First observe the red regression where an unknown-status structured upstream message is returned
by the data adapter. Then observe green focused branch-head tests while preserving typed status and
code. Run `pnpm check:web`, the scoped local contract verifier, `GRAPH_DIFF_IMPL_COUNT=1`, Rust
format/workspace/strict offline Clippy/MSRV checks, and record Docker/PostgreSQL, browser, Git, and
external release evidence as `unobserved` or `deferred`.

先观察 unknown-status structured upstream message 被 data adapter 返回的 red regression；再在保持 typed status/code 的前提下观察 focused branch-head tests 变绿。运行 `pnpm check:web`、范围化 local contract verifier、`GRAPH_DIFF_IMPL_COUNT=1`、Rust format/workspace/strict offline Clippy/MSRV checks，并将 Docker/PostgreSQL、browser、Git 与 external release evidence 记录为 `unobserved` 或 `deferred`。

## Execution Checklist / 执行清单

- [x] Observe the red raw-message regression and implement stable bilingual adapter redaction.
- [x] Preserve typed status/code and existing scope, retry, presenter, and screen behavior.
- [x] Run fresh focused and workspace verification, update bilingual roadmap receipts, and keep
  the long-term goal active.

- [x] 观察 raw-message 红回归并实现稳定双语 adapter 脱敏。
- [x] 保持 typed status/code 与既有 scope、retry、presenter、screen behavior。
- [x] 运行新鲜 focused 与 workspace 验证，更新双语路线图回执，并保持长期目标 active。

## Observed Receipt / 已观测回执（2026-08-01）

The first focused run against the old adapter behavior observed `6 passed, 2 failed`: both
unknown-status structured messages crossed the data boundary. The repaired adapter keeps the
upstream error code and HTTP status while replacing the message with the stable bilingual local
message. The green focused run observed `8 passed`. A Luna implementation dispatch stopped after
timeouts without a final report; the Integration Lead reviewed the bounded two-file changes and
completed the red/green verification. This execution provenance is not product evidence.

旧 adapter 行为的首次 focused run 真实观测到 `6 passed, 2 failed`：两个 unknown-status structured message 均穿过 data boundary。修复后的 adapter 保留 upstream error code 与 HTTP status，并将 message 替换为稳定双语本地文案；green focused run 观测到 `8 passed`。一次 Luna implementation dispatch 在超时后停止且没有 final report；Integration Lead 审查其有界两文件改动并完成红绿验证。这是执行 provenance，不是产品证据。

Fresh local verification / 新鲜本地验证：

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `135`, Web `284`, TypeScript/lint, and production build.
- `pwsh -NoProfile -File scripts/verify-local-contracts.ps1 -Root .`: scoped checks passed; `overall=unobserved` without unified diff input.
- `GRAPH_DIFF_IMPL_COUNT=1`: passed.

No Rust/API/SDK/OpenAPI/route/write/migration/provider/secret/public transport change was added.
Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. Criterion 4 and the long-term goal
remain open; the next increment requires a new bilingual Necessity Record.

未新增 Rust/API/SDK/OpenAPI/route/write/migration/provider/secret/public transport 变更。Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。条件 4 与长期目标继续开放；下一增量必须新增双语 Necessity Record。
