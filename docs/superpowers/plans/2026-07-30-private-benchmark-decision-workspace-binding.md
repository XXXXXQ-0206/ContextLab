# Private Benchmark Decision-to-Workspace Binding / 私有 Benchmark Decision 到 Workspace 绑定

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Criteria 2 and 4: benchmark evidence must be
addressable by the exact Context commit and decision that produced it, while the
evaluation, scorecard, regression, and diff policy remains in reusable Rust/storage
contracts and the Web remains a read-only presenter.

**Unmet dependency or evidence gap / 未满足依赖或证据缺口：** The local benchmark
surface can list exact sealed decisions and can render a cohort-keyed workspace, but
the consumer still asks for a manually supplied revised cohort. There is no strict
read-side contract resolving `(project, context, commit, decision)` to the existing
sealed workspace projection. This permits an incorrect cohort to be paired with a
listed decision and leaves the local workflow incomplete.

**Why now / 为什么现在优先：** The decision list, sealed run details, and evaluation
workspace projection already exist and their policies are tested. Exact decision
resolution is the smallest dependency-ready bridge that makes the benchmark vertical
slice operable without adding a writer, provider call, or new evaluation algorithm.

**Non-goals / 明确非目标：** No benchmark mutation, provider execution, new scoring or
regression algorithm, public REST/OpenAPI/SDK write, operator transport, migration,
Docker/PostgreSQL runtime claim, secret access, or production/release claim. The Web
must not derive cohorts from run IDs or recompute evaluation results.

**Smallest boundary and bilingual documentation / 最小边界与双语文档：** Add one
private exact-decision read contract and adapters across the existing local API/SDK/Web
read layers, preserving auth/RBAC/audit/rate-limit/no-store and the existing sealed
projection producer. Keep `data -> presenter -> screen`, update this plan and the
parallel/active/completion records after fresh verification, and document every
unobserved or deferred environment separately in English and Chinese.

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Focused
Rust/API/storage tests for exact scope and sealed-only resolution; local SDK and Web
contract tests for selection, invalidation, and no manual cohort derivation; then
workspace Rust, format, strict offline Clippy, locked Rust 1.85, `pnpm check:web`,
and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL/Docker runtime, authenticated browser, Git,
remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`.

## Tasks / 任务

- [x] Add the exact decision-to-workspace read DTO and fail-closed parser.
- [x] Resolve the existing sealed projection without duplicating evaluation policy.
- [x] Bind the local SDK and Web selection flow without mutation or client-side diff logic.
- [x] Add focused regression tests for wrong decision/commit, no selection, and stale selection.
- [x] Record fresh local evidence and update bilingual roadmap/audit documents.

## Root Cause and Minimal Repair / 根因与最小修复

The first Benchmark worker correctly reported that the existing workspace reader was cohort-keyed
and that decision-to-cohort mapping lived only inside storage. Reconstructing it in API would have
duplicated evaluation policy and would not survive restart. The Integration Lead therefore added
`BenchmarkWorkspaceProjectionDecisionQuery` and a private resolver to the storage reader contract,
with Memory and PostgreSQL implementations, then connected the protected decision route and its
non-public SDK/BFF/Web adapters to that resolver. The worker that first attempted the adapter was
closed after no timely delivery; its useful boundary finding was retained, and a Luna backup plus
the Integration Lead completed the code without overlapping ownership.

首个 Benchmark worker 真实指出：现有 workspace reader 只按 cohort 读取，decision 到 cohort 的映射只在 storage
内部存在；若在 API 重建，会重复 evaluation policy 且无法跨重启稳定工作。因此 Integration Lead 在 storage reader
contract 增加 `BenchmarkWorkspaceProjectionDecisionQuery` 与 private resolver，并分别实现 Memory/PostgreSQL parity，
再将 protected decision route 与非公开 SDK/BFF/Web adapter 接入该 resolver。首个 adapter worker 未在时限内交付后被
关闭，其有效边界发现被保留；随后由 Luna backup 与 Integration Lead 在不重叠 ownership 下完成实现。

## Fresh Verification Record / 新鲜验证记录

Storage projection resolver focused tests passed `2`; protected decision workspace API tests passed
`10`; storage package passed `204` with `39` ignored; API passed `191`; workflow passed; workspace
Rust passed; `cargo fmt --all -- --check`; strict offline workspace Clippy; locked Rust `1.85.0`
check; local SDK passed `100`; `pnpm check:web` passed public SDK `15`, local SDK `100`, Web `207`,
TypeScript/lint, and production build. Static inspection reports `GRAPH_DIFF_IMPL_COUNT=1`.

Storage projection resolver focused test `2` 项通过；protected decision workspace API test `10` 项通过；storage package
为 `204 passed, 39 ignored`；API `191 passed`；workflow、workspace Rust、`cargo fmt --all -- --check`、strict offline
workspace Clippy、锁定 Rust `1.85.0` check 均通过；local SDK `100` 项通过；`pnpm check:web` 通过 public SDK `15`、
local SDK `100`、Web `207`、TypeScript/lint 与 production build。静态检查为 `GRAPH_DIFF_IMPL_COUNT=1`。

PostgreSQL/Docker runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; no secret was read and no public write
surface was added.

PostgreSQL/Docker runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production
继续为 `unobserved` 或 `deferred`；未读取 secret，也未新增 public write surface。
