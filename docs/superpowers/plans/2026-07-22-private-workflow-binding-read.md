# Private Workflow Binding Read Plan / 私有 Workflow Binding 读取计划

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment directly advances Criterion 1 (Workflow persistence/API/SDK/UI inspection), Criterion 2 (commit-scoped replay), Criterion 4 (shared Context Graph provenance), Criterion 6 (exact authorization), and Criterion 9 (bilingual documentation). It turns the now-persisted private source binding into a local read workflow without exposing mutation or public transport.

本增量直接推进条件 1（Workflow persistence/API/SDK/UI inspection）、条件 2（commit-scoped replay）、条件 4（共享 Context Graph 溯源）、条件 6（精确授权）与条件 9（双语文档）。它将已持久化的私有 source binding 转为本地读取工作流，但不暴露 mutation 或 public transport。

**Unmet gap / 未满足缺口：** The repository can store and retrieve exact bindings, but `AppState` has no binding repository, the protected API has no binding read route, the local SDK cannot parse the binding summary, and Web has no inspection presenter/screen. The existing workflow capability route therefore remains default-unavailable and cannot prove Context-to-Workflow provenance.

repository 已可存取精确 binding，但 `AppState` 没有 binding repository，protected API 没有 binding read route，local SDK 不能解析 binding summary，Web 也没有 inspection presenter/screen。因此现有 workflow capability route 仍 default-unavailable，无法证明 Context 到 Workflow 的溯源。

**Why now / 为什么现在做：** The typed binding contract, memory/PostgreSQL adapters, immutable graph snapshot prerequisite, exact source scope, and fresh Rust/Web verification are complete. A private read is the smallest next consumer and directly closes the transport/UI inspection gap named by Criterion 1; workflow execution, mutation, or benchmark expansion would depend on this provenance and would be premature.

typed binding contract、memory/PostgreSQL adapter、不可变 graph snapshot 前置、精确 source scope 与新鲜 Rust/Web 验证均已完成。私有 read 是最小的下一消费者，直接收束条件 1 指定的 transport/UI inspection gap；Workflow execution、mutation 或 benchmark 扩展都依赖该溯源，当前启动会过早。

**Non-goals / 非目标：** No public REST/OpenAPI/public SDK method, write route, Web mutation, workflow execution/provider call, branch/merge/rebind lifecycle, raw workflow payload or credentials. `GraphDiff::between` remains the sole graph-diff calculator. Docker/PostgreSQL runtime, browser E2E, remote CI, operator rehearsal, release and production evidence remain deferred/unobserved.

不包含 public REST/OpenAPI/public SDK method、write route、Web mutation、Workflow execution/provider call、branch/merge/rebind lifecycle、raw workflow payload 或 credential。`GraphDiff::between` 仍是唯一 graph-diff calculator。Docker/PostgreSQL runtime、browser E2E、remote CI、operator rehearsal、release 与 production evidence 仍 deferred/unobserved。

**Smallest boundary and bilingual docs / 最小边界与双语文档：** Add one server-owned versioned redacted summary DTO and optional repository wiring; one protected local GET scoped by path `ContextId` and exact `commit_id`; one fail-closed local SDK parser/client; and one `data -> presenter -> screen` Web inspector using shared capability primitives. The route must authorize the path Context with `ContextPermission::Read` before repository access and pass the same Context/commit scope unchanged.

新增一个 server-owned、带版本的脱敏 summary DTO 与 optional repository wiring；一条以 path `ContextId` 和精确 `commit_id` 为 scope 的 protected local GET；一个 fail-closed local SDK parser/client；以及一个使用 shared capability primitive 的 `data -> presenter -> screen` Web inspector。route 必须在 repository access 前对 path Context 执行 `ContextPermission::Read`，并原样传递相同 Context/commit scope。

