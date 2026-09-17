# Private Context Lifecycle Branch-Head Binding / 私有 Context Lifecycle Branch-Head 绑定

## Boundary / 边界

This is a private Web composition contract over existing read and guarded-write paths. A
server-owned `{ branch_name, head_commit_id }` target is passed unchanged to the lifecycle editor
as its branch and expected head, and to graph review as its exact revised candidate. The Web layer
does not select a head, authorize a write, replay a commit, or calculate a diff.

这是既有读取与 guarded-write path 之上的 private Web composition contract。server-owned
`{ branch_name, head_commit_id }` target 原样传给 lifecycle editor，作为 branch 与 expected head；同时传给 graph review，作为 exact
revised candidate。Web 层不选择 head、不授权写入、不回放 commit，也不计算 diff。

## State And Scope / 状态与范围

- The branch-head inspector resets its resource and selection only when the real Context target changes; callback identity is not a data scope.
- Loading, error, empty, unavailable, and unborn states publish a null target, so the editor cannot retain a stale expected head.
- A committed lifecycle review pair is keyed by Context, materialized candidates, branch name, and exact head commit. A target transition clears the pair before composing the next review.
- `GraphDiff::between` remains the sole graph-diff calculator; the bridge only composes server-owned IDs and existing review models.

- branch-head inspector 只在真实 Context target 变化时 reset resource 与 selection；callback identity 不是 data scope。
- loading、error、empty、unavailable 与 unborn state 发布 null target，因此 editor 不能保留 stale expected head。
- committed lifecycle review pair 绑定 Context、materialized candidates、branch name 与 exact head commit；target transition 会在组合下一次 review 前清除 pair。
- `GraphDiff::between` 仍是唯一 graph-diff calculator；bridge 只组合 server-owned ID 与既有 review model。

## Compatibility And Non-Goals / 兼容性与非目标

The contract changes only `apps/web/src/app` composition and focused tests. It adds no Rust,
storage, API, SDK, OpenAPI, migration, public write, operator transport, provider, secret, Docker,
PostgreSQL runtime, browser/visual, release, or production behavior. Existing RBAC, audit,
rate-limit, idempotency, branch-head CAS, default-off mutation gate, and design-system
data-to-presenter-to-screen boundaries remain authoritative.

本 contract 只改变 `apps/web/src/app` composition 与 focused tests。未新增 Rust、storage、API、SDK、OpenAPI、migration、public write、
operator transport、provider、secret、Docker、PostgreSQL runtime、browser/visual、release 或 production behavior。既有 RBAC、audit、
rate-limit、idempotency、branch-head CAS、default-off mutation gate 与 design-system 的 data-to-presenter-to-screen boundary 仍是权威。

## Verification / 验证

Fresh local evidence is recorded in the roadmap and completion audit: red `22 passed, 2 failed`,
green focused Web `33/33`, full `pnpm check:web` `15/148/304` plus production build, workspace
Rust storage `238 passed, 41 ignored`, fmt, strict offline Clippy, locked Rust `1.85.0`, local
contract verifier, fixture verifier, one production GraphDiff implementation, ten call sites, and
zero public merge-review surface hits. PostgreSQL runtime is ignored without
`CONTEXTLAB_TEST_DATABASE_URL`; external release and production evidence remains deferred.

新鲜本地证据已记录在路线图与完成审计：红测 `22 passed, 2 failed`、focused Web 绿测 `33/33`、完整 `pnpm check:web` `15/148/304` 与
production build、workspace Rust storage `238 passed, 41 ignored`、fmt、strict offline Clippy、锁定 Rust `1.85.0`、local contract verifier、
fixture verifier、一个 production GraphDiff implementation、十个 call site 与零 public merge-review surface hits。缺少
`CONTEXTLAB_TEST_DATABASE_URL` 时 PostgreSQL runtime 为 ignored；外部 release 与 production evidence 继续延期。
