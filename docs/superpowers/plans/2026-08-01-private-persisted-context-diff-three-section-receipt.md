# Private Persisted Context Diff Three-Section Receipt / 私有持久化 Context Diff 三类证据回执

## Necessity Record / 必要性记录

### Named completion criteria / 对应完成条件

- **Criterion 2 / 条件 2:** an exact, replayable Context version history can be inspected by commit scope.
  / 可按精确 commit scope 审阅可回放的 Context 版本历史。
- **Criterion 4 / 条件 4:** semantic, behavior, and evaluation diff remain unified, deterministic, and owned by the Rust diff contract.
  / semantic、behavior 与 evaluation Diff 继续由统一、确定性的 Rust diff contract 负责。

### Gap and dependency / 缺口与依赖

The persisted `ContextDiffSnapshotV1` and private `PersistedContextDiffReviewService` already provide exact source/target scope validation and delegation to the unified diff service. Existing SDK and Web fixtures prove parsing, redaction, metadata transitions, and individual projections, but their cross-layer evidence does not explicitly assert one complete review whose semantic, behavior, and evaluation sections are all non-empty. The missing evidence is test-only and depends on the existing V1 DTOs, local read route, local SDK, same-origin BFF, and `data -> presenter -> screen` composition.

现有 `ContextDiffSnapshotV1` 与私有 `PersistedContextDiffReviewService` 已提供精确 source/target scope 校验，并委托给统一 Diff service。现有 SDK 与 Web fixture 已证明解析、脱敏、metadata transition 和局部 projection，但跨层证据尚未明确断言一个 semantic、behavior、evaluation 三个 section 均非空的完整 review。缺口仅为测试证据，依赖已存在的 V1 DTO、local read route、local SDK、同源 BFF 与 `data -> presenter -> screen` composition。

### Why now / 为何现在优先

This is the smallest dependency-ready evidence increment for the already implemented persisted Context Diff path. It closes a named evidence gap before selecting adjacent benchmark, workflow, knowledge, or plugin work, and prevents a partial projection from being mistaken for complete version-backed review coverage.

这是当前已实现 persisted Context Diff 路径最小且依赖已满足的证据增量。在进入相邻 benchmark、workflow、knowledge 或 plugin 工作前先收束已命名证据缺口，避免把局部 projection 误当成完整的 version-backed review coverage。

### Root cause found during gate / 门禁中发现的根因

The independent storage/API audit found that `CreateContextCommitSnapshot` only carried the graph snapshot. The production guarded commit writers therefore persisted the commit and graph snapshot but did not persist a `ContextDiffSnapshotV1`; the diff table was populated only by test fixtures. Existing API route tests also used empty behavior/evaluation collections. This blocks a claim of a production-backed persisted review even though SDK and Web mock fixtures are green.

独立 storage/API 审计发现 `CreateContextCommitSnapshot` 目前只携带 graph snapshot。因此生产 guarded commit writer 会持久化 commit 与 graph snapshot，却不会持久化 `ContextDiffSnapshotV1`；diff table 只由测试 fixture 填充。既有 API route test 也使用空的 behavior/evaluation collection。虽然 SDK 与 Web mock fixture 已绿，这仍阻断 production-backed persisted review 的声明。

The minimum repair is to derive one valid V1 diff snapshot from the same immutable graph during command construction, use empty behavior/evaluation collections when no evaluation evidence exists, and persist that record in the existing memory/PostgreSQL commit transaction. Non-empty behavior/evaluation evidence remains owned by benchmark/evaluation producers and is covered separately by redacted read fixtures; this repair does not invent evaluation results.

最小修复是：在 command 构造时从同一个不可变 graph 派生一个合法 V1 diff snapshot；没有评测证据时使用空的 behavior/evaluation collection；在既有 memory/PostgreSQL commit transaction 中持久化该 record。非空 behavior/evaluation evidence 仍由 benchmark/evaluation producer 负责，并通过脱敏 read fixture 单独覆盖；本修复不伪造评测结果。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK write or new public read operation.
  / 不新增 public REST/OpenAPI/public SDK write 或新的 public read operation。
- No Web mutation, operator transport, provider call, migration, Docker/PostgreSQL runtime, authenticated browser, release, production, or external receipt.
  / 不新增 Web mutation、operator transport、provider call、migration、Docker/PostgreSQL runtime、authenticated browser、release、production 或 external receipt。
- No second semantic/behavior/evaluation engine and no second `GraphDiff` calculator; Web and SDK remain adapters/presenters.
  / 不新增第二个 semantic/behavior/evaluation engine 或第二个 `GraphDiff` calculator；Web 与 SDK 继续只做 adapter/presenter。

### Minimal affected boundary and bilingual docs / 最小影响边界与双语文档

The implementation boundary now includes the existing Rust commit snapshot command and its Memory/PostgreSQL writer adapters because the red audit exposed a production contract defect: commit writers were not persisting the derived diff input. The test boundary includes the storage atomic/replay regressions, API exact-scope fixtures, local SDK parser, Web BFF, and `data -> presenter -> screen` tests. No public route, OpenAPI/public SDK method, Web mutation, or second diff calculator is admitted. The plan, `active-long-term-goal.md`, `completion-criteria.md`, `parallel-development-plan.md`, and a verification receipt record the exact evidence and its boundary in English and Chinese.

