# Private Branch-Head Error Redaction / 私有 Branch-Head 错误脱敏

> **Admission status / 准入状态:** Audited but not admitted for implementation. Two independent
> Luna reviews identified the higher-priority Criterion 2 lifecycle metadata propagation gap;
> this lower-priority UI hardening remains queued for a future Necessity Record after the current
> Context version/diff correctness increment. / 已审计但未准入实施。两名独立 Luna review 识别出更高优先级的 Criterion 2 lifecycle metadata propagation 缺口；本较低优先级 UI hardening 留待当前 Context version/diff 正确性增量之后重新通过 Necessity Record 排队。

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与章程原则

- **Criterion 4 / 条件 4:** The private version and graph read chain must preserve a fail-closed,
  redacted error contract across the protected route, local SDK, Web data adapter, presenter, and
  screen.
  / **条件 4：** 私有 version 与 graph read chain 必须在 protected route、local SDK、Web data adapter、presenter 与 screen 之间保持 fail-closed、脱敏错误 contract。
- **Security and Context-first principle / 安全与 Context-first 原则：** A Context version
  selection surface must not expose upstream diagnostics or private payloads while selecting an
  exact branch head for replay or graph review.
  / Context version selection surface 在为 replay 或 graph review 选择 exact branch head 时，不得暴露 upstream diagnostics 或 private payload。

### Gap, dependency, and evidence / 缺口、依赖与证据

The protected branch-head route and the local SDK already normalize upstream errors, but
`apps/web/src/app/local-branch-heads-data.ts` still copies a structured upstream `message` into
`LocalBranchHeadsProxyError`. The shared presenter can then render that untrusted message in the
error state. The branch-head route, parser, presenter, and focused tests already exist, so this is
a dependency-ready adapter hardening slice with no new transport.

受保护 branch-head route 与 local SDK 已经规范化 upstream error，但
`apps/web/src/app/local-branch-heads-data.ts` 仍会将 structured upstream `message` 复制到
`LocalBranchHeadsProxyError`，共享 presenter 随后可能在 error state 中渲染该不可信 message。branch-head route、parser、presenter 与 focused test 均已存在，
因此这是一个不需要新 transport 的依赖就绪 adapter hardening slice。

### Why now / 为何现在优先

The adjacent graph-diff and persisted Context-diff Web adapters have just been hardened against
the same defect. Branch-head discovery feeds exact version selection and the existing graph-review
bridge, so leaving this symmetric boundary open would let a diagnostic leak survive on the same
Context version workflow. This is smaller and more directly tied to the named security criterion
than adding another graph, benchmark, or public API surface.

相邻的 graph-diff 与 persisted Context-diff Web adapter 刚完成同类缺陷的硬化。branch-head discovery 为 exact version selection 与既有 graph-review bridge 提供输入；继续保留这个对称边界会让 diagnostic leak 留在同一条 Context version workflow 中。相比新增 graph、benchmark 或 public API surface，本增量更小且更直接服务命名安全条件。

### Explicit non-goals / 明确非目标

- No Rust domain or storage change, route, OpenAPI operation, public SDK method, write, migration,
  provider, operator transport, or second `GraphDiff` calculator.
  / 不修改 Rust domain 或 storage，不新增 route、OpenAPI operation、public SDK method、write、migration、provider、operator transport 或第二个 `GraphDiff` calculator。
- No change to branch-head scope validation, capability states, retry behavior, or known bilingual
  screen copy beyond stable error normalization.
  / 除稳定错误规范化外，不改变 branch-head scope validation、capability state、retry behavior 或既有双语 screen copy。
- No secrets, external runtime, Docker/PostgreSQL, browser, Git, remote CI, release, or production
  evidence claim.
  / 不读取 secrets，不声称 external runtime、Docker/PostgreSQL、browser、Git、remote CI、release 或 production 证据。

### Smallest boundary and ownership / 最小边界与所有权

- Luna owns only `apps/web/src/app/local-branch-heads-data.ts` and
  `apps/web/src/app/local-branch-heads.test.tsx`.
  / Luna 仅负责上述两个 adapter/test 文件。
- Integration Lead owns this plan, bilingual roadmap receipts, and final verification.
  / Integration Lead 负责本计划、双语 roadmap 回执与最终验证。

### Fresh verification required / 所需新鲜验证

First observe a red regression where an unknown-status structured upstream message is returned by
the branch-head data adapter. Then make it green while preserving the typed status/code boundary.
Run the focused branch-head tests, `pnpm check:web`, the scoped local contract verifier,
`GRAPH_DIFF_IMPL_COUNT=1`, Rust format/workspace/strict offline Clippy/MSRV checks, and record
Docker/PostgreSQL, browser, Git, and external release evidence as unobserved or deferred.

先观测 unknown-status structured upstream message 被 branch-head data adapter 返回的 red regression，再在保持 typed status/code boundary 的同时修复为 green。运行 focused branch-head tests、`pnpm check:web`、scoped local contract verifier、`GRAPH_DIFF_IMPL_COUNT=1`、Rust format/workspace/strict offline Clippy/MSRV checks，并将 Docker/PostgreSQL、browser、Git 与 external release evidence 记录为 `unobserved` 或 `deferred`。

## Execution Checklist / 执行清单

- [ ] Observe the red raw-message regression and implement stable bilingual adapter redaction.
  / 观测 raw-message 红回归并实现稳定双语 adapter 脱敏。
- [ ] Preserve structured status/code and existing scope, retry, presenter, and screen behavior.
  / 保持 structured status/code 与既有 scope、retry、presenter、screen behavior。
- [ ] Run fresh focused and workspace verification, update bilingual receipts, and keep the goal active.
  / 运行新鲜聚焦与全量验证，更新双语回执，并保持长期目标 active。
