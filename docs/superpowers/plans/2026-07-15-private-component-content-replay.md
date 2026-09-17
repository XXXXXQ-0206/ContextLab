# Private Component Content Replay Resolution Plan / 私有 Component 正文回放解析计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标：** Add a private storage read contract that resolves the immutable component body revision reachable from a target normal Context commit without adding a public body-read surface.

**Architecture / 架构：** Extend the existing `ComponentContentRevisionRepository` port with `get_component_content_at_commit(context_id, commit_id, component_id)`. The in-memory adapter walks stored `ContextCommitRecord.parent_commit_ids`; PostgreSQL uses a recursive first-parent query over `context_commits` and `context_commit_parents`. Both adapters validate the complete target ancestry before returning its nearest revision, return `None` when the component has no captured body on a valid known chain, distinguish an unknown Context from an unknown commit, and reject merge, cycle, or malformed cross-Context parent ancestry rather than silently choosing a parent.

**Tech Stack / 技术栈：** Rust stable, async traits, SQLx/PostgreSQL recursive CTE, existing Context commit records, immutable component revisions, and the current guarded storage test harness.

---

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment directly advances `Versioning and diff workflows` by making commit-bound component content replayable at a target commit, and advances `Context-first platform coverage` by giving the existing immutable body revision a reusable historical read contract. It also preserves the charter principles of reusable Rust domain boundaries, persistent storage, reproducibility, and bilingual documentation.

**服务条件：** 本增量直接推进 `版本与 Diff 工作流`：使绑定 commit 的 component 正文可以在目标 commit 上回放；同时推进 `以 Context 为核心的平台覆盖`：为现有不可变 body revision 提供可复用的历史读取 contract，并保持可复用 Rust domain boundary、持久化存储、可复现性与双语文档等宪章原则。

**Unmet dependency or evidence gap / 未满足依赖或证据缺口：** Creation and existing-component revision now persist immutable body revisions atomically, but the repository can read only an exact `(context, commit, component)` revision. A later commit that does not itself change the component cannot resolve the body that was active at that point, so replay consumers would need to duplicate ancestry logic. Merge ancestry has no approved body-resolution policy and must remain explicit rather than being guessed.

**未满足依赖或证据缺口：** Creation 与既有 component revision 现已原子持久化不可变 body revision，但 repository 只能读取精确的 `(context, commit, component)` revision。没有修改该 component 的后续 commit 无法解析当时生效的正文，replay consumer 只能重复 ancestry logic。Merge ancestry 尚无获准的 body-resolution policy，必须显式拒绝，不能猜测。

**Why now / 为什么现在优先：** The preceding private creation slice and its `0012 -> 0014` PostgreSQL integrity evidence are fresh, while the existing commit records already preserve ordered parents in both adapters. This is the smallest dependency-ready increment that closes a named replay gap before prompt/schema editing, graph editing, branches, merges, or benchmark work. It is higher priority than a UI editor because it keeps historical body semantics in the reusable storage port instead of page-local code.

**为什么现在优先：** 前一私有 creation slice 及其 `0012 -> 0014` PostgreSQL integrity evidence 已获得新鲜验证，而两个 adapter 的现有 commit record 已保存有序 parent。该增量是 prompt/schema editing、graph editing、branch、merge 或 benchmark 之前，最小且依赖已满足的 replay gap 收束项。它优先于 UI editor，因为历史正文语义应保留在可复用 storage port，而不是页面局部代码中。

**Explicit non-goals / 明确非目标：** No public REST route, OpenAPI operation, TypeScript SDK method, Web body read or mutation control, operator transport, merge policy, rollback checkout, branch mutation, graph editing, body diff calculation, second `GraphDiff` calculator, or release/public-write promotion. The resolver supports only a single normal first-parent chain; it does not infer content across merge parents.

