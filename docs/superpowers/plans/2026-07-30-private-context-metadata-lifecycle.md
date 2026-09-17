# Private Context Metadata Lifecycle / 私有 Context Metadata 生命周期

**Status / 状态:** admitted / admitted for implementation / 已准入实现

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与章程原则

This increment directly closes a named gap in Criterion 1, Context-first platform coverage:
Context metadata must have a domain model, replayable version history, private persistence/API/SDK
contracts, and a local inspection or editing path. `ContextMetadata` and
`UpdatedMetadata` already exist in the reusable Rust core, but the private Context lifecycle
contract cannot author the change or return the replayed metadata at a selected commit.

本增量直接收束条件 1（以 Context 为核心的平台覆盖）中的已命名缺口：Context metadata 必须具备 domain model、可回放版本历史、
private persistence/API/SDK contract，以及本地 inspection 或 editing path。可复用 Rust core 已有 `ContextMetadata` 与
`UpdatedMetadata`，但 private Context lifecycle contract 还不能创建该变更，也不能在选定 commit 返回回放后的 metadata。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

`ContextLifecycleOperation` currently covers initialization, component content/descriptor changes,
removal, and graph relationships only. `ContextLifecycleStateAtCommit` returns components and a
graph snapshot but omits the `ReplayState` metadata already reconstructed by storage. The API,
non-public local SDK, and Web editor therefore cannot represent or verify a metadata-only commit.
Adding a parallel metadata store would break the existing version/replay source of truth; the
change must be encoded as the existing `ContextChange::UpdatedMetadata` and persisted by the same
guarded writer.

当前 `ContextLifecycleOperation` 只覆盖初始化、component content/descriptor 变化、删除与图关系；`ContextLifecycleStateAtCommit` 返回
components 与 graph snapshot，却遗漏 storage 已重建的 `ReplayState` metadata。因此 API、非公开 local SDK 与 Web editor 无法表达或
验证 metadata-only commit。新增平行 metadata store 会破坏既有 version/replay source of truth；必须使用已有
`ContextChange::UpdatedMetadata` 编码，并通过同一个 guarded writer 持久化。

### Why now / 为什么现在优先

Five bounded Luna audits independently excluded completed component replay, graph/merge review,
benchmark read flows, and Workflow status projection as the next Criterion 1 increment. They
identified metadata lifecycle as the smallest dependency-ready Context-first discontinuity.
Workflow execution persistence is a larger Criterion 5 increment and remains a later queue item;
the stale benchmark verifier and CLI process smoke are separate evidence cleanups, not substitutes
for this missing Context lifecycle capability.

五个有界 Luna 审计分别排除了已完成的 component replay、graph/merge review、benchmark read flow 与 Workflow status projection，
并确认 metadata lifecycle 是当前最小且依赖就绪的 Criterion 1 Context-first 缺口。Workflow execution persistence 属于更大的条件 5
增量，留在后续队列；陈旧 benchmark verifier 与 CLI process smoke 是独立的证据清理，不能替代当前缺失的 Context lifecycle 能力。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

The implementation boundary is limited to:

- `crates/storage/src/context_lifecycle.rs` and focused lifecycle tests: operation, command, guarded
  commit composition, replayed state projection, and Memory/PostgreSQL repository parity;
- the existing private lifecycle adapter in `server/api/src/routes.rs` and its focused tests;
- `packages/local-sdk/src/types.ts`, `client.ts`, and focused parser/client tests;
- the existing local Context lifecycle data/presenter/editor/screen adapters and focused Web tests;
- this plan plus the latest bilingual roadmap and completion-criteria receipts.

实现边界仅限：

- `crates/storage/src/context_lifecycle.rs` 及 focused lifecycle tests：operation、command、guarded commit composition、回放 state projection
  与 Memory/PostgreSQL repository parity；
- `server/api/src/routes.rs` 中既有 private lifecycle adapter 及 focused tests；
- `packages/local-sdk/src/types.ts`、`client.ts` 及 focused parser/client tests；
- 既有 local Context lifecycle data/presenter/editor/screen adapter 与 focused Web tests；
- 本计划以及最新双语 roadmap 与 completion-criteria 回执。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK method, public write promotion, operator transport, or new
  mutation surface.
- No Workflow execution producer/persistence, benchmark execution, provider call, MCP/plugin
  loading, merge writer, rollback, branch mutation, or second GraphDiff calculator.
- No separate metadata table or parallel source of truth; no raw secrets or external service access.
- No Docker/PostgreSQL runtime claim, authenticated browser/visual claim, release, production, or
  remote evidence claim. Local PostgreSQL evidence remains unobserved when the runtime is unavailable.

