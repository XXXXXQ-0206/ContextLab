# Plan Closure Ledger / 计划结案台账

**Status / 状态:** closed for the local plan set on 2026-09-11. This ledger reconciles every file in
`docs/superpowers/plans/` against the code, migrations, tests, and roadmap receipts that actually
exist in the worktree.

**状态：** 本地计划集已于 2026-09-11 结案。本台账把 `docs/superpowers/plans/` 中的每一份计划，与工作树中实际存在的代码、迁移、测试和路线图回执逐一对账。

## Why this ledger exists / 为什么需要本台账

Every plan in this repository was written **before** implementation, as a test-first instruction
list. Implementing workers recorded their results in `docs/roadmap/active-long-term-goal.md`,
`docs/roadmap/completion-criteria.md`, and `docs/roadmap/parallel-development-plan.md`, but most
never went back to tick the original step boxes. The result was a misleading picture: 228 unticked
boxes across 26 plans, almost all of them describing work that is demonstrably shipped.

本仓库的每份计划都是在实现**之前**写下的 test-first 指令清单。实现者把结果记在
`docs/roadmap/active-long-term-goal.md`、`docs/roadmap/completion-criteria.md` 与
`docs/roadmap/parallel-development-plan.md`，但绝大多数没有回头勾选最初的步骤框。结果是画面失真：
26 份计划里留下 228 个未勾选步骤，而其中几乎全部描述的是已经交付的工作。

An unticked box is **not** evidence of missing work, and a ticked box is **not** evidence of shipped
work. Only the linked artifact is. This ledger therefore closes plans by artifact, not by checkbox.

未勾选的框**不是**缺失工作的证据，勾选过的框**也不是**已交付的证据，只有链接到的产物才是。因此本台账按产物结案，而不是按复选框结案。

## Closure rule / 结案规则

A plan is `closed / 已结案` when every capability it names is present in the worktree and is covered
by the roadmap receipts, or when its remaining lines are explicit deferrals with a named external
prerequisite. A plan is `open / 未结案` only if it names a capability that no artifact provides.

当一份计划点到的每项能力都已在工作树中存在且有路线图回执覆盖，或其剩余条目是带有明确外部前置的显式延期项时，该计划记为 `closed / 已结案`。
只有当计划点到的某项能力没有任何产物提供时，才记为 `open / 未结案`。

## A. Closed — shipped, step boxes are an authoring artifact / 已结案：已交付，步骤框属于写作残留

| Plan / 计划 | Unticked | Artifact that closes it / 结案产物 |
| --- | --- | --- |
| `2026-07-08-contextlab-foundation.md` | 3 | `Cargo.toml`, `crates/`, `server/api`, `apps/web`, `packages/` |
| `2026-07-09-context-graph-foundation.md` | 1 | `crates/graph/src/lib.rs`, `GET /api/v1/context-graph/preview` |
| `2026-07-09-context-persistence-graph-projection.md` | 2 | `crates/storage/src/projection.rs`, `crates/storage/migrations/0001_context_platform.sql` |
| `2026-07-10-context-evaluation-scorecard-api.md` | 15 | `GET /api/v1/contexts/{id}/evaluation-scorecard`, `docs/api/openapi.json` |
| `2026-07-11-atomic-commit-snapshot-capture.md` | 14 | `crates/storage/src/commit_snapshot_writer.rs`, `0003`/`0004` migrations |
| `2026-07-11-commit-graph-snapshot-contract.md` | 1 | `crates/storage/src/commit_graph_snapshot.rs` |
| `2026-07-11-context-graph-inspector.md` | 13 | `apps/web/src/app/context-graph-inspector.tsx` plus its presenter/screen tests |
| `2026-07-11-oidc-jwks-authentication.md` | 20 | `crates/auth/src/oidc.rs`, `.env.example` OIDC block |
| `2026-07-11-postgres-authorization-audit-storage.md` | 19 | `crates/storage/src/authorization_audit_review.rs`, `0005`/`0008` migrations |
| `2026-07-11-postgres-commit-graph-snapshots.md` | 17 | `0002_context_commit_graph_snapshots.sql`, `crates/storage/src/postgres.rs` |
| `2026-07-11-principal-identity-namespace.md` | 1 | `0006_principal_identity_namespace.sql` |
| `2026-07-11-protected-runtime-auth.md` | 10 | `AppState::try_from_env`, `.env.example` protected block, protected-router tests |
| `2026-07-11-version-backed-graph-diff-api.md` | 14 | protected `GET /api/v1/local/contexts/{id}/graph-diff` |
| `2026-07-13-production-migration-rehearsal.md` | 9 | `scripts/verify-production-migration-rehearsal.sh`, `0009_contextlab_migration_ledger.sql` |
| `2026-07-13-restricted-audit-purge-executor.md` | 21 | `crates/storage/src/authorization_audit_purge.rs`, privileged SQL assets |
| `2026-07-13-shared-rate-limit.md` | 16 | `crates/storage/src/protected_route_rate_limit.rs`, `0010`/`0011` migrations |
| `2026-07-14-private-component-content-revisions.md` | 23 | `crates/storage/src/component_content_revision.rs`, `0012`–`0014` migrations |
| `2026-07-18-private-benchmark-execution-orchestration.md` | 1 | `crates/evaluation/src/benchmark.rs`, `0021_benchmark_execution_idempotency.sql` |
| `2026-07-18-private-component-uses-relationship-lifecycle.md` | 7 | typed `Uses` add/remove in `crates/versioning/src/change.rs` and the lifecycle writer |
| `2026-07-22-private-benchmark-decision-discovery.md` | 1 | `apps/web/src/app/context-benchmark-decision-discovery-*` |
| `2026-07-22-private-benchmark-diff-selection.md` | 2 | `apps/web/src/app/context-benchmark-decision-diff-*` |
| `2026-08-01-private-branch-head-error-redaction.md` | 3 | `apps/web/src/app/local-branch-heads.test.tsx` redaction regressions |