**明确非目标：** 不新增 public REST route、OpenAPI operation、TypeScript SDK method、Web body read 或 mutation control、operator transport、merge policy、rollback checkout、branch mutation、graph editing、body diff calculation、第二个 `GraphDiff` calculator，也不做 release/public-write promotion。resolver 只支持单一 normal first-parent chain，不会跨 merge parent 推断正文。

**Minimal affected boundary and bilingual docs / 最小受影响边界与双语文档：** Modify only `crates/storage` domain/repository contracts and its in-memory/PostgreSQL adapters/tests, the private plan, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/long-term-roadmap.md`, and `docs/roadmap/completion-criteria.md`. Public API, SDK, OpenAPI, Web, and `crates/diff-engine` remain unchanged.

**最小受影响边界与双语文档：** 只修改 `crates/storage` domain/repository contract、内存/PostgreSQL adapter 与测试，以及本 private plan、`ARCHITECTURE.md`、`docs/storage/persistence-foundation.md`、`docs/roadmap/active-long-term-goal.md`、`docs/roadmap/long-term-roadmap.md` 与 `docs/roadmap/completion-criteria.md`。Public API、SDK、OpenAPI、Web 与 `crates/diff-engine` 保持不变。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Observe red memory and PostgreSQL contract tests; then run the focused storage resolver tests, all 22 reset-per-test disposable PostgreSQL cases, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, shell guard/syntax tests, and boundary searches. The local database run remains non-production evidence; remote CI and operator rehearsal receipts remain deferred release evidence.

**下一增量前的新鲜验证：** 先观察内存与 PostgreSQL contract red test；随后运行 focused storage resolver test、全部 22 个逐例 reset 的 disposable PostgreSQL case、`cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、shell guard/syntax test 与 boundary search。本地数据库运行仍是 non-production evidence；remote CI 与 operator rehearsal receipt 仍属于延期 release evidence。

## Tasks / 任务

### Task 1: Define the private replay result and port / 定义私有回放结果与 port

**Files / 文件：**
- Modify: `crates/storage/src/component_content_revision.rs`
- Modify: `crates/storage/src/lib.rs`
- Test: `crates/storage/src/component_content_revision.rs`

- [x] **Step 1: Write the failing contract tests / 编写失败 contract test**

Add unit-level contract coverage showing that the port distinguishes an exact revision from an ancestry-resolved revision and that the resolver's result keeps the immutable revision's source commit identity. The new API must use typed `ContextId`, `CommitId`, and `ComponentId`; it must not accept arbitrary route strings.

- [x] **Step 2: Add the private read method / 增加私有读取方法**

Add `get_component_content_at_commit` to `ComponentContentRevisionRepository` with the same `Result<Option<ComponentContentRevision>, StorageRepositoryError>` shape as exact lookup. Document that `Some` is the nearest body revision on the target's normal first-parent ancestry, `None` means no captured body is reachable, and merge ancestry is a conflict rather than an implicit parent choice.

- [x] **Step 3: Verify the contract compiles / 验证 contract 编译**

Run `cargo test -p contextlab-storage component_content_revision`; the new tests must fail before adapters implement the method and then pass after both adapters are wired.

### Task 2: Implement deterministic in-memory first-parent replay / 实现确定性的内存 first-parent 回放

**Files / 文件：**
- Modify: `crates/storage/src/memory.rs`
- Test: `crates/storage/src/memory.rs`

- [x] **Step 1: Write the failing ancestry test / 编写失败 ancestry test**

Create an initial component revision, create a later normal commit without a component revision, then resolve at that later commit and assert the initial body. Add a subsequent component revision and assert that the resolver returns the new body at that commit while the earlier target still returns the initial body. Also cover unknown Context and target commit, unknown component body (`None`), a target with two parents, and a nearer revision whose ancestor is a merge.

- [x] **Step 2: Walk only the stored normal parent / 只沿存储的 normal parent 回放**

