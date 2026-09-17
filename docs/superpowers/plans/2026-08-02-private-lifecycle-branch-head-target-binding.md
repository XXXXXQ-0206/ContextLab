# Private Lifecycle Branch-Head Target Binding / 私有 Lifecycle Branch-Head Target Binding

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly serves Criteria 1 (Context-first graph coverage), Criterion 2 (replayable
version history), and Criterion 4 (Context Graph as the system skeleton). The private lifecycle
editor must submit a branch-head target selected from the server-owned branch-head resource, while
the existing guarded writer remains the final authority for exact-head compare-and-swap. This
preserves the charter's rule that UI adapters do not invent version or authorization semantics. /
本增量直接服务条件 1（Context-first graph coverage）、条件 2（可回放版本历史）与条件 4（Context Graph 作为系统骨架）。私有
lifecycle editor 必须提交从 server-owned branch-head resource 选择的 branch-head target，同时继续由既有 guarded writer 作为
exact-head compare-and-swap 的最终权威。这保持章程中 UI adapter 不发明 version 或 authorization semantics 的原则。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The Rust lifecycle service already carries `BranchName` and `ExpectedBranchHead` into the guarded
writer, and the server rejects stale targets. The remaining Web composition is weaker: the branch
head inspector forwards only a commit id to graph review, while the lifecycle editor derives its
expected head from an arbitrary materialized-commit candidate list and lets branch name drift
independently. A user can therefore draft against a non-head or the wrong branch, producing a
predictable conflict instead of a target-bound local workflow. The evidence gap is a focused
Web contract proving that the selected server-owned `{ branch_name, head_commit_id }` drives both
editor fields and graph review, including null/unborn and loading/error reset semantics. /
Rust lifecycle service 已将 `BranchName` 与 `ExpectedBranchHead` 交给 guarded writer，server 也会拒绝 stale target。剩余的 Web
composition 较弱：branch-head inspector 只向 graph review 转发 commit id，lifecycle editor 却从任意 materialized-commit candidate
list 推导 expected head，branch name 还可独立漂移。因此用户可能针对非 head 或错误 branch 起草，最终只得到可预期的 conflict，而不是
target-bound local workflow。证据缺口是 focused Web contract：证明 server-owned `{ branch_name, head_commit_id }` 同时驱动 editor
fields 与 graph review，并覆盖 null/unborn、loading/error reset semantics。

### Why now / 为什么现在优先

The parent-snapshot ancestry invariant and server-owned branch-head witness are now locally
verified. Binding the existing editor to that witness is the smallest remaining Context lifecycle
integration gap before adding another lifecycle consumer. It reuses the existing local SDK branch
head parser, BFF, shared design-system controls, and guarded write path without expanding the
public surface. / parent-snapshot ancestry invariant 与 server-owned branch-head witness 已完成本地验证。在增加其他 lifecycle
consumer 前，把现有 editor 绑定到该 witness 是 Context lifecycle 剩余的最小 integration gap。它复用现有 local SDK branch-head
parser、BFF、shared design-system controls 与 guarded write path，不扩大 public surface。

### Explicit non-goals / 明确非目标

- No Rust domain, storage, migration, REST, OpenAPI, public SDK, or server write-policy change.
- No new branch mutation, merge, rollback, operator transport, provider, secret access, Docker,
  PostgreSQL runtime, release, production, or external-evidence claim.
- No browser automation, visual regression, or authenticated runtime claim; local Web tests cover
  the adapter composition only.
- No second `GraphDiff` calculator and no business logic in a screen component. Existing server
  RBAC, audit, rate-limit, idempotency, branch-head CAS, and default-off lifecycle gate remain the
  only mutation path.

- 不改变 Rust domain、storage、migration、REST、OpenAPI、public SDK 或 server write policy。
- 不新增 branch mutation、merge、rollback、operator transport、provider、secret access、Docker、PostgreSQL runtime、release、production
  或外部证据声明。
- 不宣称 browser automation、visual regression 或 authenticated runtime；local Web tests 只覆盖 adapter composition。
- 不新增第二个 `GraphDiff` calculator，不把业务逻辑放进 screen component。既有 server RBAC、审计、限流、idempotency、branch-head
  CAS 与 default-off lifecycle gate 仍是唯一 mutation path。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

Only the existing private Web composition is in scope: branch-head data selection, the
`LocalBranchHeadsInspector`/`LocalBranchHeadsGraphReview` callback bridge, the lifecycle editor's
server-selected target props, and their focused tests. The data layer remains responsible for
canonical `{ branch_name, head_commit_id }` selection; presenter/screen layers only render and
transport the frozen target. No backend or SDK file is required. / 范围仅限现有 private Web composition：branch-head data selection、
`LocalBranchHeadsInspector`/`LocalBranchHeadsGraphReview` callback bridge、lifecycle editor 的 server-selected target props 与
focused tests。data layer 负责 canonical `{ branch_name, head_commit_id }` selection；presenter/screen layer 只渲染并 transport frozen
target。不需要 backend 或 SDK 文件。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

Observe a red Web contract showing that selecting a server-owned non-main branch head does not
update the lifecycle editor target, then make it green. Run focused branch-head/editor/bridge tests,
`pnpm check:web`, `cargo test --workspace --quiet --offline -j 1`, format, strict offline Clippy,
locked Rust 1.85 check, local contract verification, and the exact-one `GraphDiff` source check.
PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, and production remain ignored, unobserved, or deferred. / 先观测 Web 红回归：选择 server-owned 的非 main branch head 后，
lifecycle editor target 未更新；然后修复为 green。运行 focused branch-head/editor/bridge tests、`pnpm check:web`、
`cargo test --workspace --quiet --offline -j 1`、format、strict offline Clippy、锁定 Rust 1.85 check、local contract verification 与
exact-one `GraphDiff` source check。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、
release 与 production 继续为 ignored、unobserved 或 deferred。

## Status / 状态

`deferred / superseded by a higher-priority Rust ancestry contract`; no Web implementation was
started from this record. / `deferred / 被更高优先级的 Rust ancestry contract 取代`；本记录未启动 Web 实现。

## Deferral Note / 延期说明

Independent review found a smaller, deeper Criteria 2/4 gap: version-backed graph review accepts
an existing source and target commit without proving that the target is on the source's valid
normal first-parent replay path. That contract is a prerequisite for any trustworthy editor target
binding, so this Web adapter record is deliberately deferred rather than implemented first. The
next admitted increment is recorded separately in the Rust storage/versioning plan. / 独立审查发现更小且更深的条件 2/4 缺口：
version-backed graph review 接受已存在的 source 与 target commit，却没有证明 target 位于 source 的合法 normal first-parent replay path。
该 contract 是可信 editor target binding 的前置条件，因此本 Web adapter 记录明确延期，不先行实现。下一项准入增量单独记录在 Rust
storage/versioning plan 中。
