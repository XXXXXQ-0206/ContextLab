# Private Context Graph Snapshot Integrity / 私有 Context Graph Snapshot 完整性

## Necessity Record / 必要性记录

**Named completion criteria and charter principles / 命名完成条件与章程原则:** This increment
directly serves Criterion 1 (Context-first graph coverage), Criterion 2 (replayable version history),
and Criterion 4 (reviewable semantic/versioned change). A commit-associated ContextGraph snapshot
must be immutable at the PostgreSQL boundary as well as at the Rust repository boundary.

本增量直接服务条件 1（Context-first graph coverage）、条件 2（可回放版本历史）与条件 4（可审阅的
semantic/versioned change）。commit-associated ContextGraph snapshot 必须在 PostgreSQL boundary 与 Rust
repository boundary 同时保持 immutable。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The typed scope,
Memory/PostgreSQL repository behavior, and version-backed GraphDiff adapter already exist and pass
local tests, but independent review found that migration `0002` permits direct PostgreSQL UPDATE or
DELETE of `context_commit_graph_snapshots`. The API also trusted a resolver's returned commit scope
without rechecking it against the request. These are fail-closed integrity gaps, not reasons to add
new transport.

typed scope、Memory/PostgreSQL repository behavior 与 version-backed GraphDiff adapter 已存在并通过本地测试，
但独立审查发现 migration `0002` 允许直接 UPDATE/DELETE `context_commit_graph_snapshots`；API 也曾直接信任
resolver 返回的 commit scope，未与 request 二次比对。这些是 fail-closed 完整性缺口，不是新增 transport 的理由。

**Why now / 为什么现在优先:** These are the smallest blocking defects in the already-admitted
snapshot contract. Fixing them before another Context consumer or editor depends on the read path
protects the version history and directly strengthens named completion criteria; benchmark, workflow,
and UI expansion can wait.

这是已准入 snapshot contract 中最小且会阻断可信度的缺陷。在任何新的 Context consumer 或 editor 依赖该 read
path 前修复它们，可以保护版本历史并直接强化命名完成条件；benchmark、workflow 与 UI 扩展可以后置。

**Smallest boundary and bilingual documentation / 最小边界与双语文档:** Add migration `0025`
for V1-only schema enforcement and append-only snapshot rows, add static migration and API exact-scope
regressions, add the route contract note in `docs/api/rest-api.md`, and record this plan plus the
roadmap receipt. No branch constraint, writer API redesign, public surface, or SDK/Web product change.

最小边界是新增 migration `0025`，强制 V1 schema 与 snapshot append-only；新增 migration static contract 与 API
exact-scope regression；修正 `docs/api/rest-api.md` 的 route contract，并记录本计划与 roadmap 回执。不新增 branch
constraint、writer API redesign、public surface 或 SDK/Web product change。

**Explicit non-goals / 明确非目标:** No public write, public OpenAPI/SDK method, Web mutation,
operator transport, branch/merge mutation, provider, raw private content, Docker or production claim,
and no second graph-diff calculator. `GraphDiff::between` remains the sole graph-diff calculator.

不新增 public write、public OpenAPI/SDK method、Web mutation、operator transport、branch/merge mutation、provider、
raw private content、Docker 或 production 声明，也不新增第二个 graph-diff calculator。`GraphDiff::between` 仍是唯一
graph-diff calculator。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** Run the focused storage
migration contract and API scope tests, then `cargo fmt --all -- --check`, offline workspace tests,
strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and the fixture verifier. PostgreSQL
runtime is only recorded if a fresh disposable non-production database is actually available; otherwise
it remains `ignored/unobserved`.

下一增量前运行 storage migration contract 与 API scope focused tests，随后运行 `cargo fmt --all -- --check`、offline
workspace tests、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web` 与 fixture verifier。只有确实拥有新的
disposable non-production database 时才记录 PostgreSQL runtime；否则继续标记为 `ignored/unobserved`。

## Implementation Receipt / 实现回执

- [x] Added the V1-only append-only PostgreSQL migration and composed it after the base snapshot schema.
- [x] Rechecked resolver scope against the requested Context and commit before version-backed review.
- [x] Added focused static migration and API helper regressions.
- [x] Fresh post-fix verification and bilingual roadmap receipt.

- [x] 新增 V1-only append-only PostgreSQL migration，并在 base snapshot schema 后组合执行。
- [x] 在 version-backed review 前将 resolver scope 与请求的 Context、commit 二次校验。
- [x] 新增 focused static migration 与 API helper regression。
- [x] 新鲜 post-fix verification 与双语 roadmap 回执。

## Fresh Local Verification Receipt / 新鲜本地验证回执

- `cargo test -p contextlab-storage --test commit_graph_snapshot_migration_contract --offline -- --nocapture`: `3 passed`, `0 failed`; V1 schema, append-only trigger, and migration composition checks passed.
- `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture`: `3 passed`, `0 failed`; resolver scope recheck, protected exact scope, and retired public read checks passed.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --offline --no-fail-fast`: passed; `220 passed`, `41 ignored`, `0 failed`. PostgreSQL runtime tests remained ignored because they require `CONTEXTLAB_TEST_DATABASE_URL`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; Web tests `298` and production build passed. The public SDK, local SDK, and Web checks completed within the command.
- `pwsh -NoProfile -File .\\tests\\contract\\verify-local-contracts.test.ps1`: fixture tests passed.

- `cargo test -p contextlab-storage --test commit_graph_snapshot_migration_contract --offline -- --nocapture`：`3 passed`、`0 failed`；V1 schema、append-only trigger 与 migration composition 检查通过。
- `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture`：`3 passed`、`0 failed`；resolver scope 二次校验、protected exact scope 与 retired public read 检查通过。
- `cargo fmt --all -- --check`：通过。
- `cargo test --workspace --quiet --offline --no-fail-fast`：通过；`220 passed`、`41 ignored`、`0 failed`。PostgreSQL runtime test 因需要 `CONTEXTLAB_TEST_DATABASE_URL` 继续 ignored。
- `cargo clippy --workspace --all-targets --offline -- -D warnings`：通过。
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`：通过。
- `pnpm check:web`：通过；Web tests `298` 与 production build 通过。public SDK、local SDK 与 Web 检查均在该命令内完成。
- `pwsh -NoProfile -File .\\tests\\contract\\verify-local-contracts.test.ps1`：fixture tests 通过。

This is fresh local contract evidence only. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred` as applicable;
no public write, public OpenAPI/SDK method, Web mutation, provider, secret access, or second `GraphDiff` calculator was
added. / 本回执仅是新鲜本地 contract evidence。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release 与 production 按实际情况继续标记为 `ignored`、`unobserved` 或 `deferred`；
未新增 public write、public OpenAPI/SDK method、Web mutation、provider、secret access 或第二个 `GraphDiff` calculator。
