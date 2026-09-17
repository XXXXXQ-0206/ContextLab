# Private Guarded Lifecycle to GraphDiff Integration / 私有 Guarded Lifecycle 到 GraphDiff 集成

## Necessity Record / 必要性记录

**Named completion criteria and charter principles / 命名完成条件与章程原则:** This increment
directly serves Criterion 2 (replayable version history and reusable versioned diff workflows)
and Criterion 4 (one Context Graph contract across storage and diff review). The Context-first
charter requires a persisted lifecycle change to remain replayable and reviewable through the
same graph state rather than through test-only synthetic facts.

本增量直接服务条件 2（可回放版本历史与可复用的版本 Diff workflow）和条件 4（storage 与 diff review 共用同一套
Context Graph contract）。Context-first 章程要求持久化 lifecycle change 必须通过同一份 graph state 可回放、可审阅，
而不是只通过测试合成 facts 证明。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The lifecycle-witness
GraphDiff gate is now implemented and its focused tests pass, while guarded lifecycle writer tests
and graph-diff review tests are separate. No local integration test yet proves that a real guarded
create/update/removal or relationship change produces exact lifecycle facts and graph snapshots
that the review service can compare at the requested commit pair. Without this proof, Criteria 2
and 4 remain locally composable but not end-to-end evidenced.

生命周期见证 GraphDiff gate 已实现且 focused test 通过，但 guarded lifecycle writer test 与 graph-diff review test 仍然分离。
当前没有本地集成测试证明真实 guarded create/update/removal 或 relationship change 会产生可供 review service 按请求 commit pair
比较的 exact lifecycle facts 与 graph snapshot。缺少该证明时，条件 2 与 4 只有可组合的局部证据，尚无端到端证据。

**Why now / 为什么现在优先:** The exact lifecycle witness boundary is the latest verified prerequisite;
the smallest next step is to connect it to the already-existing writer and review contracts before
adding more branch, merge, benchmark, or UI consumers. This is a test-only convergence increment,
so it reduces a named evidence gap without widening any product surface.

exact lifecycle witness boundary 是最近刚验证的前置；在增加更多 branch、merge、benchmark 或 UI consumer 前，最小下一步是把
它接到已经存在的 writer 与 review contract。这是 test-only convergence 增量，只收束一个命名证据缺口，不扩大产品 surface。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Add one
storage integration test at
`crates/storage/tests/context_graph_diff_review_lifecycle_integration.rs`. It may touch
`crates/storage/src/context_graph_diff_review.rs` only if the test exposes a real composition defect.
Record the result in `ARCHITECTURE.md`, `docs/roadmap/active-long-term-goal.md`,
`docs/roadmap/completion-criteria.md`, and `docs/roadmap/parallel-development-plan.md` after
fresh verification.

新增一个 storage integration test：
`crates/storage/tests/context_graph_diff_review_lifecycle_integration.rs`。只有测试暴露真实 composition defect 时，才可
最小修改 `crates/storage/src/context_graph_diff_review.rs`。新鲜验证后在 `ARCHITECTURE.md`、
`docs/roadmap/active-long-term-goal.md`、`docs/roadmap/completion-criteria.md` 与
`docs/roadmap/parallel-development-plan.md` 记录结果。

**Explicit non-goals / 明确非目标:** No public REST/OpenAPI/SDK write, new route, Web mutation,
operator transport, migration, provider call, raw private content, secret access, Docker or
PostgreSQL runtime claim, external release evidence, or second graph-diff calculator. The test
must continue to use the existing guarded writer, lifecycle repository, and `GraphDiff::between`
delegation.

不新增 public REST/OpenAPI/SDK write、新 route、Web mutation、operator transport、migration、provider call、raw private content、
secret access、Docker 或 PostgreSQL runtime 声明、external release evidence，也不新增第二个 graph-diff calculator。测试必须继续
复用既有 guarded writer、lifecycle repository 与 `GraphDiff::between` delegation。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** First observe a focused
red failure or a contract-level missing composition if the integration cannot yet be wired, then
obtain green integration and focused review tests; `cargo fmt --all -- --check`; offline workspace
tests; strict offline Clippy; locked Rust `1.85.0` check; `pnpm check:web`; the local contract
verifier; and a source count proving one `GraphDiff` implementation. PostgreSQL runtime, Docker,
browser/visual, Git, remote CI, operator, release, and production remain `ignored`, `unobserved`,
or `deferred` unless independently observed.

先观测 focused red failure 或 contract-level missing composition（若 integration 尚不能接通），再取得 integration 与 focused
review test 的 green；运行 `cargo fmt --all -- --check`、offline workspace test、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`、local contract verifier 与唯一 `GraphDiff` implementation source count。PostgreSQL runtime、Docker、
browser/visual、Git、remote CI、operator、release 与 production 除非独立观测，否则继续标记为 `ignored`、`unobserved` 或 `deferred`。

## Implementation Receipt / 实现回执

- [x] Red/green integration proof using the existing guarded lifecycle writer and exact commit pair.
  The storage integration test reports `3 passed` and covers initialize/create/update, a second
  component, Uses add/remove, and missing or mismatched exact scope failure.
- [x] Review service consumes the writer-produced lifecycle facts and graph snapshots without a
  second calculator. The successful path asserts revision provenance, replay commit scope, graph
  edge transitions, and the review projection's exact source/target scopes.
- [x] Fresh Rust, Web, contract, and bilingual documentation receipts are recorded. The focused
  test and format check passed; the full workspace/Web/contract evidence is recorded below.

- [x] 使用既有 guarded lifecycle writer 与 exact commit pair 的红绿 integration proof。storage
  integration test 报告 `3 passed`，覆盖 initialize/create/update、第二个 component、Uses 添加/移除，以及
  missing 或 mismatched exact scope failure。
- [x] review service 消费 writer 产生的 lifecycle facts 与 graph snapshot，且没有第二个 calculator。成功路径断言
  revision provenance、replay commit scope、graph edge transition 与 review projection 的 exact source/target scope。
- [x] 记录新鲜 Rust、Web、contract 与双语文档回执。focused test 与 format check 已通过；完整 workspace/Web/contract
  evidence 记录如下。

## Fresh Verification Receipt / 新鲜验证回执

- `cargo test -p contextlab-storage --test context_graph_diff_review_lifecycle_integration --offline -- --nocapture`: `3 passed`.
- `cargo test -p contextlab-storage context_graph_diff_review --offline -- --nocapture`: `4 passed`.
- `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture`: `3 passed`.
- `cargo test --workspace --quiet --no-fail-fast --offline`: workspace passed; storage `221 passed, 41 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web` (`15/148/298` plus
  production build), local contract fixture verifier, and the single `impl GraphDiff` source count all passed.

- `cargo test -p contextlab-storage --test context_graph_diff_review_lifecycle_integration --offline -- --nocapture`：`3 passed`。
- `cargo test -p contextlab-storage context_graph_diff_review --offline -- --nocapture`：`4 passed`。
- `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture`：`3 passed`。
- `cargo test --workspace --quiet --no-fail-fast --offline`：workspace 通过；storage `221 passed, 41 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`（`15/148/298` 与
  production build）、local contract fixture verifier，以及唯一 `impl GraphDiff` source count 均通过。

PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and
production remain `ignored`, `unobserved`, or `deferred`; the long-term goal remains active.

PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production
继续为 `ignored`、`unobserved` 或 `deferred`；长期目标保持 active。
