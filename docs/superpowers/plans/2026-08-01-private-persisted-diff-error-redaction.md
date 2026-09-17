# Private Persisted Context Diff Error Redaction / 私有持久化 Context Diff 错误脱敏

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与章程原则

- **Criterion 4 / 条件 4:** The private version-backed Context diff read must keep the same
  fail-closed, redacted error boundary across API, local SDK, Web data adapter, presenter, and
  screen.
  / **条件 4：** 私有 version-backed Context diff read 必须在 API、local SDK、Web data adapter、presenter 与 screen 之间保持一致的 fail-closed、脱敏错误边界。
- **Security principle / 安全原则:** Upstream diagnostics and private payloads must not enter
  client-visible error objects.
  / **安全原则：** upstream diagnostics 与 private payload 不得进入 client-visible error object。

### Gap and dependency / 缺口与依赖

The adjacent `local-commit-graph-diff-data.ts` boundary is now redacted, but
`local-persisted-context-diff-review-data.ts` still preserves a structured upstream `message` for
the private persisted semantic/behavior/evaluation diff review. Its route, parser, exact scope
checks, and presenter already exist; only this adapter error normalization is missing.

相邻的 `local-commit-graph-diff-data.ts` boundary 已完成脱敏，但 `local-persisted-context-diff-review-data.ts` 仍会为私有 persisted semantic/behavior/evaluation diff review 保留 structured upstream `message`。
其 route、parser、exact scope check 与 presenter 均已存在；只缺少 adapter error normalization。

### Why now / 为何现在优先

This is the remaining symmetric error-boundary defect in the version-backed graph/diff read path.
Closing it now prevents a known security regression from surviving beside the newly hardened graph
diff adapter, without adding public transport or duplicating any diff calculation.

这是 version-backed graph/diff read path 中剩余的对称错误边界缺陷。现在关闭它，可避免已知安全回归与刚完成硬化的 graph diff adapter 并存，同时不新增 public transport 或重复任何 diff calculation。

### Explicit non-goals / 明确非目标

- No route, OpenAPI operation, public SDK method, write, migration, provider, or GraphDiff change.
  / 不新增 route、OpenAPI operation、public SDK method、write、migration、provider 或 GraphDiff 改动。
- No change to exact scope validation, retry parsing, known status presentation, or response DTOs.
  / 不改变 exact scope validation、retry parsing、known status presentation 或 response DTO。
- No secret access, Docker/PostgreSQL runtime, browser, Git, remote CI, release, or production
  evidence claim.
  / 不读取 secret，不声称 Docker/PostgreSQL runtime、browser、Git、remote CI、release 或 production 证据。

### Smallest boundary and fresh verification / 最小边界与新鲜验证

Integration Lead owns only `apps/web/src/app/local-persisted-context-diff-review-data.ts`, its
focused test, this plan, and the bilingual receipts. First observe a red unknown-status raw-message
regression; then run the focused green test, `pnpm check:web`, local contract verifier,
`GRAPH_DIFF_IMPL_COUNT=1`, and the Rust gates. Keep the long-term goal active.

Integration Lead 仅负责上述 adapter、focused test、本计划与双语回执。先观测 unknown-status raw-message 红回归，再运行 focused green test、`pnpm check:web`、local contract verifier、
`GRAPH_DIFF_IMPL_COUNT=1` 与 Rust gates。保持长期目标 active。

## Execution Checklist / 执行清单

- [x] Observe the raw-message red regression and apply stable adapter redaction.
  / 观测 raw-message 红回归并应用稳定 adapter 脱敏。
- [x] Preserve structured error code, status, retry timing, scope checks, and known status behavior.
  / 保持 structured error code、status、retry timing、scope check 与 known status behavior。
- [x] Run fresh full verification and update bilingual receipts without closing the goal.
  / 运行新鲜全量验证并更新双语回执，不关闭长期目标。

## Observed Result / 已观测结果

The red regression exposed the synthetic upstream diagnostic through
`LocalPersistedContextDiffReviewProxyError`. The adapter now preserves only the structured error
code and HTTP status while replacing structured or malformed messages with the existing stable
bilingual unavailable message. Scope, retry, response parsing, and presenter behavior are unchanged.

红回归证明 synthetic upstream diagnostic 会通过 `LocalPersistedContextDiffReviewProxyError` 暴露。adapter 现只保留 structured error code 与 HTTP status，
并将 structured 或 malformed message 替换为既有稳定双语 unavailable message。Scope、retry、response parsing 与 presenter behavior 均未改变。

Fresh evidence / 新鲜证据：

- Red focused run: `4 passed, 2 failed`, with the raw `sql://internal-db?token=secret` diagnostic visible.
- Green focused run: `6 passed`; Web TypeScript check passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `135`, Web `278`, TypeScript/lint and production build.
- `cargo fmt --all -- --check`, workspace tests (storage `212 passed, 39 ignored`), strict offline Clippy, and locked Rust `1.85.0` check passed.
- Scoped local contract verifier and `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` without unified diff input.

新鲜证据：

- 红 focused run：`4 passed, 2 failed`，且失败输出可见 raw `sql://internal-db?token=secret` diagnostic。
- 绿 focused run：`6 passed`；Web TypeScript check 通过。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `278`，TypeScript/lint 与 production build 均通过。
- `cargo fmt --all -- --check`、workspace tests（storage `212 passed, 39 ignored`）、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- scoped local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过；因未提供 unified diff input，verifier `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
operator transport, or second GraphDiff calculator was added. Docker/PostgreSQL runtime,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain `unobserved` or `deferred`. The goal remains active; the next implementation requires a new
bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、operator transport 或第二个 GraphDiff calculator。
Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。
目标保持 active；下一项实现必须先新增双语 Necessity Record。
