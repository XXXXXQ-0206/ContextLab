# Private Versioned Context Diff Boundary Witness / 私有版本化 Context Diff 边界见证

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与宪章原则

- **Criterion 2 / 条件 2:** exact Context commit history must be replayable and reviewable through a stable version-backed diff contract.
  / 精确 Context commit history 必须能通过稳定的版本化 diff 契约回放与审阅。
- **Criterion 4 / 条件 4:** Context Graph remains the system backbone and `GraphDiff::between` remains the sole graph-diff calculator.
  / 条件 4：Context Graph 继续作为系统骨架，且 `GraphDiff::between` 继续是唯一 graph-diff calculator。
- **Charter / 宪章:** reusable Rust domain contracts own policy; API and SDK are typed adapters, and Web consumes `data -> presenter -> screen` without recalculating diff semantics.
  / 可复用 Rust domain contract 负责策略；API 与 SDK 只是 typed adapter，Web 遵循 `data -> presenter -> screen`，不重新计算 diff 语义。

### Gap and ready dependencies / 缺口与已满足依赖

Storage tests and the in-module API test already prove persisted V1 snapshots and non-empty semantic, behavior, and evaluation sections. Before this increment, the independent API contract test proved only retirement of the old public route; it did not independently witness the protected version-backed graph-diff response. The SDK parser implementation already supported all variants, but its focused matrix lacked `added` and `removed` coverage for the three consumer sections. Existing protected routing, repository fixtures, SDK parser unions, and Web screen integration are already available; no runtime, database, or external receipt is required.

Storage test 与 API 模块内测试已经证明 persisted V1 snapshot 以及 semantic、behavior、evaluation 三类非空 section。本增量前，独立 API contract test 只证明旧 public route 已退役，尚未独立见证 protected version-backed graph-diff response。SDK parser implementation 已支持所有 variant，但 focused matrix 尚未覆盖三类 consumer section 的 `added`、`removed`。既有 protected routing、repository fixture、SDK parser union 与 Web screen integration 已满足依赖；不需要 runtime、database 或外部回执。

### Why now / 为何现在优先

This is the smallest dependency-ready evidence increment attached directly to the already implemented commit-associated snapshot and version-backed review path. It converts internal/fixture evidence into an independently reviewable API boundary and closes parser-shape evidence before another consumer or feature is admitted. It is therefore earlier and narrower than adding new Context editing, benchmark behavior, or public write capability.

这是直接依附于已实现的 commit-associated snapshot 与 version-backed review path 的最小依赖就绪证据增量。它把内部/fixture 证据提升为可独立审阅的 API boundary evidence，并在准入新的 consumer 或功能前补齐 parser shape 证据。因此它比新增 Context 编辑、benchmark behavior 或 public write 更优先且更窄。

### Explicit non-goals / 明确非目标

- No new route, public REST/OpenAPI/public SDK write method, Web mutation, operator transport, migration, provider call, secret access, Docker/PostgreSQL runtime, browser E2E, release, or production claim.
  / 不新增 route、public REST/OpenAPI/public SDK write method、Web mutation、operator transport、migration、provider call、secret access、Docker/PostgreSQL runtime、browser E2E、release 或 production 声明。
- No second diff implementation. `GraphDiff::between` remains the sole graph-diff calculator; API, SDK, and Web only adapt Rust-owned output.
  / 不新增第二套 diff 实现。`GraphDiff::between` 仍是唯一 graph-diff calculator；API、SDK 与 Web 只适配 Rust 所有的输出。
- No claim that local evidence substitutes for deferred remote CI, operator rehearsal, or production deployment evidence.
  / 不声称本地证据可以替代延期的 remote CI、operator rehearsal 或 production deployment evidence。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

The implementation boundary is limited to `server/api/tests/commit_graph_snapshot_scope_contract.rs` and `packages/local-sdk/src/persisted-context-diff-review.test.ts`, plus this plan and a fresh bilingual verification receipt. Production routes, storage, API DTOs, SDK implementation, Web screens, OpenAPI, and public catalogs remain unchanged.

实现边界仅限 `server/api/tests/commit_graph_snapshot_scope_contract.rs` 与 `packages/local-sdk/src/persisted-context-diff-review.test.ts`，以及本计划和一份新的双语验证回执。production route、storage、API DTO、SDK implementation、Web screen、OpenAPI 与 public catalog 保持不变。

### Fresh verification required / 下一增量前的新鲜验证

Observe the focused protected API contract test and local SDK parser test, then run `cargo fmt --all -- --check`, offline workspace tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, the local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL-backed runtime, browser/visual E2E, Git change-set, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or `deferred`.

先观察 focused protected API contract test 与 local SDK parser test，再运行 `cargo fmt --all -- --check`、offline workspace tests、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL-backed runtime、browser/visual E2E、Git change-set、remote CI、operator rehearsal、release 与 production evidence 继续保持 `unobserved` 或 `deferred`。

## Execution Checklist / 执行清单

- [x] Add the independent protected-router witness for exact scope, stable non-empty graph diff, authentication, and same-commit rejection.
  / 增加独立 protected-router witness，验证精确 scope、稳定非空 graph diff、authentication 与 same-commit rejection。
- [x] Add SDK parser matrix coverage for semantic, behavior, and evaluation `added` and `removed` values with fail-closed extra-field checks.
  / 增加 SDK parser 对 semantic、behavior、evaluation 的 `added` 与 `removed` value 及 fail-closed extra-field check 的矩阵覆盖。
- [x] Run fresh cross-stack verification and record observed states without external overclaim.
  / 运行新鲜跨栈验证并记录真实状态，不夸大外部证据。
