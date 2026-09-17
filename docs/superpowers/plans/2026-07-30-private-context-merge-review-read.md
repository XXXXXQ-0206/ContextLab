# Private Server-Owned Context Merge Review Read / 私有服务端拥有的 Context Merge Review 读取

## Necessity Record / 必要性记录

**Completion criteria and charter principles / 完成条件与章程原则:** This increment directly
advances Criteria 2 and 4. The reusable Rust/storage layers now resolve a three-way `MergePlan`
from exact Context commit ancestry and classify the persisted graph snapshots. That result remains
unreachable to a local consumer. A private read contract makes replayable version history and the
Context Graph backbone usable without moving merge mutation into a transport.

本增量直接推进条件 2 与 4。可复用 Rust/storage 层已经从 exact Context commit ancestry 解析 three-way `MergePlan`，并对持久化
graph snapshot 完成分类，但结果仍没有 local consumer。private read contract 让可回放版本历史与 Context Graph 骨架可用，同时不把
merge mutation 推入 transport。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The
`PersistedContextGraphMergeReviewService::review_server_owned` and the V1 projection are locally
verified, but no protected API, non-public local SDK, or Web `data -> presenter -> screen` reader
consumes the server-owned tips. The API repository wrapper still falls back to three independent
snapshot reads instead of delegating the concrete batch primitive, and the server-owned method
currently returns only the legacy classification, which would discard the exact resolved base and
plan if transported directly. These are root-cause prerequisites: add the minimal batch delegation
and a projection-preserving private adapter, then regress exact scope and server-owned ancestry
before exposing the reader. A caller-supplied ancestry plan must not be accepted by this consumer,
and missing or malformed snapshot data must remain fail-closed.

`PersistedContextGraphMergeReviewService::review_server_owned` 与 V1 projection 已在本地验证，但还没有 protected API、非公开
local SDK 或 Web `data -> presenter -> screen` reader 消费 server-owned tips。API repository wrapper 目前仍回退到三次独立 snapshot
read，而 server-owned method 当前只返回 legacy classification；若直接传输会丢失 exact resolved base 与 plan。这是根因级前置缺口：
先增加最小 batch delegation 与保留 projection 的 private adapter，再用 exact scope 与 server-owned ancestry regression 验证，之后才能暴露
reader。该 consumer 不得接受 caller-supplied ancestry plan；缺少或 malformed snapshot data 必须继续 fail-closed。

**Why now / 为何现在优先:** The exact commit-DAG repository contract, server-owned plan resolver,
version-bound classifier, authorization, rate-limit, audit, and no-store read boundaries already
exist. This is the smallest dependency-ready consumer for the named version/Diff and graph criteria,
and it is more direct than adding another benchmark or mutation surface.

exact commit-DAG repository contract、server-owned plan resolver、version-bound classifier、authorization、rate-limit、audit 与 no-store
read boundary 已存在。本增量是当前已命名 version/Diff 与 graph criteria 最小的依赖就绪 consumer，比新增 benchmark 或 mutation surface
更直接。

**Smallest prerequisite repair / 最小前置修复:** Override
`CommitGraphSnapshotRepository::get_commit_graph_snapshot_batch` in the API's
`WorkspaceDataRepository` so the concrete Memory/PostgreSQL adapter primitive is preserved. Add a
private storage method that returns the existing
`VersionedContextGraphMergeReviewProjectionV1` after server-owned ancestry resolution, while
keeping the legacy classification method source-compatible. Focused tests must prove the batch
delegation and that the serialized projection retains the resolved three-way plan and exact base
scope. This repair does not add a new algorithm or a new persistence model.

在 API 的 `WorkspaceDataRepository` 中覆写 `CommitGraphSnapshotRepository::get_commit_graph_snapshot_batch`，保留具体 Memory/PostgreSQL
adapter 的 batch primitive。增加一个返回既有 `VersionedContextGraphMergeReviewProjectionV1` 的 private storage method，用于 server-owned ancestry
resolution，同时保持 legacy classification method 的 source compatibility。focused tests 必须证明 batch delegation，以及 serialized projection 保留已解析的
three-way plan 与 exact base scope。本修复不新增算法或 persistence model。

