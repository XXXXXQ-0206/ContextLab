# Context Commit Parent Scope Integrity Plan / Context Commit 父提交范围完整性计划

**Goal / 目标：** Enforce the existing same-Context commit-parent invariant in PostgreSQL so malformed ancestry cannot enter durable history, while preserving the private replay resolver's fail-closed defense.

**Architecture / 架构：** Add a forward migration that records the owning Context on every `context_commit_parents` row, backfills it from the child commit, and binds both child and parent with composite foreign keys to `context_commits(context_id, id)`. The guarded writer binds the already-owned Context explicitly. Existing replay validation remains a defense for unavailable or legacy-corrupted history; it does not become a second graph-diff implementation.

**Tech Stack / 技术栈：** Rust stable, SQLx/PostgreSQL migrations and integration tests, existing guarded commit transaction, disposable loopback PostgreSQL 16.14 harness, and bilingual architecture documentation.

---

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment directly advances the named `Versioning and diff workflows` completion criterion and the charter's durable, reproducible persistent-storage principle. A replayable Context history must preserve a Context's commit-parent boundary in the database, not only in the writer and reader adapters.

**服务条件：** 本增量直接推进已命名的 `版本与 Diff 工作流` 完成条件，以及宪章中持久、可复现的存储原则。可回放的 Context 历史必须在数据库中保持 Context 的 commit-parent 边界，而不能只依赖 writer 和 reader adapter。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口：** `context_commits` already has the `(context_id, id)` unique key, and guarded writes validate parent scope, but `context_commit_parents` currently stores only global commit IDs. A direct SQL or operational bypass can therefore persist a foreign parent. The just-verified replay resolver rejects that corruption at read time, but durable history should prevent it from being written in the first place.

**未满足依赖、风险或证据缺口：** `context_commits` 已具备 `(context_id, id)` 唯一键，guarded write 也会验证 parent scope，但 `context_commit_parents` 目前只保存全局 commit ID。因此，direct SQL 或运维绕过仍可能持久化外部 Context 的 parent。刚完成验证的 replay resolver 会在读取时拒绝该损坏状态，但 durable history 应首先阻止其写入。

**Why now / 为什么现在优先：** The preceding replay increment exposed and contained this exact root cause with a fail-closed regression. The composite key needed for the narrow migration already exists, so this is the smallest dependency-ready persistence follow-up before Context/Prompt/Schema editing, branches, merges, or benchmark work. It is more direct than a UI slice because it closes the source-of-truth invariant on which later version workflows rely.

**为什么现在优先：** 前一回放增量已通过 fail-closed regression 暴露并控制了这一根因。该窄迁移所需的复合键已经存在，因此它是 Context/Prompt/Schema 编辑、分支、合并或 benchmark 工作之前最小、依赖就绪的持久化后续项。它比 UI 切片更直接，因为它收束了后续 version workflow 所依赖的 source-of-truth 不变量。

**Explicit non-goals / 明确非目标：** No public REST route, protected-write promotion, OpenAPI operation, SDK method, Web control, operator transport, branch/merge policy, graph editor, body diff, new graph-diff calculator, or `GraphDiff` change. This migration does not repair or publish a corrupted production database; remote CI, operator rehearsal, release, and production promotion are deferred future deployment work outside this local increment.

**明确非目标：** 不新增 public REST route、protected-write promotion、OpenAPI operation、SDK method、Web control、operator transport、branch/merge policy、graph editor、body diff、新的 graph-diff calculator，也不修改 `GraphDiff`。本迁移不会修复或发布到损坏的生产数据库；远端 CI、operator 演练、release 与生产推广是本地增量范围外的未来部署工作。

**Minimal affected boundary and bilingual docs / 最小受影响边界与双语文档：** Restrict edits to `crates/storage` migrations, migration aggregation/tests, the PostgreSQL commit-parent insert and directly related disposable fixtures, the isolated-test script registration, this plan, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, and roadmap evidence. Public API, SDK, Web, and `crates/diff-engine` remain unchanged.

