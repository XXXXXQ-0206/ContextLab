# Private Context Lifecycle Graph-Diff Review / 私有 Context 生命周期图谱 Diff 审阅

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** A Context lifecycle
commit must be immediately reviewable as a versioned change. After a successful local create,
content update, removal, or relationship operation, the Web workflow must preserve the exact prior
head and new commit as a graph-diff pair. This directly advances Criteria 1, 2, 4, 5, and 9 while
keeping Context, replayable history, the shared Context Graph, and the design-system boundary primary.

**完成条件与章程原则：** Context lifecycle commit 必须可以立即作为版本化变更审阅。local create、content update、removal 或
relationship operation 成功后，Web workflow 必须保留精确的旧 head 与新 commit 作为 graph-diff pair。本增量直接推进条件
1、2、4、5 与 9，同时保持 Context、可回放历史、共享 Context Graph 与 design-system boundary 为首要骨架。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The guarded lifecycle
writer already returns the new commit and the editor already knows the expected head, but the
success path only reloads lifecycle state and workspace data. Users therefore cannot verify the
exact graph change produced by their own committed operation from the same workflow. The existing
private graph-diff read, local SDK adapter, presenter, and review screen are already available;
the gap is only their composition after commit success.

**未满足依赖、风险或证据缺口：** guarded lifecycle writer 已返回新 commit，editor 也已知道 expected head，但成功路径只会
重新加载 lifecycle state 与 workspace data。因此用户无法在同一 workflow 中审阅自己刚提交 operation 产生的 exact graph change。
现有 private graph-diff read、local SDK adapter、presenter 与 review screen 均已具备；缺口仅在 commit success 后的组合。

**Why now / 为何现在优先:** This is the smallest dependency-ready local increment after the
Workflow binding interaction receipt. It turns the already verified Context lifecycle write and
version-backed GraphDiff read into one usable Context-first workflow, closer to Criteria 1 and 4
than adding another storage-only contract or provider surface.

**为何现在优先：** 这是 Workflow binding interaction receipt 之后最小且依赖已满足的本地增量。它把已验证的 Context lifecycle
write 与 version-backed GraphDiff read 组合为一个可用的 Context-first workflow，相比再增加 storage-only contract 或 provider
surface 更直接服务条件 1 与 4。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Only the
Web lifecycle editor composition, its focused tests, the existing graph-review composition props,
and this plan/roadmap receipt are affected. The existing private BFF route, local SDK transport,
server authorization/rate-limit/audit, guarded writer, branch-head CAS, idempotency, presenter,
screen, and shared design-system primitives remain the only implementation paths.

**最小受影响边界与双语文档：** 仅影响 Web lifecycle editor composition、其 focused tests、既有 graph-review composition props
与本 plan/roadmap receipt。既有 private BFF route、local SDK transport、server authorization/rate-limit/audit、guarded writer、
branch-head CAS、idempotency、presenter、screen 与 shared design-system primitives 仍是唯一实现路径。

**明确非目标：** No new Rust domain, storage port, migration, REST route, OpenAPI operation, SDK
write method, public graph-diff read, Web mutation, merge/rollback writer, provider call, raw
payload, credential persistence, Docker/PostgreSQL runtime, browser automation, release claim,
production claim, or second `GraphDiff` calculator is added.

**Non-goals / 明确非目标：** 不新增 Rust domain、storage port、migration、REST route、OpenAPI operation、SDK write method、
public graph-diff read、Web mutation、merge/rollback writer、provider call、raw payload、credential persistence、
Docker/PostgreSQL runtime、browser automation、release claim、production claim 或第二个 `GraphDiff` calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Focused Web
tests must prove create/update/removal/relationship success selects the exact old-head to new-commit
pair, replay does not fabricate a second pair, stale-head conflict does not request a diff, and
the existing scope/error/no-store contract remains intact. Then run `pnpm check:web`, Rust workspace
tests, format, strict offline Clippy, locked Rust `1.85.0` check, and static `GraphDiff` singularity
inspection. Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`.

**下一增量前的新鲜验证：** focused Web tests 必须证明 create/update/removal/relationship success 会选择精确的旧 head 到新
commit pair，replay 不会伪造第二个 pair，stale-head conflict 不会请求 diff，且既有 scope/error/no-store contract 保持不变。
随后运行 `pnpm check:web`、Rust workspace tests、format、strict offline Clippy、锁定 Rust `1.85.0` check 与 static
`GraphDiff` singularity inspection。Docker/PostgreSQL runtime、authenticated browser、Git、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`。

