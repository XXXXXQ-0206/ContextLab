# Private Branch-Head Graph Review Selection / 私有 Branch-Head 图谱审阅选择

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** This increment
directly advances the Context-first, replayable version-history and graph-review criteria. A
branch-head read that cannot feed the existing exact commit-pair review remains a disconnected
inspection. The smallest useful consumer is a private client composition that makes the selected
branch head an explicit revised-commit candidate while continuing to delegate comparison to the
existing server-owned `GraphDiff::between` path.

**完成条件与章程原则：** 本增量直接推进 Context-first、可回放版本历史与图谱审阅条件。Branch-head read 如果不能进入既有
exact commit-pair review，就仍是割裂的 inspection。最小可用 consumer 是一个 private client composition，把选中的 branch
head 作为明确的 revised-commit candidate，同时继续把比较委托给既有 server-owned `GraphDiff::between` path。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The typed branch-head
projection and private graph-diff read are already locally verified, but the Web currently keeps
branch selection and graph-review selection in separate component state. A user can see a head
commit but cannot use that exact server-owned identity in the existing review workflow. The risk
is stale or manually retyped commit selection.

**未满足依赖、风险或证据缺口：** typed branch-head projection 与 private graph-diff read 已完成本地验证，但 Web 目前把
branch selection 与 graph-review selection 分置于两个组件状态。用户可以看到 head commit，却不能在既有 review workflow 中使用
这个 server-owned exact identity，风险是 stale 或手工重新输入 commit selection。

**Why now / 为何现在优先:** Branch-head discovery is the immediately preceding verified
increment and graph review already accepts exact Context/original/revised commit scope. This
consumer closes the shortest remaining usability gap for Criteria 1, 2, and 4 before branch
mutation, merge, rollback, or new benchmark/provider work.

**为何现在优先：** Branch-head discovery 是刚刚完成验证的前置增量，而 graph review 已接受 exact Context/original/revised
commit scope。本 consumer 是条件 1、2、4 在进入 branch mutation、merge、rollback 或新的 benchmark/provider work 前最短的
剩余可用性缺口。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Add
one Web client composition, a narrow callback from the existing branch-head inspector, and
pure presenter/contract tests. Reuse the existing branch-head parser, graph-diff loader,
`CommitGraphReview`, shared UI primitives, and `data -> presenter -> screen` boundary. Update
this plan and the four roadmap receipts with observed local evidence only.

**最小受影响边界与双语文档：** 只新增一个 Web client composition、现有 branch-head inspector 的窄 callback，以及 pure
presenter/contract tests。复用既有 branch-head parser、graph-diff loader、`CommitGraphReview`、shared UI primitive 与
`data -> presenter -> screen` boundary。只用实际观测到的本地证据更新本计划与四份 roadmap receipt。

**Explicit non-goals / 明确非目标:** No new REST route, OpenAPI operation, public or local SDK
method, branch mutation, merge/rollback, server-side selection policy, second GraphDiff
calculator, provider call, credential persistence, raw content, migration, Docker/PostgreSQL
runtime, browser E2E, remote CI, operator rehearsal, release, or production claim.

**明确非目标：** 不新增 REST route、OpenAPI operation、public 或 local SDK method、branch mutation、merge/rollback、server-side
selection policy、第二个 GraphDiff calculator、provider call、credential persistence、raw content、migration、Docker/PostgreSQL
runtime、browser E2E、remote CI、operator rehearsal、release 或 production claim。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Run
focused Web composition/selection tests, full Web TypeScript and test suite, production build,
`pnpm check:web`, Rust format/workspace tests, strict offline Clippy, locked Rust check, and the
single-`GraphDiff`/public-surface inspection. Keep runtime and external release evidence labels
unchanged.

**下一增量前的新鲜验证：** 运行 focused Web composition/selection test、完整 Web TypeScript 与 test suite、production build、
`pnpm check:web`、Rust format/workspace test、strict offline Clippy、locked Rust check 与 single-`GraphDiff`/public-surface
inspection。runtime 与 external release evidence 的标签保持不变。