**最小受影响边界与双语文档：** 修改范围仅限 `crates/storage` 的 migration、migration aggregation/test、PostgreSQL commit-parent insert 与直接相关的 disposable fixture、isolated-test script 登记、本计划、`ARCHITECTURE.md`、`docs/storage/persistence-foundation.md` 和路线图证据。Public API、SDK、Web 与 `crates/diff-engine` 保持不变。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Observe a red migration/integration test, then prove valid historical backfill, fail-fast rejection of malformed history, and direct cross-Context insertion rejection in the loopback disposable database. Run the focused writer and replay tests, all newly registered reset-per-test PostgreSQL cases, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, relevant shell guards, and public-surface/`GraphDiff` boundary searches. Local results remain non-production evidence; external deployment work is outside this increment.

**下一增量前的新鲜验证：** 先观察 migration/integration 的红测，再在 loopback disposable 数据库中证明合法历史 backfill、损坏历史会 fail-fast，以及 direct 跨 Context 插入会被拒绝。运行 focused writer 与 replay 测试、全部新登记的 reset-per-test PostgreSQL case、`cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、相关 shell guard，以及 public-surface/`GraphDiff` 边界搜索。本地结果仍是非生产证据；外部部署工作不属于本增量。

## Tasks / 任务

### Task 1: Prove the missing database invariant / 证明缺失的数据库不变量

- [x] Write a disposable PostgreSQL regression that applies the predecessor schema, creates valid same-Context parent history, upgrades it, and proves a direct cross-Context edge is rejected by the database. / 编写 disposable PostgreSQL 回归：应用前置 schema、创建合法同 Context parent 历史、执行升级，并证明数据库拒绝直接跨 Context edge。
- [x] Add migration-asset assertions that require the Context backfill and both composite foreign keys. / 增加 migration asset 断言，要求 Context backfill 与两端复合外键同时存在。
- [x] Observe the regression fail before the migration is added for the missing asset or accepted foreign edge. / 在增加迁移前观察因缺失 asset 或错误接受跨 Context edge 而产生的失败。

### Task 2: Add the forward scope migration / 增加前向范围迁移

- [x] Add `0015_context_commit_parent_scope_integrity.sql` to preflight malformed history, record/backfill `context_id`, require it, and bind both parent-table endpoints to `context_commits(context_id, id)` with the existing delete semantics. / 增加 `0015_context_commit_parent_scope_integrity.sql`：先检查损坏历史，再记录并回填必填 `context_id`，并以既有删除语义把 parent table 两端绑定到 `context_commits(context_id, id)`。
- [x] Register the migration in `CONTEXT_PLATFORM_MIGRATION` and any exact historical migration constants needed by the regression. / 在 `CONTEXT_PLATFORM_MIGRATION` 及回归所需的精确历史 migration constant 中登记该迁移。
- [x] Keep the replay resolver's defensive cross-Context rejection intact. / 保留 replay resolver 对跨 Context history 的防御性拒绝。

### Task 3: Bind storage writes and fixtures explicitly / 显式绑定存储写入与 fixture

- [x] Bind `context_id` in the PostgreSQL guarded writer's parent insert. / 在 PostgreSQL guarded writer 的 parent insert 中绑定 `context_id`。
- [x] Update only affected direct-SQL fixtures to provide their owning Context. / 仅更新受影响的 direct-SQL fixture，使其提供所属 Context。
- [x] Register the focused migration/integrity test so the isolated script count changes only when a genuinely separate reset case is added. / 登记聚焦 migration/integrity test，只有新增真正独立的 reset case 时才改变隔离脚本计数。

### Task 4: Document and verify / 记录并验证

- [x] Document the durable same-Context parent invariant and retain the private/public boundary in English and Chinese. / 以中英双语记录持久化同 Context parent 不变量，并保持 private/public 边界。
- [x] Run the fresh local verification named above and record only observed evidence. / 运行上述新鲜本地验证，只记录实际观察到的证据。
- [x] Update the active roadmap to name the following dependency-ready increment while keeping the long-term goal active. / 更新活跃路线图，命名下一项依赖就绪的增量，同时保持长期目标 active。

## Fresh Verification Record / 新鲜验证记录

On 2026-07-15, the migration asset contract first failed because no historical preflight existed, then passed after the preflight was added before every schema mutation. Independent review also identified two evidence gaps: a rejected malformed migration did not assert absence of partial catalog changes, and a parented idempotent replay did not count its durable rows. The minimum repair added both regressions without changing the public surface or writer architecture.

2026-07-15，migration asset contract 首先因缺少历史 preflight 而按预期失败；在所有 schema mutation 之前增加 preflight 后通过。独立审查还发现两个证据缺口：损坏历史被拒绝后没有断言 catalog 不存在部分变更，带 parent 的幂等 replay 也没有统计其持久化记录。最小修复补齐了这两项回归，未改变 public surface 或 writer 架构。

- The four focused loopback PostgreSQL cases passed after an explicit schema reset per case: valid backfill and delete semantics, malformed-history rejection with no `context_id` column or scope constraints left behind, direct child/parent scope rejection, and parented replay with exactly one commit, parent link, snapshot, idempotency receipt, and matching branch head.
- 四个聚焦 loopback PostgreSQL case 在每例显式 reset schema 后通过：合法 backfill 与删除语义；损坏历史被拒绝且未留下 `context_id` column 或 scope constraint；直接 child/parent scope 拒绝；以及带 parent 的 replay 只保留一个 commit、parent link、snapshot、idempotency receipt 与匹配的 branch head。
- All 25 cases registered by `scripts/verify-disposable-postgres-storage.sh` passed against the local `contextlab-postgres-disposable` PostgreSQL 16 container, bound exactly to `127.0.0.1:55432`, with a fresh random test password and a full `public` schema reset before each case.
- `scripts/verify-disposable-postgres-storage.sh` 登记的 25 个 case 均在本地 `contextlab-postgres-disposable` PostgreSQL 16 容器中通过；容器只绑定 `127.0.0.1:55432`，本轮使用随机测试密码，并在每个 case 前完整 reset `public` schema。
- `cargo fmt --all -- --check` and `cargo test --workspace` passed; `contextlab-storage` reported `142 passed, 25 ignored`, with all ignored registered PostgreSQL cases exercised separately. `pnpm check:web` passed 14 SDK tests, 19 Web tests, type checks, and the production Next.js build. The Bash syntax guard and 25-invocation verifier self-test also passed.
- `cargo fmt --all -- --check` 与 `cargo test --workspace` 通过；`contextlab-storage` 报告 `142 passed, 25 ignored`，其中已登记的 PostgreSQL ignored case 均已单独执行。`pnpm check:web` 通过 14 个 SDK test、19 个 Web test、type check 与 Next.js production build。Bash syntax guard 与 25 次调用的 verifier self-test 也通过。
- Boundary searches found only the existing pure `POST /api/v1/graph-diffs` in public OpenAPI/SDK mutation entries. Context commit mutation remains confined to the protected router, and the two production graph-diff call sites still delegate to `GraphDiff::between` in `crates/diff-engine`.
- 边界搜索在 public OpenAPI/SDK mutation entry 中仅发现既有的纯计算 `POST /api/v1/graph-diffs`。Context commit mutation 仍仅位于 protected router，两处生产 graph-diff 调用仍委托给 `crates/diff-engine` 中的 `GraphDiff::between`。
- This evidence is local and non-production. Remote CI, operator rehearsal, public protected-write promotion, release, and production rollout are unavailable external deployment prerequisites and were neither run nor claimed.
- 这些证据仅限本地、非生产环境。远端 CI、operator 演练、public protected-write promotion、release 与生产推广属于当前不可用的外部部署前置，本轮未运行也未声称其完成。
