# Private Context Diff V1 Read-Scope Hardening / 私有 Context Diff V1 读取范围硬化

## Necessity Record / 必要性记录

### Named criterion / 对应完成条件

- **Criterion 2 / 条件 2:** exact commit-scoped Context history must be replayable and fail closed when any identity dimension or schema drifts.
  / 精确 commit scope 的 Context history 必须可回放，并在任一 identity 维度或 schema 漂移时 fail closed。

### Gap and dependency / 缺口与依赖

The V1 PostgreSQL read currently filters project, Context, and commit but selects the first row by
`ORDER BY schema_version`; it does not explicitly bind the only supported V1 schema. The focused
Memory test also changes all scope identifiers together, so it does not independently prove project,
Context, and commit isolation. Existing typed scope validation, V1 decode, and the PostgreSQL
transaction helper are already available; no new transport or runtime dependency is needed.

当前 PostgreSQL V1 read 会过滤 project、Context 与 commit，但只按 `ORDER BY schema_version` 选择第一行，没有显式绑定唯一支持的 V1 schema。现有
focused Memory test 同时替换所有 scope identifier，未分别证明 project、Context、commit 的隔离。既有 typed scope validation、V1 decode 与 PostgreSQL
transaction helper 已满足依赖；不需要新增 transport 或 runtime 依赖。

### Why now / 为何现在优先

This is the smallest local hardening directly attached to the newly verified persisted writer and
version-backed review path. It removes future-schema ambiguity before any additional Context diff
consumer is admitted, and it closes evidence needed to trust exact commit history without waiting
for unavailable external deployment receipts.

这是刚验证的 persisted writer 与 version-backed review path 直接需要的最小本地硬化。在准入更多 Context diff consumer 前先消除 future-schema 歧义，并补齐
可信 exact commit history 所需证据；不等待当前不可用的外部部署回执。

### Explicit non-goals / 明确非目标

- No schema migration, PostgreSQL runtime execution, public REST/OpenAPI/public SDK method, Web mutation, operator transport, or production claim.
  / 不新增 schema migration、PostgreSQL runtime execution、public REST/OpenAPI/public SDK method、Web mutation、operator transport 或 production 声明。
- No new diff algorithm; `GraphDiff::between` remains the sole graph-diff calculator.
  / 不新增 diff algorithm；`GraphDiff::between` 仍是唯一 graph-diff calculator。
- No change to valid no-op behavior/evaluation semantics or to redacted producer evidence ownership.
  / 不改变合法 no-op behavior/evaluation 语义，也不改变脱敏 producer evidence 的所有权。

### Minimal boundary and bilingual docs / 最小边界与双语文档

Only `crates/storage/src/postgres.rs`, `crates/storage/tests/context_diff_snapshot_repository.rs`,
this plan, the active goal, completion criteria, parallel ledger, and the verification receipt are
affected. The change is a storage-port/query contract hardening with focused tests; API, SDK, Web,
OpenAPI, and public catalogs remain untouched.

仅影响 `crates/storage/src/postgres.rs`、`crates/storage/tests/context_diff_snapshot_repository.rs`、本计划、active goal、completion criteria、parallel ledger
与 verification receipt。这是 storage-port/query contract hardening 与 focused test；API、SDK、Web、OpenAPI 与 public catalog 保持不变。

### Fresh verification required / 下一增量前的新鲜验证

Run the focused Memory scope tests, SQL contract tests, `cargo fmt --all -- --check`, offline
storage and workspace tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`,
the local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL runtime remains explicitly
unobserved while Docker is disabled.

运行 focused Memory scope tests、SQL contract tests、`cargo fmt --all -- --check`、offline storage/workspace tests、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。Docker 关闭时 PostgreSQL runtime 明确保持未观测。

## Execution Checklist / 执行清单

- [x] Add red/green exact-dimension Memory scope coverage.
  / 增加 exact-dimension Memory scope 红绿覆盖。
- [x] Bind the PostgreSQL read query to `CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1`.
  / 将 PostgreSQL read query 显式绑定到 `CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1`。
- [x] Run fresh local verification and update the bilingual receipts.
  / 运行新鲜本地验证并更新双语回执。