## B. Closed as deliberate deferral / 结案为显式延期

These plans keep their unticked lines on purpose. Each line is a **guard** that must stay unmet
until a named external prerequisite exists, so ticking it would be a false claim. They are closed
as decisions, not as work.

这些计划有意保留未勾选条目。每一条都是**守卫条件**，在具名的外部前置具备之前必须保持未满足；勾选它等于作出虚假声明。它们作为决策结案，而不是作为工作结案。

| Plan / 计划 | Unticked | Why it stays deferred / 延期原因 |
| --- | --- | --- |
| `2026-07-11-guarded-context-commit-write.md` | 6 | Public promotion of the protected commit route, its OpenAPI operation, and its TypeScript SDK method require production authorization configuration plus disposable-PostgreSQL concurrency evidence that does not exist yet. The private route is shipped; only public promotion is deferred. / protected commit route 的 public promotion、其 OpenAPI operation 与 TypeScript SDK method，需要尚不存在的 production authorization 配置与 disposable PostgreSQL 并发证据。私有 route 已交付，延期的只是 public promotion。 |
| `2026-07-12-trusted-identity-group-rbac.md` | 2 | Remote CI upgrade/revocation evidence and public-protected-write evaluation. Both are now reachable through `.github/workflows/verify.yml` once the repository is published, and neither is a local code gap. / 远端 CI 的 upgrade/revocation 证据与 public protected-write 评估。仓库发布后两者都可通过 `.github/workflows/verify.yml` 获得，且都不是本地代码缺口。 |
| `2026-07-13-audit-retention-access-governance.md` | 4 | The restricted purge executor was deferred by this plan and then delivered by the later `2026-07-13-restricted-audit-purge-executor.md`. The remaining lines are the public-surface guards that keep audit review private. / 受限 purge executor 由本计划延期，随后由 `2026-07-13-restricted-audit-purge-executor.md` 交付。剩余条目是保持 audit review 私有的 public-surface 守卫。 |
| `2026-07-15-local-context-lifecycle-vertical-slice.md` | 3 | Lifecycle-specific PostgreSQL E2E and authenticated browser-to-BFF-to-protected-Axum smoke. The PostgreSQL half is superseded by the 2026-08-02 lifecycle runtime receipt; the authenticated browser half needs a disposable loopback PostgreSQL service, which this machine does not have. / lifecycle 专用 PostgreSQL E2E 与 authenticated browser-to-BFF-to-protected-Axum smoke。PostgreSQL 部分已被 2026-08-02 的 lifecycle runtime 回执取代；authenticated browser 部分需要 disposable loopback PostgreSQL service，本机没有。 |

## C. Open / 未结案

None. No plan names a capability that lacks an artifact in this worktree.

无。没有任何计划点到工作树中缺少产物的能力。

## D. What this ledger does not claim / 本台账不作声明

Closing the plan set is **not** project completion. It says only that the plan backlog is
reconciled. The completion criteria in `docs/roadmap/completion-criteria.md` remain the governing
document, and the boundaries below are still exactly as recorded elsewhere:

计划集结案**不等于**项目完成。它只说明计划积压已完成对账。治理文件仍是 `docs/roadmap/completion-criteria.md` 中的收束条件，下列边界与其它文档记录完全一致：

- PostgreSQL live runtime on this machine remains `unobserved`; the named ignored cases are
  available through `scripts/verify-disposable-postgres-storage.sh` and run in CI.
- Authenticated browser-to-BFF-to-protected-Axum runtime remains `unobserved` on this machine.
- Remote CI, operator-approved change rehearsal, release, production rollout, and public
  protected-write promotion remain `deferred`.
- Local test suites, preview data, fixtures, and screenshots never substitute for any of the above.

- 本机的 PostgreSQL live runtime 仍为 `unobserved`；具名 ignored case 可通过 `scripts/verify-disposable-postgres-storage.sh` 运行，并在 CI 中执行。
- 本机的 authenticated browser-to-BFF-to-protected-Axum runtime 仍为 `unobserved`。
- 远端 CI、operator 批准的变更演练、release、production rollout 与 public protected-write promotion 仍为 `deferred`。
- 本地测试套件、preview data、fixture 与截图都不能替代上述任何一项。