## Implementation Checklist / 实施清单

- [x] Add the callback and a client composition that maps a selected non-null branch head to the
      existing graph-review revised-commit candidate.
- [x] Preserve preview synthetic IDs, unborn/null heads, the existing selection remount reset, and exact
      request-memory credential/no-cookie/no-store transport.
- [x] Add focused bilingual contract tests and run the admitted verification matrix.
- [x] Update active goal, parallel plan, completion criteria, and this plan with fresh receipts;
      keep the long-term goal active.

- [x] 增加 callback 与 client composition，把非 null selected branch head 映射为既有 graph-review revised-commit candidate。
- [x] 保持 preview synthetic ID、unborn/null head、既有 selection remount reset 与 exact request-memory credential/no-cookie/no-store transport。
- [x] 增加 focused 双语 contract test 并运行准入的验证矩阵。
- [x] 用新鲜 receipt 更新 active goal、parallel plan、completion criteria 与本计划；保持长期目标 active。

## Fresh Verification Receipt / 新鲜验证回执

**Status / 状态:** `completed / verified locally`. The implementation is limited to
`apps/web/src/app/local-branch-heads-graph-review.tsx`, its focused tests, the existing branch-head
inspector callback, and workspace wiring. A non-null server-owned branch head becomes the exact
revised graph-review candidate; existing candidates are not duplicated and an unborn/null head
leaves the review defaults unchanged. The keyed review composition resets the existing review
state when the Context or selected head changes. No parser, transport, server policy, or Diff
algorithm was duplicated.

**状态：** `completed / verified locally`。实现边界仅限于
`apps/web/src/app/local-branch-heads-graph-review.tsx`、其 focused tests、既有 branch-head inspector callback 与
workspace wiring。非 null 的 server-owned branch head 会成为 exact revised graph-review candidate；既有 candidate 不会
重复添加，unborn/null head 保持原 review defaults。带 key 的 review composition 会在 Context 或 selected head 变化时重置
既有 review state。没有复制 parser、transport、server policy 或 Diff algorithm。

**Observed local evidence / 已观测本地证据:**

- `pnpm --filter @contextlab/web exec tsx --test src/app/local-branch-heads-graph-review.test.tsx src/app/local-branch-heads.test.tsx`: `8 passed`.
- `pnpm check:web`: public SDK `15 passed`, local SDK `99 passed`, Web `200 passed`, TypeScript/lint, and production build passed.
- `cargo fmt --all -- --check`, `cargo test --workspace --quiet --no-fail-fast` (storage `193 passed, 39 ignored`), `cargo clippy --workspace --all-targets --offline -- -D warnings`, and `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- Static inspection: `impl GraphDiff` count `1`; public SDK branch-head hits `0`; public retired commit-graph-diff read hits `0`.

**已观测本地证据：** 上述 focused Web `8 passed`、`pnpm check:web` 的 public SDK `15`、local SDK `99`、Web `200` 与 production
build，以及 Rust format、workspace `193 passed, 39 ignored`、strict offline Clippy 与 locked check 均真实通过。静态检查仍为
`impl GraphDiff` count `1`；public SDK branch-head hit 为 `0`，retired public commit-graph-diff read hit 为 `0`。

**Boundary / 边界:** This remains a private, read-only local composition. No REST/OpenAPI/public
SDK method, branch mutation, merge/rollback, Web mutation, provider, migration, secret,
Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
release, or production evidence was added. The long-term goal remains `active`; the next increment
requires a new bilingual Necessity Record.

**边界：** 本切片仍是 private、read-only local composition。没有新增 REST/OpenAPI/public SDK method、branch mutation、
merge/rollback、Web mutation、provider、migration、secret、Docker/PostgreSQL runtime、authenticated browser、Git change-set、
remote CI、operator rehearsal、release 或 production evidence。长期目标保持 `active`；下一增量必须先新增双语 Necessity Record。