**Shared contract / 共享契约:** Add only the private GET
`/api/v1/local/projects/{project_id}/contexts/{context_id}/merge-review` with required distinct
`left_commit_id` and `right_commit_id` query fields. The server derives the base and `MergePlan`
from the exact Context DAG. The response is the existing serialized
`VersionedContextGraphMergeReviewProjectionV1` V1 shape: explicit schema version, plan, exact
base/left/right `(project_id, context_id, commit_id)` scopes, and deterministic `clean`,
`equivalent`, or `conflict` classification. Clients parse the frozen response and never calculate
graph differences.

仅增加 private GET
`/api/v1/local/projects/{project_id}/contexts/{context_id}/merge-review`，要求 distinct 的 `left_commit_id` 与 `right_commit_id` query
字段。server 从 exact Context DAG 派生 base 与 `MergePlan`。response 复用已存在的序列化
`VersionedContextGraphMergeReviewProjectionV1` V1 shape：显式 schema version、plan、exact
`(project_id, context_id, commit_id)` base/left/right scope，以及确定性的 `clean`、`equivalent` 或 `conflict` classification。client 只
解析 frozen response，不计算 graph difference。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:**
The API route and focused contract test belong to `server/api`; the strict parser/client and tests
belong to `packages/local-sdk`; the same-origin BFF, presenter, screen, and tests belong to
`apps/web/src/app`. This plan and the bilingual roadmap/audit receipt are the documentation
boundary. No worker may edit another ownership set.

API route 与 focused contract test 归 `server/api`；strict parser/client 与 tests 归 `packages/local-sdk`；同源 BFF、presenter、screen 与
tests 归 `apps/web/src/app`。本计划与双语 roadmap/audit receipt 构成文档边界。任何 worker 不得编辑其他 ownership set。

**Explicit non-goals / 明确非目标:** No merge writer, branch mutation, rollback, conflict
resolution, public REST/OpenAPI/public SDK method, Web mutation, persistence schema or migration,
provider call, raw private content, secret, operator transport, Docker/PostgreSQL runtime claim,
authenticated browser claim, remote CI, release, production work, or second `GraphDiff` calculator.

不实现 merge writer、branch mutation、rollback、conflict resolution、public REST/OpenAPI/public SDK method、Web mutation、persistence schema
或 migration、provider call、raw private content、secret、operator transport、Docker/PostgreSQL runtime claim、authenticated browser claim、
remote CI、release、production work 或第二个 `GraphDiff` calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Observe
red/green focused API, local SDK, and Web tests; `pnpm check:web`; workspace Rust tests; format;
strict offline Clippy; locked Rust 1.85; public-surface exclusion; and
`GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL/Docker runtime, browser, Git, remote CI, operator, release,
and production remain `unobserved` or `deferred`.

先真实观察 focused API、local SDK 与 Web red/green tests、`pnpm check:web`、workspace Rust tests、format、strict offline Clippy、锁定 Rust 1.85、
public-surface exclusion 与 `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL/Docker runtime、browser、Git、remote CI、operator、release 与 production
继续为 `unobserved` 或 `deferred`。

## Ownership and checklist / Ownership 与清单

- [x] Add and test the projection-preserving storage adapter and concrete batch delegation before transport.
- [x] Add the protected private API route and server-owned repository/service delegation in `server/api`.
- [x] Add the strict non-public local SDK V1 parser/client in `packages/local-sdk`.
- [x] Add the same-origin Web BFF and shared `data -> presenter -> screen` merge review states.
- [x] Mount the gated Web inspector from the Context workspace with exact left/right commit selection and typed V1 resource input.
- [x] Run fresh cross-stack verification and record the bilingual receipt without closing the long-term goal.