**Fresh verification / 新鲜验证：** Before the next increment, observe red/green tests for denied Context reads, unavailable repository, exact commit scope, parser schema rejection, no raw fields, and all Web states. Then run `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm check:web`, focused API/local-SDK/Web tests, and public-surface searches proving no public mutation/OpenAPI drift.

在开始下一增量前，必须观察到 denied Context read、repository unavailable、exact commit scope、parser schema rejection、无 raw field 与全部 Web state 的红绿测试。随后运行 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、`pnpm check:web`、聚焦 API/local-SDK/Web test，以及证明无 public mutation/OpenAPI drift 的 public-surface search。

## Shared Read Contract / 共享读取契约

- Schema: `contextlab.local-workflow-context-bindings.v1`.
- Scope: `GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings`.
- Safe response: `{ schema_version, context_id, commit_id, bindings: [{ binding_id, workflow_id, workflow_revision, node_count, edge_count }] }`.
- Ordering: `bindings` is the storage-defined `(workflow_id, workflow_revision, binding_id)` order.
- Disabled state: absent repository returns typed `unavailable`; no fallback to preview/current head.

## Current Evidence Ledger / 当前证据台账

The following labels separate observed source state from executable receipts. `passed` means the command was run in this Docs/QA handoff; `in progress` means code is present or being integrated but a required check is still red; `unobserved` means no receipt exists; `deferred` is reserved for external or explicitly out-of-scope gates.

以下标签区分已观测源码状态与可执行回执。`passed` 只表示本次 Docs/QA handoff 实际运行过对应 command；`in progress` 表示代码已出现或正在集成，但必需检查仍为红灯；`unobserved` 表示没有回执；`deferred` 仅用于外部或明确不在范围内的门禁。