Under the existing read lock, locate the target in preview or dynamic `ContextCommitRecord`, require the requested Context, and iterate `parent_commit_ids.first()` while tracking visited commit IDs. Retain the first matching `component_content_revisions` entry but return it only after reaching a valid root; return `None` at an unborn root with no matching revision; return `ScopeUnavailable` for an unknown Context or target and `ComponentContentRevisionConflict` for merge ancestry or a cycle.

- [x] **Step 3: Run in-memory resolver tests / 运行内存 resolver test**

Run `cargo test -p contextlab-storage memory::tests::in_memory_component_content_replays_the_nearest_normal_parent_revision --lib -- --exact --test-threads=1`; all ancestry, no-body, scope, merge-boundary, and full-chain validation assertions must pass.

### Task 3: Implement PostgreSQL recursive ancestry replay / 实现 PostgreSQL recursive ancestry 回放

**Files / 文件：**
- Modify: `crates/storage/src/postgres.rs`
- Test: `crates/storage/src/postgres.rs`
- Modify: `scripts/verify-disposable-postgres-storage.sh`
- Modify: `scripts/verify-disposable-postgres-storage.test.sh`

- [x] **Step 1: Write the failing disposable PostgreSQL test / 编写失败 disposable PostgreSQL test**

Using the existing guarded writer, create a component, create an unrelated child commit, revise the component on a later normal child, and assert that the private resolver returns the nearest reachable revision for each target. Assert unknown Context and target, merge ancestry, and direct-SQL malformed cross-Context parent failures without adding any route or SDK call. Register the test name in the disposable script so the expected isolated count increases from 21 to 22.

- [x] **Step 2: Add one recursive first-parent query / 增加 recursive first-parent query**

Use a parameterized `WITH RECURSIVE` query rooted at the requested `(context_id, commit_id)`, follow only `context_commit_parents.position = 0`, return ancestry depth plus revision fields, and surface parent counts and Context mismatches so the adapter rejects any merge or malformed cross-Context parent in the traversed chain. Check Context existence before traversal, hydrate through the existing persisted-revision parser, and never concatenate IDs or body content into SQL.

- [x] **Step 3: Run the focused database test / 运行 focused database test**

Reset only the verified loopback disposable database and run the new ignored test with `--ignored --exact --test-threads=1`. It must pass without reading secrets or connecting to remote/production infrastructure.

### Task 4: Document and verify the private replay boundary / 记录并验证私有回放边界

**Files / 文件：**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: this plan

- [x] **Step 1: Record the private first-parent rule / 记录 private first-parent 规则**

Document the return semantics, merge rejection, exact source commit identity, no-body result, and the fact that GraphDiff remains the only graph-diff calculator. State in English and Chinese that public body reads/writes and graph editing remain absent.

- [x] **Step 2: Run fresh cross-stack verification / 运行新鲜跨栈验证**

Run:

```powershell
cargo fmt --all -- --check
cargo test -p contextlab-storage component_content_at_commit
cargo test --workspace
pnpm check:web
```

Run the 22 reset-per-test local PostgreSQL cases and both shell guard tests. Search `server/api`, `docs/api/openapi.json`, `packages/ts-sdk`, `apps/web`, and `crates/diff-engine` for new public body transport or a second diff calculator; expected result is no new public replay surface and one `GraphDiff` implementation path.

- [x] **Step 3: Update the active next increment / 更新 active next increment**

Only after fresh verification, update `active-long-term-goal.md` and the roadmap to name this slice as verified progress and select the next dependency-ready unmet criterion. Keep the long-term goal active.

## Scope Self-Review / 范围自审

- The contract is private storage-only and depends on already persisted commit parents and immutable revisions.
- First-parent-only resolution is explicit; merge, rollback, branch mutation, UI, public transport, and GraphDiff changes are excluded.
- `None`, unknown Context, unknown target, unsupported merge ancestry, cycles, and malformed cross-Context parents are distinct outcomes, so callers cannot mistake missing body data for missing commits or silently accept ambiguous history.
- All database input is parameterized, all IDs remain typed at the domain boundary, and all verification stays local/non-production unless an external receipt is independently observed.