代码边界仅限现有 local SDK persisted-diff test、现有 Web persisted-diff data/presenter/screen tests，以及其已有的双语验证记录。Rust storage/API production contract 只作为只读审计范围，除非红测暴露真实 contract 缺陷。计划、`active-long-term-goal.md`、`completion-criteria.md`、`parallel-development-plan.md` 与 verification receipt 必须以中英双语记录精确证据及边界。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

1. Focused local SDK persisted-diff tests prove the same exact review has non-empty semantic, behavior, and evaluation sections and remains fail-closed.
   / local SDK 聚焦测试证明同一 exact review 的 semantic、behavior、evaluation section 均非空且继续 fail-closed。
2. Focused Web data/presenter/screen tests prove the redacted three-section payload crosses all layers without recomputation and preserves bilingual/accessibility states.
   / Web data/presenter/screen 聚焦测试证明脱敏三 section payload 穿过全部层级、不重算，并保持双语/可访问状态。
3. Existing Rust persisted snapshot/review tests and API route tests remain green; then run format, offline workspace tests, strict offline Clippy, locked Rust 1.85 check, `pnpm check:web`, local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`.
   / 既有 Rust persisted snapshot/review tests 与 API route tests 继续通过；随后运行 format、offline workspace tests、strict offline Clippy、锁定 Rust 1.85 check、`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。
4. Docker/PostgreSQL, browser, Git, remote CI, operator, release, and production evidence remain explicitly `unobserved` or `deferred` and cannot be inferred from local tests.
   / Docker/PostgreSQL、browser、Git、remote CI、operator、release 与 production evidence 必须明确保持 `unobserved` 或 `deferred`，不得从本地测试推断。

## Execution Checklist / 执行清单

- [x] Confirm the existing SDK fixture or add the smallest missing three-section assertion.
  / 确认现有 SDK fixture，或补最小三 section assertion。
- [x] Confirm the existing Web fixture or add the smallest data-to-presenter-to-screen assertion.
  / 确认现有 Web fixture，或补最小 data-to-presenter-to-screen assertion。
- [x] Persist the command-derived V1 snapshot through the existing guarded commit writers and add memory/API regression evidence.
  / 通过既有 guarded commit writer 持久化 command-derived V1 snapshot，并补 memory/API 回归证据。
- [x] Run focused and fresh repository-wide verification; record observed output only.
  / 运行聚焦与新鲜仓库级验证，只记录实际观察到的输出。
- [x] Reconcile bilingual roadmap and completion audit; keep the long-term goal active.

## Observed Receipt / 已观测回执

The production writer defect is repaired within the existing commit transaction boundary:
`CreateContextCommitSnapshot` derives one validated `ContextDiffSnapshotV1` from the same immutable
graph; Memory and PostgreSQL normal and guarded writers persist it with the commit and graph
snapshot; Memory exact-scope reads and idempotent guarded replay return the same immutable record.
Behavior and evaluation collections remain empty when no producer evidence exists; the API fixtures
use explicit redacted producer facts to prove non-empty three-section projection without inventing
runtime evaluation results.

生产 writer 缺陷已在既有 commit transaction 边界内修复：`CreateContextCommitSnapshot` 从同一不可变 graph 派生一个经过校验的
`ContextDiffSnapshotV1`；Memory 与 PostgreSQL 的普通和 guarded writer 将其与 commit、graph snapshot 一起持久化；Memory exact-scope read
与幂等 guarded replay 返回同一不可变 record。没有 producer evidence 时 behavior 与 evaluation collection 继续为空；API fixture 使用明确的
脱敏 producer facts 证明三段 projection 非空，不虚构运行时评测结果。

Fresh local evidence / 新鲜本地证据:

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; API `214 passed`, storage `212 passed, 39 ignored`.
- Focused storage guarded replay: `1 passed`; storage persisted review: `8 passed`; API direct review: `1 passed`; API protected review: `3 passed`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production build.
- `scripts/verify-local-contracts.ps1`: scoped checks passed, including `graph_diff_application=passed count=1`; `overall=unobserved` because no unified diff input was supplied.
- `GRAPH_DIFF_IMPL_COUNT=1`: passed.

`GraphDiff::between` remains the sole graph-diff calculator. No public REST/OpenAPI/public SDK
write, operator transport, Web mutation, provider, secret access, migration, or release claim was
added. PostgreSQL/Docker runtime, authenticated browser, visual smoke, Git change-set, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`. This closes only the
bounded local evidence slice; the long-term goal remains active and the next increment needs a new
bilingual Necessity Record.

`GraphDiff::between` 仍是唯一 graph-diff calculator。没有新增 public REST/OpenAPI/public SDK write、operator transport、Web mutation、provider、
secret access、migration 或 release 声明。PostgreSQL/Docker runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`。本回执只收束有界本地证据切片；长期目标保持 active，下一增量必须新增双语 Necessity Record。
  / 收束双语路线图与完成度审计；保持长期目标 active。