| Boundary / 边界 | Evidence label / 证据标签 | Observed fact / 已观测事实 |
| --- | --- | --- |
| `server/api` DTO and protected route / DTO 与 protected route | `passed` | Source contains `LocalWorkflowContextBindingsResponse` and the exact protected route. `cargo test -p contextlab-api local_workflow_bindings --quiet` returned `4 passed`. / 源码包含 `LocalWorkflowContextBindingsResponse` 与精确 protected route；该 command 返回 `4 passed`。 |
| `packages/local-sdk` parser/client / parser/client | `passed` | The exact schema, scope, ordering, redaction, and request-scoped bearer tests ran with `35 passed`. / 精确 schema、scope、ordering、脱敏与 request-scoped bearer 测试实际运行并返回 `35 passed`。 |
| Web `data -> presenter -> screen` and inspector / Web data -> presenter -> screen 与 inspector | `passed` | Fresh focused Web test returned `84 passed` and `pnpm --filter @contextlab/web lint` passed. The inspector is anchored to the selected exact commit, uses shared capability states, and rejects scope drift before presentation. / 新鲜聚焦 Web test 返回 `84 passed`，`pnpm --filter @contextlab/web lint` 通过。inspector 锚定选定的精确 commit，使用 shared capability states，并在 presentation 前拒绝 scope drift。 |
| Web typecheck / Web 类型检查 | `passed` | `pnpm --filter @contextlab/web lint` (`tsc --noEmit`) passed. / `pnpm --filter @contextlab/web lint`（`tsc --noEmit`）通过。 |
| Same-origin BFF / 同源 BFF | `passed` | The exact route and raw-field fail-closed regression test returned `7 passed`; the default Web glob does not discover this nested route test, so the receipt is recorded as focused. / 精确 route 与 raw-field fail-closed 回归 test 返回 `7 passed`；默认 Web glob 不发现嵌套路由 test，因此该回执明确记录为 focused。 |
| Full cross-stack check / 全栈检查 | `passed` | Fresh `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, and `pnpm check:web` passed. The Web command reports TS SDK `14`, local SDK `35`, Web `84`, and a production build; storage reports `166 passed, 36 ignored`. / 新鲜的 `cargo fmt --all -- --check`、`cargo test --workspace --quiet` 与 `pnpm check:web` 通过。Web command 报告 TS SDK `14`、local SDK `35`、Web `84` 与 production build；storage 报告 `166 passed, 36 ignored`。 |
| Public/release/production and external runtime / public、release、production 与外部 runtime | `deferred` / `unobserved` | No public REST/OpenAPI/public SDK write, Docker/PostgreSQL runtime, authenticated browser E2E, remote CI, operator rehearsal, release, or production evidence is claimed. / 不声称 public REST/OpenAPI/public SDK write、Docker/PostgreSQL runtime、authenticated browser E2E、remote CI、operator rehearsal、release 或 production evidence。 |

## Tasks / 任务

- [x] Freeze server DTO and protected route contract with red tests; focused API receipt is `passed` (`4 passed`).
- [x] Wire local SDK parser/client and focused contract tests; focused receipt is `passed` (`35 passed`).
- [x] Repair and verify Web `data -> presenter -> screen` plus inspector; focused Web test `84 passed` and typecheck passed.
- [x] Add and verify the same-origin BFF route; focused route test `7 passed`, including raw-field fail-closed behavior.
- [x] Update bilingual architecture/API/roadmap evidence without changing completion-criteria verified claims.
- [x] Run `cargo fmt --all -- --check`, the Rust workspace check, focused API/local-SDK/Web checks, `pnpm check:web`, and public-surface searches; then select the next dependency-ready increment. Fresh format, Rust workspace, Web full check, and focused boundary receipts are recorded above.

The private read slice is locally verified and this plan is `validated` for its named boundary. The long-term goal remains active; the next increment requires a new bilingual Necessity Record and does not become a release, public-surface, or production-readiness receipt.

私有 read slice 已在本地验证，本计划对其命名边界标记为 `validated`。长期目标保持 active；下一增量必须有新的双语 Necessity Record，且本计划不是 release、public surface 或 production-readiness 回执。

## 2026-07-27 Fresh Revalidation / 2026-07-27 新鲜复核

The named local boundary was revalidated after the implementation settled. The protected API binding tests returned `4 passed`; storage binding tests returned `3 passed`; the local SDK full suite returned `70 passed`; the focused Web BFF route returned `9 passed`; the focused Web binding/presenter/inspector suite returned `8 passed`; `cargo fmt --all -- --check` passed; and `pnpm check:web` returned public SDK `14`, local SDK `70`, Web `160`, with a successful production Web build. A Luna review worker independently confirmed the necessity record and evidence scope. The worker also identified and the integration lead corrected stale user-flow/API wording. Git metadata was unavailable, PostgreSQL-backed authenticated runtime and authenticated browser/visual E2E were not observed, and remote CI, operator rehearsal, release, and production promotion remain deferred. The local Workflow read boundary is therefore validated, while the long-term goal remains active. The next admitted local increment is the protected benchmark execution adapter in `docs/superpowers/plans/2026-07-27-private-benchmark-execution-adapter.md`.

本次实现稳定后重新验证了命名的本地边界。protected API binding test 返回 `4 passed`；storage binding test 返回 `3 passed`；local SDK 全套返回 `70 passed`；聚焦 Web BFF route 返回 `9 passed`；聚焦 Web binding/presenter/inspector suite 返回 `8 passed`；`cargo fmt --all -- --check` 通过；`pnpm check:web` 返回 public SDK `14`、local SDK `70`、Web `160`，并成功完成 production Web build。一名 Luna 审查 worker 独立确认了必要性记录和证据范围；其指出的过时 user-flow/API 表述已由 Integration Lead 修正。Git metadata 不可用，PostgreSQL-backed authenticated runtime 与 authenticated browser/visual E2E 未观测，remote CI、operator rehearsal、release 与 production promotion 继续延期。因此，Workflow read 本地边界已验证，但长期目标保持 active。下一项准入的本地增量是 `docs/superpowers/plans/2026-07-27-private-benchmark-execution-adapter.md` 中的 protected benchmark execution adapter。
