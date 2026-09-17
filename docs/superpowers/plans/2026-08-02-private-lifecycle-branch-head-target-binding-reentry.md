# Private Lifecycle Branch-Head Target Binding Re-entry / 私有 Lifecycle Branch-Head Target Binding 重新准入

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly serves Criteria 1 (Context-first graph coverage), Criterion 2
(replayable version history), and Criterion 4 (Context Graph as the system skeleton). The
private lifecycle editor must consume a server-owned `{ branch_name, head_commit_id }` target for
both editing and graph review. UI adapters may present and transport that target, but may not
invent branch, version, authorization, or diff semantics. `GraphDiff::between` remains the sole
graph-diff calculator.

本增量直接服务条件 1（Context-first graph coverage）、条件 2（可回放版本历史）与条件 4（Context Graph 作为系统骨架）。private
lifecycle editor 必须将 server-owned `{ branch_name, head_commit_id }` target 同时用于编辑与 graph review。UI adapter 只能呈现和传输
该 target，不能发明 branch、version、authorization 或 diff semantics。`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The Rust guarded writer, branch-head witness, local SDK parser, BFF, lifecycle editor, and graph
review primitives already exist. The remaining gap is composition: selecting a non-main server-owned
branch head does not yet prove that the editor's expected-head fields and graph-review candidate
are updated together, and null/unborn or loading/error transitions can retain stale target state.

Rust guarded writer、branch-head witness、local SDK parser、BFF、lifecycle editor 与 graph review primitives 已存在。剩余缺口是
composition：当前尚未证明选择非 main 的 server-owned branch head 会同时更新 editor expected-head fields 与 graph-review candidate，且
null/unborn 或 loading/error transition 可能保留 stale target state。

### Why now / 为什么现在优先

The parent-snapshot and normal first-parent invariants, atomic branch-head witness, and atomic
merge-review witness are locally verified. This is the smallest user-facing Context lifecycle
integration gap before another lifecycle consumer, and it reuses the existing private read path
without adding a transport or policy surface.

parent-snapshot 与 normal first-parent invariant、atomic branch-head witness 以及 atomic merge-review witness 已完成本地验证。这是
增加其他 lifecycle consumer 前最小的 user-facing Context lifecycle integration gap，并复用既有 private read path，不增加 transport 或
policy surface。

### Explicit non-goals / 明确非目标

- No Rust domain/storage/API/SDK change, migration, public REST/OpenAPI method, public write, or server policy change.
- No branch/merge/rollback mutation, provider, secret, operator transport, Docker/PostgreSQL runtime, browser/visual, release, or production claim.
- No second `GraphDiff` calculator and no business logic in screen components. Existing RBAC, audit, rate-limit, idempotency, branch-head CAS, and default-off gates remain authoritative.

- 不改变 Rust domain/storage/API/SDK、migration、public REST/OpenAPI method、public write 或 server policy。
- 不新增 branch/merge/rollback mutation、provider、secret、operator transport、Docker/PostgreSQL runtime、browser/visual、release 或 production 声明。
- 不新增第二个 `GraphDiff` calculator，不把业务逻辑放进 screen component。既有 RBAC、audit、rate-limit、idempotency、branch-head CAS 与 default-off gate 仍是最终权威。

### Smallest affected boundary and bilingual documentation / 最小边界与双语文档

Only the existing private Web composition is owned here: branch-head data selection, the
`LocalBranchHeadsInspector` to `LocalBranchHeadsGraphReview` callback bridge, lifecycle editor
target props, and their focused tests. Ownership is limited to `apps/web/src/app` files named by
the failing contract; no Rust, API, SDK, migration, or public route file may be changed.

本记录只负责既有 private Web composition：branch-head data selection、`LocalBranchHeadsInspector` 到
`LocalBranchHeadsGraphReview` 的 callback bridge、lifecycle editor target props 与 focused tests。ownership 限定于红测点名的
`apps/web/src/app` 文件；不得修改 Rust、API、SDK、migration 或 public route 文件。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe a red Web contract for selecting a server-owned non-main branch head and for
clearing stale target state on null/unborn or loading/error. Then run the focused branch-head,
editor, and bridge tests, `pnpm check:web`, workspace Rust, format, strict offline Clippy, locked
Rust 1.85, local contract verification, and the exact-one `GraphDiff` source check. Browser/visual,
PostgreSQL runtime, Docker, Git, remote CI, operator, release, and production remain explicitly
unobserved or deferred.

先观测 server-owned 非 main branch head 选择与 null/unborn 或 loading/error 清理 stale target 的 Web 红测，再运行 branch-head/editor/bridge
focused tests、`pnpm check:web`、workspace Rust、format、strict offline Clippy、锁定 Rust 1.85、local contract verification 与唯一
`GraphDiff` source check。browser/visual、PostgreSQL runtime、Docker、Git、remote CI、operator、release 与 production 继续明确为
unobserved 或 deferred。

## Status / 状态

`completed / verified locally`; the long-term goal remains `active`. / `completed / verified locally`；长期目标保持 `active`。
