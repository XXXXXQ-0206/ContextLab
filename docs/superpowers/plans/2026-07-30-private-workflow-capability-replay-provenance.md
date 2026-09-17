# Private Workflow Capability Replay Provenance / 私有 Workflow Capability Replay Provenance

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Criteria 1, 2, and 4: a workflow execution must be
replayable from an immutable Context/version boundary, and replay must prove the
capability snapshot used for execution rather than accepting a merely compatible
registry state.

**Unmet dependency or evidence gap / 未满足依赖或证据缺口：** Workflow binding
currently validates an exact capability snapshot before execution, but the execution
log persists only the binding and caller-supplied replay context. A different
compatible snapshot can therefore be injected during replay without the log detecting
the substitution. Snapshot provenance is an in-memory fact, not a versioned replay
contract.

**Why now / 为什么现在优先：** The deterministic provider-free capability bridge,
execution state machine, and replay log already exist and have focused tests. Sealing
the snapshot representation/digest into that log is the smallest direct fix for the
replay criterion and is independent of benchmark work, so both can advance in parallel.

**Non-goals / 明确非目标：** No provider calls, registry mutation, workflow scheduler,
new transport, REST/OpenAPI/SDK/Web write, public API, storage migration, secret access,
Docker/PostgreSQL runtime claim, operator transport, release, production claim, or
second diff calculator. Existing scope/error precedence must remain unchanged.

**Smallest boundary and bilingual documentation / 最小边界与双语文档：** Add a
versioned canonical capability snapshot representation and bind its exact identity to
`WorkflowExecutionLogV1` in `crates/workflow`; validate it on log restore/replay and
cover the contract with deterministic tests. Update this plan and the bilingual
parallel/active/completion records after verification; keep API, SDK, Web, and plugin
registry ownership untouched.

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Focused
workflow replay/provenance tests, existing MCP/plugin contract tests, `cargo fmt`,
workspace Rust, strict offline Clippy, locked Rust 1.85, `pnpm check:web`, and the
singular `GraphDiff` static check. PostgreSQL/Docker runtime, authenticated browser,
Git, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`.

## Tasks / 任务

- [x] Add a versioned canonical capability snapshot serialization/digest boundary.
- [x] Persist and validate provenance in execution log restore/replay.
- [x] Add deterministic ordering, schema drift, substitution, missing capability, and exact replay tests.
- [x] Record fresh local evidence and update bilingual roadmap/audit documents.

## Fresh Verification Record / 新鲜验证记录

The focused deterministic replay contract passed `15` tests. The full workflow package,
workspace Rust, `cargo fmt --all -- --check`, strict offline workspace Clippy, locked Rust
`1.85.0` check, and `pnpm check:web` (public SDK `15`, local SDK `100`, Web `207`,
TypeScript/lint, and production build) passed. Static inspection still reports
`GRAPH_DIFF_IMPL_COUNT=1`.

聚焦 deterministic replay contract 通过 `15` 项测试。workflow package 全量、workspace Rust、
`cargo fmt --all -- --check`、strict offline workspace Clippy、锁定 Rust `1.85.0` check 与
`pnpm check:web`（public SDK `15`、local SDK `100`、Web `207`、TypeScript/lint 与 production build）
均通过。静态检查仍为 `GRAPH_DIFF_IMPL_COUNT=1`。

No API, SDK, Web, registry mutation, provider, storage migration, public write, secret,
Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator, release, or
production evidence was added. Those boundaries remain `unobserved` or `deferred`.

没有新增 API、SDK、Web、registry mutation、provider、storage migration、public write、secret、
Docker/PostgreSQL runtime、authenticated browser、Git、remote CI、operator、release 或 production
evidence；这些边界继续为 `unobserved` 或 `deferred`。