- [x] 在 transport 之前增加并测试保留 projection 的 storage adapter 与 concrete batch delegation。
- [x] 在 `server/api` 增加 protected private API route 与 server-owned repository/service delegation。
- [x] 在 `packages/local-sdk` 增加 strict non-public local SDK V1 parser/client。
- [x] 增加同源 Web BFF 与 shared `data -> presenter -> screen` merge review states。
- [x] 从 Context workspace 挂载 gated Web inspector，使用 exact left/right commit selection 与 typed V1 resource input。
- [x] 运行新鲜跨栈验证并记录双语回执，不关闭长期目标。

## Inspector closure receipt / Inspector 收束回执

The Web gap was real: the merge-review data, presenter, and screen existed but were unreachable
from the mounted workspace. `LocalContextMergeReviewInspector` now owns only request-memory Bearer
input, exact left/right commit selection, loading/error/empty/unavailable state mapping, and stale
request cancellation. `ContextWorkspaceScreen` mounts it behind the independent
`CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW=true` gate; the default remains disabled. Candidate
or Context changes reset the selected pair and resource, and the resource input no longer accepts
an untyped `review` value: it accepts a normalized V1 DTO or a structured wire record that is
parsed once at the boundary.

Web gap 的事实是：merge-review data、presenter 与 screen 虽已存在，却没有从 mounted workspace 到达用户。现在
`LocalContextMergeReviewInspector` 只负责 request-memory Bearer 输入、exact left/right commit selection、loading/error/empty/unavailable
状态映射与过期请求取消。`ContextWorkspaceScreen` 通过独立的
`CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW=true` gate 挂载，默认仍关闭。Context 或 candidate 变化会重置 selected pair 与 resource；
resource input 不再接受无类型 `review`，只接受 normalized V1 DTO 或在边界只解析一次的 structured wire record。

Fresh local verification / 新鲜本地验证:

- Red phase observed: the new inspector test failed with `ERR_MODULE_NOT_FOUND` before the implementation existed.
- Green focused Web tests and full Web suite passed; the full suite reported `260 passed` and includes the merge-review BFF coverage.
- `pnpm check:web` passed: public SDK `15`, local SDK `126`, Web `260`, TypeScript/lint, and production build.
- `cargo test --workspace --quiet` passed; storage reported `208 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline workspace Clippy, and locked Rust `1.85.0` workspace check passed.
- Static boundary checks passed: production `impl GraphDiff` count `1`; public OpenAPI/SDK merge-review hits `0`.

- 已真实观察红阶段：实现文件不存在时 inspector test 以 `ERR_MODULE_NOT_FOUND` 失败。
- Green focused Web tests 与完整 Web suite 通过；完整 suite 报告 `260 passed`，其中包含 merge-review BFF 覆盖。
- `pnpm check:web` 通过：public SDK `15`、local SDK `126`、Web `260`、TypeScript/lint 与 production build。
- `cargo test --workspace --quiet` 通过；storage 报告 `208 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline workspace Clippy 与锁定 Rust `1.85.0` workspace check 通过。
- 静态边界检查通过：production `impl GraphDiff` 数量 `1`；public OpenAPI/SDK merge-review 命中 `0`。

No new public REST/OpenAPI/public SDK method, merge writer, branch mutation, Web mutation,
migration, provider, secret access, operator transport, Docker/PostgreSQL runtime, browser,
release, or production claim was added. Docker/PostgreSQL runtime, authenticated browser, visual
smoke, and Git remain `unobserved`; remote CI, operator rehearsal, release, and production remain
`deferred`. `GraphDiff::between` remains the sole graph-diff calculator and the long-term goal
remains active.

没有新增 public REST/OpenAPI/public SDK method、merge writer、branch mutation、Web mutation、migration、provider、secret access、operator transport、
Docker/PostgreSQL runtime、browser、release 或 production 声明。Docker/PostgreSQL runtime、authenticated browser、visual smoke 与 Git 继续为
`unobserved`；remote CI、operator rehearsal、release 与 production 继续为 `deferred`。`GraphDiff::between` 仍是唯一 graph-diff calculator，
长期目标保持 active。
