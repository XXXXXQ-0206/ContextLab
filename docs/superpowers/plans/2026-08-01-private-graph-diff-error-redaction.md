# Private Graph Diff Error Redaction / 私有 Graph Diff 错误脱敏

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与章程原则

- **Criterion 4 / 条件 4:** Cross-layer graph comparison must preserve a fail-closed, redacted
  contract across protected route, local SDK, Web data adapter, presenter, and screen.
  / **条件 4：** 跨层 graph comparison 必须在 protected route、local SDK、Web data adapter、presenter 与 screen 之间保持 fail-closed、脱敏 contract。
- **Security principle / 安全原则:** Do not expose upstream diagnostics, private payload, or
  credentials through a local UI error object.
  / **安全原则：** 不得通过 local UI error object 暴露 upstream diagnostics、private payload 或 credential。

### Gap, dependency, and evidence / 缺口、依赖与证据

The existing `local-commit-graph-diff-data.ts` preserves any structured upstream `message` in its
proxy error body. The current screen maps known statuses to stable bilingual text, but callers and
future consumers can still inspect the raw message for an unknown status. The graph diff route and
parser already exist, so this is a dependency-ready adapter hardening slice with no new transport.

现有 `local-commit-graph-diff-data.ts` 会在 proxy error body 中保留任意 structured upstream `message`。当前 screen 对已知 status 会映射稳定双语文案，
但 caller 与未来 consumer 仍可能从未知 status 读取 raw message。graph diff route 与 parser 已存在，因此这是不需要新 transport 的依赖就绪 adapter hardening slice。

### Why now / 为何现在优先

The exact-commit relationship receipt just established the Web read boundary. Hardening the adjacent
version-backed graph comparison adapter now closes a concrete redaction gap before expanding any new
graph, workflow, or benchmark surface. It is smaller and more directly tied to the named security
criterion than new UI or public API work.

刚完成的 exact-commit relationship receipt 已建立 Web read boundary。现在收紧相邻的 version-backed graph comparison adapter，可在扩展新的 graph、workflow 或 benchmark surface 前关闭明确的脱敏缺口。
它比新增 UI 或 public API 更小，也更直接服务命名的安全条件。

### Explicit non-goals / 明确非目标

- No Rust domain change, route, OpenAPI operation, public SDK method, write, migration, provider,
  or second `GraphDiff` calculator.
  / 不修改 Rust domain，不新增 route、OpenAPI operation、public SDK method、write、migration、provider 或第二个 `GraphDiff` calculator。
- No change to known status-specific bilingual screen copy beyond the adapter's stable redaction.
  / 除 adapter 的稳定脱敏外，不改变已知 status 的双语 screen copy。
- No secrets, external runtime, Docker/PostgreSQL, browser, Git, remote CI, release, or production
  evidence claim.
  / 不读取 secret，不声称 external runtime、Docker/PostgreSQL、browser、Git、remote CI、release 或 production 证据。

### Smallest boundary and ownership / 最小边界与所有权

- Luna owns only `apps/web/src/app/local-commit-graph-diff-data.ts` and
  `apps/web/src/app/local-commit-graph-diff-data.test.ts`.
  / Luna 仅负责上述两个 adapter/test 文件。
- Integration Lead owns this plan, bilingual roadmap/verification receipt, and final checks.
  / Integration Lead 负责本计划、双语 roadmap/verification receipt 与最终检查。

### Fresh verification required / 所需新鲜验证

Observe a red regression where an unknown-status structured upstream message is not returned to the
caller, then make it green while preserving the typed status/code boundary. Run focused adapter and
Web tests, `pnpm check:web`, the local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`; Rust gates
remain a fresh regression check because no Rust files change.

先观测 unknown-status structured upstream message 会被返回给 caller 的红回归，再在保持 typed status/code boundary 的同时修复为绿。运行 focused adapter 与 Web tests、
`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`；虽然不改 Rust 文件，仍重新检查 Rust gates。

## Execution Checklist / 执行清单

- [x] Observe the red raw-message regression and implement stable adapter redaction.
  / 观测 raw-message 红回归并实现稳定 adapter 脱敏。
- [x] Preserve structured status/code and existing known-status presentation behavior.
  / 保持 structured status/code 与既有 known-status presentation 行为。
- [x] Run fresh focused and workspace verification, update bilingual receipts, and keep the goal active.
  / 运行新鲜聚焦与全量验证，更新双语回执，并保持长期目标 active。

## Observed Result / 已观测结果

The red regression reproduced an unknown-status proxy error exposing
`sql://internal-db?token=secret diagnostic payload`. The adapter now preserves the structured
upstream error code and HTTP status while replacing all structured or malformed error messages with
the stable bilingual local graph review message. No transport, parser, or presenter algorithm was
changed.

红回归复现了 unknown-status proxy error 暴露 `sql://internal-db?token=secret diagnostic payload` 的问题。adapter 现保留 structured upstream error code 与 HTTP status，
但将 structured 或 malformed error message 统一替换为稳定的双语 local graph review message。没有修改 transport、parser 或 presenter algorithm。

Fresh evidence / 新鲜证据：

- Red focused run: `2 passed, 1 failed`; the failure output contained the upstream internal-db/token message.
- Green focused adapter run: `3 passed`; Web TypeScript check passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `135`, Web `277`, TypeScript/lint and production build.
- `cargo fmt --all -- --check`, offline workspace tests (storage `212 passed, 39 ignored`), strict offline Clippy, and locked Rust `1.85.0` check passed.
- `scripts/verify-local-contracts.ps1` scoped checks passed; `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` without unified diff input.

新鲜证据：

- 红 focused run：`2 passed, 1 failed`；失败输出包含 upstream internal-db/token message。
- 绿 focused adapter run：`3 passed`；Web TypeScript check 通过。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `277`，TypeScript/lint 与 production build 均通过。
- `cargo fmt --all -- --check`、offline workspace tests（storage `212 passed, 39 ignored`）、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- `scripts/verify-local-contracts.ps1` 的 scoped checks 通过；`GRAPH_DIFF_IMPL_COUNT=1` 通过；因未提供 unified diff input，verifier `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
operator transport, or second GraphDiff calculator was added. Docker/PostgreSQL runtime,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain `unobserved` or `deferred`. The goal remains active; the next implementation needs a new
bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、operator transport 或第二个 GraphDiff calculator。
Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。
目标保持 active；下一项实现必须先新增双语 Necessity Record。