## Implementation Checklist / 实施清单

- [x] Capture the exact pre-commit head and post-commit commit in the editor success state.
- [x] Compose the existing graph-review screen without duplicating data loading or diff logic.
- [x] Add focused regressions for create/update/removal/relationship and stale-head/conflict paths.
- [x] Record fresh local verification and keep the long-term goal active.

- [x] 在 editor success state 捕获精确的 commit 前 head 与 commit 后新 commit。
- [x] 组合既有 graph-review screen，不重复 data loading 或 diff logic。
- [x] 为 create/update/removal/relationship 与 stale-head/conflict path 增加 focused regression。
- [x] 记录新鲜本地验证并保持长期目标 active。

## Fresh Verification Receipt / 新鲜验证回执

The implementation is complete within the admitted Web composition boundary. `ContextLifecycleEditor`
captures the selected pre-commit head and returned commit, rejects an empty or same-commit pair, and
passes the pair to `ContextLifecycleGraphReviewBridge`. The bridge composes the existing
`LocalBranchHeadsGraphReview`; `composeCommittedGraphReview` adds the exact two candidates, selects
them as original/revised, clears any prior result, and remounts on Context or candidate identity
changes. The editor also scopes its idempotency key to a draft version, reusing it for a retry of
the unchanged request and rotating it after draft changes or a successful response. No loader,
presenter, screen, GraphDiff algorithm, transport, or public surface was duplicated.

本实现已在准入的 Web composition boundary 内完成。`ContextLifecycleEditor` 捕获选中的 commit 前 head 与返回的 commit，
拒绝空 pair 或 same-commit pair，并将 pair 传给 `ContextLifecycleGraphReviewBridge`。bridge 组合既有
`LocalBranchHeadsGraphReview`；`composeCommittedGraphReview` 加入 exact 两个 candidate、将其选为 original/revised、清除旧
result，并在 Context 或 candidate identity 变化时重新挂载。editor 还将 idempotency key 绑定到 draft version：未改变的 request
重试会复用 key，draft 变化或成功响应后会轮换 key。没有重复 loader、presenter、screen、GraphDiff algorithm、transport 或
public surface。

Fresh focused Web evidence passed `context-lifecycle-editor.test.tsx` `7/7` and
`local-branch-heads-graph-review.test.tsx` `5/5`.
The unified local verification also passed `pnpm check:web` (public SDK `15`, local SDK `99`, Web
`207`, TypeScript/lint, and production build), `cargo test --workspace --quiet --no-fail-fast`
(storage `193 passed, 39 ignored`), `cargo fmt --all -- --check`, strict offline workspace Clippy,
and locked Rust `1.85.0` check. Static inspection observed `impl GraphDiff` count `1`.

新鲜 focused Web evidence 已通过 `context-lifecycle-editor.test.tsx` `7/7` 与
`local-branch-heads-graph-review.test.tsx` `5/5`。统一本地验证也已通过
`pnpm check:web`（public SDK `15`、local SDK `99`、Web `207`、TypeScript/lint 与 production build）、
`cargo test --workspace --quiet --no-fail-fast`（storage `193 passed, 39 ignored`）、`cargo fmt --all -- --check`、strict
offline workspace Clippy 与锁定 Rust `1.85.0` check。静态 inspection 观测到 `impl GraphDiff` count `1`。

Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. The local mutation remains server-gated
and default-off; no public REST/OpenAPI/SDK write method, operator transport, migration, secret,
provider, or production-readiness claim was added. The long-term goal remains active; the next
increment requires a new bilingual Necessity Record.

Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`。local mutation 仍由 server gate 控制且默认关闭；没有新增 public REST/OpenAPI/SDK write method、
operator transport、migration、secret、provider 或 production-readiness 声明。长期目标保持 active；下一增量必须先新增双语
Necessity Record。
