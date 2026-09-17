# Private Context Commit Ancestry Read / 私有 Context Commit Ancestry 读取

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criteria 2 and 4. The existing persisted ContextGraph merge classifier is validated, but a safe
local transport cannot consume it until the `MergePlan` comes from the exact persisted Context
commit DAG. Context history must remain replayable and server-owned; `GraphDiff::between` remains
the only graph-diff calculator.

本增量直接推进条件 2 与 4。现有持久化 ContextGraph merge classifier 已完成验证，但在 `MergePlan` 来自精确持久化 Context
commit DAG 之前，安全的本地 transport 不能消费它。Context history 必须可回放且由服务端拥有；`GraphDiff::between` 仍是唯一
graph-diff calculator。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The reusable
`ContextCommitGraphRepository` now loads one complete exact-Context commit DAG through Memory and
PostgreSQL adapters. The remaining integration risk is that future merge consumers must use this
server-owned graph rather than accept an unchecked caller-supplied ancestry plan.

可复用的 `ContextCommitGraphRepository` 现已通过 Memory 与 PostgreSQL adapter 加载完整的 exact-Context commit DAG。剩余集成风险是，
未来 merge consumer 必须使用这个 server-owned graph，不能接受 caller 提交的未经校验 ancestry plan。

**Why now / 为什么现在优先:** The commit-associated graph snapshot contract, version-bound merge
review application contract, atomic persisted three-snapshot read, and pure classifier are already
locally verified. This is their smallest direct missing dependency and must precede any private
merge-review API/SDK/Web adapter. It is closer to the named version/replay criteria than new
benchmark, provider, or UI breadth.

commit-associated graph snapshot contract、version-bound merge review application contract、atomic persisted three-snapshot read 与
pure classifier 均已完成本地验证。这是它们当前最小且直接的缺失依赖，必须先于 private merge-review API/SDK/Web adapter。相比新增
benchmark、provider 或 UI breadth，它更直接服务命名的 version/replay 条件。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Add and
verify the reusable storage `ContextCommitGraphRepository` port, Memory/PostgreSQL adapters,
focused contract tests, and internal `WorkspaceDataRepository`/`AppState` delegation. Convert
stored UUID/parent rows into `contextlab-versioning::CommitGraphNode` and fail closed on missing,
invalid, duplicate, cross-Context, or cyclic history. Update this plan and the bilingual roadmap
ledgers with observed evidence.

增加并验证可复用的 storage `ContextCommitGraphRepository` port、Memory/PostgreSQL adapter、focused contract test，以及内部
`WorkspaceDataRepository`/`AppState` delegation。将存储的 UUID/parent row 转换为 `contextlab-versioning::CommitGraphNode`，并对
missing、invalid、duplicate、cross-Context 或 cyclic history fail closed。更新本计划与双语 roadmap ledger，记录实际观测证据。

**Explicit non-goals / 明确非目标:** No merge-review route, REST/OpenAPI/public SDK method,
Web/CLI/Desktop surface, merge writer, rollback, branch mutation, migration, provider call, raw
private content, secret access, Docker/PostgreSQL runtime claim, browser claim, operator transport,
release/production work, or second GraphDiff calculator.

不新增 merge-review route、REST/OpenAPI/public SDK method、Web/CLI/Desktop surface、merge writer、rollback、branch mutation、migration、
provider call、raw private content、secret access、Docker/PostgreSQL runtime claim、browser claim、operator transport、release/production
work 或第二个 GraphDiff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** First observe
red tests for exact Context graph loading and malformed-parent rejection, then green Memory and
PostgreSQL adapter-contract tests, `cargo fmt --all -- --check`, workspace Rust tests, strict
offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and `GRAPH_DIFF_IMPL_COUNT=1`.
PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and
production remain separately classified by actual evidence.

先观察 exact Context graph loading 与 malformed-parent rejection 的红测，再取得 Memory/PostgreSQL adapter contract 绿测、
`cargo fmt --all -- --check`、workspace Rust、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web` 与
`GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与 production
继续按真实证据单独分类。

## Implementation checklist / 实施清单

- [x] Add the reusable Context commit DAG repository port and exports.
- [x] Add Memory/PostgreSQL implementations with exact Context and fail-closed graph validation.
- [x] Add focused contract tests and WorkspaceDataRepository/AppState delegation.
- [x] Run fresh verification and update bilingual roadmap receipts.

- [x] 增加可复用 Context commit DAG repository port 与 exports。
- [x] 增加 Memory/PostgreSQL 实现，执行 exact Context 与 fail-closed graph validation。
- [x] 增加 focused contract tests 与 WorkspaceDataRepository/AppState delegation。
- [x] 运行新鲜验证并更新双语 roadmap 回执。

## Implementation Receipt / 实施回执

The exact Context commit-DAG port is implemented and exported. Memory and PostgreSQL adapters
validate persisted commit identity, parent identity, Context ownership, duplicate/missing history,
and versioning graph integrity fail closed. The internal `WorkspaceDataRepository` delegation,
`WorkspaceCatalogRepositories`, `AppState` construction, and typed getter are wired without
adding a route or transport. Focused storage ancestry tests passed (`2` integration tests and
`13` unit tests); API repository contract tests passed (`4`).

exact Context commit-DAG port 已实现并导出。Memory 与 PostgreSQL adapter 对 persisted commit identity、parent identity、Context ownership、
duplicate/missing history 与 versioning graph integrity 执行 fail-closed validation。内部 `WorkspaceDataRepository` delegation、
`WorkspaceCatalogRepositories`、`AppState` construction 与 typed getter 已接通，但没有新增 route 或 transport。storage ancestry focused
test 通过（`2` 个 integration test 与 `13` 个 unit test）；API repository contract test 通过（`4` 个）。

Fresh local verification passed: `cargo fmt --all -- --check`; `cargo test --workspace --quiet --no-fail-fast --offline`
with `208 passed, 39 ignored` in the storage suite; `cargo clippy --workspace --all-targets --offline -- -D warnings`;
`cargo +1.85.0 check --workspace --all-targets --locked --offline`; `pnpm check:web` with public SDK `15`, local SDK `117`,
Web `245`, TypeScript/lint, and production build; the public OpenAPI exclusion contract (`1 passed`); and
`GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and
production remain `unobserved` or `deferred`; the long-term goal remains active.

新鲜本地验证通过：`cargo fmt --all -- --check`；`cargo test --workspace --quiet --no-fail-fast --offline`，storage suite 为 `208 passed, 39 ignored`；
`cargo clippy --workspace --all-targets --offline -- -D warnings`；`cargo +1.85.0 check --workspace --all-targets --locked --offline`；
`pnpm check:web`（public SDK `15`、local SDK `117`、Web `245`、TypeScript/lint 与 production build）；public OpenAPI exclusion contract（`1 passed`）；
以及 `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`；长期目标保持 active。
