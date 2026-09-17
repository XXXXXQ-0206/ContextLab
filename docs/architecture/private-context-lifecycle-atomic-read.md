# Private Context Lifecycle Atomic Read / 私有 Context 生命周期原子读取

## Boundary / 边界

The private lifecycle read is a storage contract, not a new transport. The application asks for
`(ContextId, CommitId)`; the repository derives the owning `ProjectId` and returns one
`ContextLifecycleReadFacts` aggregate. The aggregate contains the exact graph snapshot scope,
replayed version state, component inventory, and immutable body witnesses. The application service
checks the returned Context and commit against the request, then performs the existing witness and
replay-to-graph validation before adapting the unchanged local response DTO.

私有 lifecycle read 是 storage contract，不是新的 transport。application 以 `(ContextId, CommitId)` 请求；
repository 推导所属 `ProjectId`，返回一个 `ContextLifecycleReadFacts` aggregate。aggregate 包含 exact graph
snapshot scope、replayed version state、component inventory 与 immutable body witness。application service
先校验返回的 Context 与 commit 是否等于请求，再执行既有 witness 与 replay-to-graph 校验，最后适配未改变的
local response DTO。

## Backend Guarantees / 后端保证

- Memory holds one `RwLock` read guard while resolving ancestry, graph, component revisions, and replay state.
- PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction for the exact scope, normal-parent history, component revision rows, and replay rows.
- `ContextLifecycleReadFacts::from_parts` rejects graph, inventory, replay, content-Context, and service-request scope drift.
- `GraphDiff::between` remains the only graph-diff calculator; this read path never computes a diff.

- Memory 在解析 ancestry、graph、component revision 与 replay state 时持有同一个 `RwLock` read guard。
- PostgreSQL 对 exact scope、normal-parent history、component revision rows 与 replay rows 使用同一个
  `REPEATABLE READ READ ONLY` transaction。
- `ContextLifecycleReadFacts::from_parts` 拒绝 graph、inventory、replay、content-Context 与 service-request 的 scope drift。
- `GraphDiff::between` 仍是唯一 graph-diff calculator；此 read path 不计算 diff。

## Compatibility And Non-Goals / 兼容性与非目标

The API, local SDK, BFF, Web data/presenter/screen chain, and public OpenAPI surface keep their
existing shapes. This increment adds no route, write, mutation, migration, provider, scheduler,
operator transport, raw-content expansion, or production readiness claim. A disposable PostgreSQL
runtime receipt is still required before claiming observed runtime behavior; absent that receipt,
the local transaction implementation is only compile/static evidence.

API、local SDK、BFF、Web data/presenter/screen chain 与 public OpenAPI surface 保持既有形状。本增量不新增
route、write、mutation、migration、provider、scheduler、operator transport、raw-content expansion 或
production readiness 声明。在声称 runtime behavior 已观测前仍需 disposable PostgreSQL runtime receipt；
缺少该回执时，local transaction implementation 仅有 compile/static evidence。

## Verification / 验证

The fresh local receipt covers storage lifecycle tests (`15 passed`), PostgreSQL SQL contract (`1
passed`), API lifecycle tests (`5 passed`), workspace Rust (`219 passed, 39 ignored`), format,
strict offline Clippy, Rust `1.85.0` locked check, `pnpm check:web` (`15/135/284` plus production
build), scoped local contract checks, and `GRAPH_DIFF_IMPL_COUNT=1`. Runtime PostgreSQL, browser,
Git, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or
deferred.

新鲜本地回执覆盖 storage lifecycle（`15 passed`）、PostgreSQL SQL contract（`1 passed`）、API lifecycle
（`5 passed`）、workspace Rust（`219 passed, 39 ignored`）、format、strict offline Clippy、Rust `1.85.0`
locked check、`pnpm check:web`（`15/135/284` 加 production build）、范围化 local contract check 与
`GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release 与
production evidence 继续为 `unobserved` 或 `deferred`。