- 不新增 public REST/OpenAPI/public SDK method、public write promotion、operator transport 或新的 mutation surface；
- 不实现 Workflow execution producer/persistence、benchmark execution、provider call、MCP/plugin loading、merge writer、rollback、branch mutation
  或第二个 GraphDiff calculator；
- 不新增独立 metadata table 或平行 source of truth；不读取 secret，不访问外部服务；
- 不声称 Docker/PostgreSQL runtime、authenticated browser/visual、release、production 或 remote evidence 已通过；runtime 不可用时本地 PostgreSQL
  evidence 继续标记为 unobserved。

### Fresh verification required before the next increment / 下一增量前的新鲜验证

Observe red tests for the missing metadata lifecycle operation and green focused Rust/API/SDK/Web
tests. Then run `cargo fmt --all -- --check`, `cargo test --workspace --quiet --no-fail-fast`,
strict offline workspace Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and static checks for
one `impl GraphDiff` plus no public metadata write surface. Docker/PostgreSQL runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred` unless independently observed.

先观察 metadata lifecycle operation 缺失的 red test，再取得 Rust/API/SDK/Web focused green tests；随后运行
`cargo fmt --all -- --check`、`cargo test --workspace --quiet --no-fail-fast`、strict offline workspace Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`，以及唯一 `impl GraphDiff` 与无 public metadata write surface 的静态检查。Docker/PostgreSQL runtime、authenticated browser、Git、
remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`，除非有独立真实回执。

## Ownership / 所有权

- Luna storage/domain worker: `crates/storage/src/context_lifecycle.rs` and focused storage tests only.
- Luna API/SDK worker: `server/api/src/routes.rs` and `packages/local-sdk/src/{types.ts,client.ts}` with focused tests only.
- Luna Web worker: existing local Context lifecycle data/presenter/editor/screen adapters and focused tests only.
- Integration Lead: root manifests if strictly required, this plan, bilingual roadmap receipts, contract review, and all final verification.

All workers use `gpt-5.6-luna`, do not read secrets, do not revert unrelated work, and must report
any ownership conflict before editing. No worker may add public REST/OpenAPI/public SDK surface or
reimplement graph diff logic.

所有 worker 使用 `gpt-5.6-luna`，不得读取 secret，不得回退无关改动；发现 ownership 冲突必须先报告。任何 worker 都不得新增 public
REST/OpenAPI/public SDK surface，也不得重算 graph diff。

## 2026-07-30 Closure Necessity Record / 2026-07-30 收束必要性记录

The bounded closure is necessary because the Context-first gap named above crossed the full private
lifecycle boundary: a metadata-only change must be authored as the existing versioned change, guarded
by the existing writer, replayable at an exact commit, and inspectable through the existing private
API/SDK/Web path. A partial local type or UI control would not establish that contract.

本次有界收束是必要的，因为上文所述 Context-first 缺口横跨完整的 private lifecycle boundary：metadata-only change 必须编码为既有
versioned change，经既有 guarded writer 写入，在精确 commit 上可回放，并能通过既有 private API/SDK/Web path inspection。只有局部类型或
UI 控件不能证明该 contract 已闭合。

## 2026-07-30 Integration Receipt / 2026-07-30 集成回执

`completed / verified locally` for this bounded private metadata slice only; this is not project or
long-term-goal completion. The observed chain is `ContextLifecycleOperation::UpdateMetadata` -> the
guarded lifecycle writer and replay projection -> protected private API state/write contracts -> strict
local SDK V1 parsing and scope checks -> existing Web presenter/editor read-and-submit path. Current
code locations are `crates/storage/src/context_lifecycle.rs:31,192,419,670`,
`server/api/src/routes.rs:171,195,235,475,502`, `packages/local-sdk/src/types.ts:25,48,172,628`,
`packages/local-sdk/src/client.ts:88,386`, and
`apps/web/src/app/context-lifecycle-presenter.ts:103,115` plus
`apps/web/src/app/context-lifecycle-editor.tsx:262,422,546`.

本回执只将该 private metadata slice 标记为 `completed / verified locally`，不代表项目或长期目标完成。已观察链路为
`ContextLifecycleOperation::UpdateMetadata` -> guarded lifecycle writer 与 replay projection -> protected private API state/write contract ->
strict local SDK V1 parsing 与 scope check -> 既有 Web presenter/editor 读取与提交路径。当前代码位置见上述英文行号。

Fresh local receipts: `cargo test -p contextlab-storage lifecycle_update_metadata --offline` passed `3`;
`cargo test -p contextlab-api protected_local_lifecycle_updates_and_replays_context_metadata --offline`
passed `1`; the focused lifecycle data/proxy command passed `14`; the full local SDK suite passed
`131`; and the full Web suite passed `265`.
No code or test files were changed in this docs-only integration pass.

新鲜本地回执：storage 命令通过 `3` 项；API 命令通过 `1` 项；lifecycle data/proxy focused command 通过 `14` 项；完整 local SDK 通过 `131` 项；
完整 Web 通过 `265` 项。本次仅为文档集成，
没有修改 code 或 test 文件。

Not observed or deferred by this receipt: full workspace/production verification, PostgreSQL or Docker
runtime, authenticated browser or visual smoke, Git change-set, remote CI, operator rehearsal, release,
production deployment, public REST/OpenAPI/public SDK promotion, and secrets or external systems. The
long-term goal remains active.

本回执未观察或延期：full workspace/production verification、PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、
remote CI、operator rehearsal、release、production deployment、public REST/OpenAPI/public SDK promotion，以及 secret 或 external system。长期目标仍为 active。

## 2026-07-30 Fixture Contract Repair / 2026-07-30 Fixture 契约修复

The lifecycle red phase exposed three stale Web fixtures rather than a production defect: one
metadata lookup used an old component identity, and two proxy success payloads omitted the response
schema version and canonical UUID snapshot identities. The minimum repair updated only the focused
fixtures to the frozen local-SDK contract; production adapters were not weakened.

本次 lifecycle 红灯暴露的是三个过时 Web fixture，而不是生产逻辑缺陷：一个 metadata lookup 使用旧 component identity，两个 proxy
成功 payload 缺少 response schema version 与 canonical UUID snapshot identity。最小修复仅更新 focused fixture 以符合 frozen local-SDK
contract，没有放宽 production adapter 校验。

Observed red/green evidence: the stale command failed `3` tests, then the focused lifecycle
data/proxy command passed `14`; the full Web suite passed `265`; `pnpm check:web` passed public SDK
`15`, local SDK `131`, Web `265`, TypeScript/lint, and production build. Rust workspace tests
passed with `40` ignored, format passed, strict offline Clippy passed, locked Rust `1.85.0` passed,
and static inspection found one `impl GraphDiff`.

新鲜红绿证据：旧 fixture 命令先失败 `3` 项，随后 lifecycle data/proxy focused command 通过 `14` 项；完整 Web 通过 `265` 项；
`pnpm check:web` 通过 public SDK `15`、local SDK `131`、Web `265`、TypeScript/lint 与 production build。Rust workspace test 通过且
观察到 `40` 个 ignored，format、strict offline Clippy、锁定 Rust `1.85.0` 与唯一 `impl GraphDiff` 静态检查均通过。

This repair remains local/private. PostgreSQL/Docker runtime, authenticated browser/visual smoke,
Git change-set, remote CI, operator rehearsal, release, production, and public promotion remain
`unobserved` or `deferred`; the long-term goal remains active.

本修复仍是 local/private。PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、remote CI、operator rehearsal、
release、production 与 public promotion 继续为 `unobserved` 或 `deferred`；长期目标保持 active。

## Next Necessity Record / 下一项必要性记录

The next admitted local increment is a private version-bound Context metadata semantic-diff
projection. Replayable metadata now exists at exact commits, but the current semantic snapshot
contract contains graph and documents only, so a metadata-only commit can still appear unchanged in
Context diff review. This directly serves Criteria 1 and 2 and is smaller than another UI or
transport surface.

下一项准入的本地增量是 private、version-bound 的 Context metadata semantic-diff projection。metadata 已可在精确 commit 回放，但当前
semantic snapshot contract 仅包含 graph 与 documents，因此 metadata-only commit 仍可能在 Context diff review 中显示为 unchanged。本项
直接服务条件 1 与 2，且边界小于新增 UI 或 transport surface。

Boundary: extend the reusable `contextlab-diff-engine` V1 semantic contract and its focused tests,
then adapt existing private persisted snapshot construction only where required. Non-goals are
public REST/OpenAPI/SDK changes, Web/CLI/Desktop work, mutations, migrations, provider/evaluator
calls, PostgreSQL runtime claims, release work, and any second `GraphDiff` calculator.

边界：扩展可复用 `contextlab-diff-engine` V1 semantic contract 与 focused tests，仅在必要处适配既有 private persisted snapshot construction。
非目标包括 public REST/OpenAPI/SDK 变化、Web/CLI/Desktop、mutation、migration、provider/evaluator call、PostgreSQL runtime 声明、release
工作与第二个 `GraphDiff` calculator。

Before the next increment, require a red test proving metadata-only change is invisible to the old
contract, a green deterministic exact-scope metadata diff test, the workspace Rust/Web gates above,
and a static proof that `GraphDiff::between` remains the sole graph calculator.

下一增量开始前必须取得：证明旧 contract 看不见 metadata-only change 的 red test、确定性 exact-scope metadata diff green test、上述 Rust/Web
门禁，以及 `GraphDiff::between` 仍是唯一 graph calculator 的静态证明。
