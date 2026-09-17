# External Release Evidence Protocol / 外部发布证据协议

> **Status / 状态：** Deferred future-deployment reference. Remote disposable CI, operator-approved rehearsal, and public release or production promotion are outside the current local open-source delivery scope; do not collect, audit, request, or wait for receipts until external operating conditions exist.
>
> **状态：** 延期的未来部署参考。远端 disposable CI、operator 批准的演练以及公开发布或生产推广均不属于当前本地开源交付范围；在外部运行条件具备前，不得收集、审计、索取或等待回执。

## Purpose and Scope / 目的与范围

This protocol defines the only admissible evidence receipts for the two external gates that remain before ContextLab may decide whether protected Context commit writes are ready for a public REST/SDK contract: a remote disposable-CI result and an operator-approved production-change rehearsal. It is an evidence contract, not a deployment runbook, an operator transport, or a product feature.

本协议定义 ContextLab 在决定 protected Context commit write 是否可进入 public REST/SDK contract 前，两个外部门禁唯一可准入的证据回执：远端 disposable CI 结果与 operator 批准的 production-change rehearsal。它是一份证据契约，不是部署 runbook、operator transport 或产品功能。

This gate applies only to public protected-write promotion, release, and production rollout. An `unobserved` receipt is deferred release evidence; it does not block dependency-ready private core development, and local tests must never be presented as a substitute.

本门禁只适用于 public protected-write promotion、release 与生产推广。`unobserved` 回执属于延期发布证据，不阻断依赖就绪的私有核心开发；任何本地测试都不得被描述成其替代品。

The authoritative status ledger is `docs/roadmap/goal-governance.md`. This protocol defines what a receipt must contain and how it is reviewed. It never reads or records `.env`, DSNs, passwords, access tokens, JWTs, OIDC claims, production hostnames, raw audit data, or unredacted logs.

权威状态账本位于 `docs/roadmap/goal-governance.md`。本协议定义回执必须包含的字段以及审阅方法；它绝不读取或记录 `.env`、DSN、密码、access token、JWT、OIDC claim、生产主机名、原始审计数据或未脱敏日志。

## Necessity and Boundaries / 必要性与边界

This protocol serves the `Reliable release gates` and `Production security and collaboration` completion conditions. Fresh local PostgreSQL evidence, the private isolated purge executor, and a disposable production-like forward rehearsal are necessary local prerequisites, but cannot prove an external CI execution or an operator-approved change rehearsal. A receipt may provide evidence for one specific gate only; it never closes the long-term goal.

本协议服务于 `Reliable release gates` 与 `Production security and collaboration` 完成条件。新鲜的本地 PostgreSQL 证据、私有隔离 purge executor 与 disposable 的 production-like forward rehearsal 是必要的本地前提，但无法证明一次外部 CI 执行或 operator 批准的变更演练。每份回执只能为一个特定门禁提供证据，永远不能关闭长期目标。

**Non-goals / 非目标:** This protocol does not authorize a production deployment, production or replica connection, SSH tunnel, port-forwarded target, shared cluster reset, migration rewrite, rollback claim, public write promotion, REST/OpenAPI/SDK/Web mutation, operator endpoint, scheduler, or a second graph-diff calculator. `GraphDiff` remains the sole graph-diff calculator.

**非目标:** 本协议不授权生产部署、生产或 replica 连接、SSH tunnel、端口转发目标、共享集群 reset、migration 改写、rollback 声明、public write promotion、REST/OpenAPI/SDK/Web mutation、operator endpoint、scheduler 或第二套 graph-diff calculator。`GraphDiff` 仍是唯一的 graph-diff calculator。

## Evidence States and Binding / 证据状态与归属

Every receipt uses exactly one evidence state:

- `unobserved`: no reviewable receipt is available.
- `observed_pass`: all required fields are present, the receipt is independently reviewable, and its stated gate passes.
- `observed_fail`: the receipt is reviewable and proves the stated gate failed.
- `inconclusive`: a receipt exists but is incomplete, unreviewable, contradictory, expired, or cannot be bound to the claimed source.

每份回执都只能使用一种证据状态：

- `unobserved`：没有可审阅回执。
- `observed_pass`：必填字段齐全、回执可被独立审阅，且其声明的门禁通过。
- `observed_fail`：回执可审阅，且证明其声明的门禁失败。
- `inconclusive`：回执存在但不完整、不可审阅、相互矛盾、已失效，或无法归属到其声称的来源。

Each receipt includes an opaque `evidence_id`, collection time in UTC, custodian/reviewer, retention location, redaction statement, and immutable reference or digest. A status may not be inferred from a narrative, a screenshot without a stable source, a local test result, or a mutable link alone.

每份回执都包含不透明 `evidence_id`、UTC 采集时间、保管人/审阅人、保留位置、脱敏声明以及不可变引用或 digest。不得从叙述、没有稳定来源的截图、本地测试结果或单独的可变链接推断证据状态。

`remote_snapshot_binding` is one of `exact_commit`, `provider_only`, or `unavailable`. `exact_commit` requires an independently reproducible match between the provider-reported `head_sha`, reviewed workflow revision, and the reviewed repository snapshot. `provider_only` means the provider result is authentic but cannot be matched to the local review snapshot. `unavailable` means no trustworthy binding can be established. When local Git metadata is unavailable, the local-worktree binding is `unavailable`; a reviewer may say only that a provider result was observed, never that the current worktree passed the remote gate.

`remote_snapshot_binding` 只能是 `exact_commit`、`provider_only` 或 `unavailable`。`exact_commit` 要求 provider 报告的 `head_sha`、已审阅的 workflow revision 与已审阅的仓库 snapshot 可被独立复现地匹配。`provider_only` 表示 provider 结果可信，但不能与本地审阅 snapshot 匹配。`unavailable` 表示无法建立可信归属。本地 Git 元数据不可用时，local-worktree binding 必须为 `unavailable`；审阅者只能说观察到了 provider 结果，绝不能说当前工作树通过了远端门禁。

## Remote Disposable-CI Receipt / 远端 Disposable-CI 回执

A remote CI receipt records all of the following, redacted where necessary:

一份远端 CI 回执必须记录以下全部字段，并在必要时脱敏：

1. `evidence_id`, provider, repository identity, workflow path, reviewed workflow blob digest, run ID, immutable run URL, run attempt, event, ref, and pull-request identifier when applicable.
2. Provider-reported `head_sha`, a retained checked-out revision proof from `git rev-parse --verify HEAD^{commit}` (or an equivalent immutable provider attestation), `remote_snapshot_binding`, local-worktree binding, start and end UTC timestamps, runner image identity, and resolved action/container references or digests. `exact_commit` additionally requires the checked-out revision to equal the provider-reported `head_sha`.
3. Each required job and expected step, its conclusion, command summary, and exit code. A required missing, skipped, neutral, cancelled, or timed-out step makes the receipt `inconclusive` unless a separately reviewed workflow policy marks it non-required.
4. Immutable or retained redacted log/artifact references with digests and retention period, plus the collection time, collector, independent reviewer, and redaction statement.
5. A precise conclusion limited to the bound snapshot, such as “the disposable CI gate passed for this exact commit.” It must not claim production deployment, production migration completion, rollback/DR proof, zero downtime, or public-write readiness.

1. `evidence_id`、provider、仓库身份、workflow path、已审阅的 workflow blob digest、run ID、不可变 run URL、run attempt、event、ref，以及适用时的 pull-request identifier。
2. Provider 报告的 `head_sha`、来自 `git rev-parse --verify HEAD^{commit}` 的已保留 checked-out revision proof（或等价的不可变 provider attestation）、`remote_snapshot_binding`、local-worktree binding、开始和结束 UTC 时间、runner image identity，以及解析后的 action/container 引用或 digest。`exact_commit` 还要求 checked-out revision 与 provider 报告的 `head_sha` 相等。
3. 每个必需 job 与预期 step、其结论、命令摘要与退出码。除非经单独审阅的 workflow policy 将其标记为非必需，否则缺失、skipped、neutral、cancelled 或 timed-out 的必需 step 会使回执成为 `inconclusive`。
4. 带 digest 和保留期的不可变或已保留脱敏 log/artifact 引用，以及采集时间、采集者、独立审阅者与脱敏声明。
5. 只针对已归属 snapshot 的精确结论，例如“该 disposable CI gate 已为这个 exact commit 通过”。不得声称生产部署、生产 migration 完成、rollback/DR 证明、零停机或 public-write readiness。

A successful `verify` job captures `.ci-evidence/verification-evidence.txt` and its SHA-256 in the retained `contextlab-ci-evidence` artifact. The capture proves only that the runner's checked-out commit equals `GITHUB_SHA` and fingerprints the checked-out workflow. It deliberately contains no environment values, connection URLs, credentials, tokens, claims, raw logs, production facts, job approval, or release conclusion. It is input to a later redacted, independently reviewed receipt; the artifact alone is neither `observed_pass` nor public-write approval.

成功的 `verify` job 会在保留的 `contextlab-ci-evidence` artifact 中捕获 `.ci-evidence/verification-evidence.txt` 及其 SHA-256。该捕获只证明 runner 的 checked-out commit 等于 `GITHUB_SHA`，并 fingerprint 当前 checked-out workflow。它刻意不包含 environment value、connection URL、credential、token、claim、raw log、production fact、job approval 或 release conclusion。它是后续脱敏、独立审阅回执的输入；artifact 本身既不是 `observed_pass`，也不是 public-write approval。

## Operator-Approved Production-Change Rehearsal Receipt / Operator 批准的生产变更演练回执

The rehearsal is always executed in an isolated environment explicitly classified as non-production. It must not target production, a production replica, an SSH tunnel, a port-forwarded endpoint, or a shared cluster. A loopback URL alone is insufficient proof of isolation; it cannot qualify the existing disposable reset scripts as operator-rehearsal evidence.

演练始终在被明确分类为 non-production 的隔离环境执行。不得以生产环境、生产 replica、SSH tunnel、端口转发 endpoint 或共享集群为目标。单独的 loopback URL 不能证明隔离性；它不能使现有 disposable reset script 成为 operator-rehearsal evidence。

The receipt records all of the following, without secrets or raw target identity:

回执必须记录以下全部字段，且不包含 secret 或原始目标身份：

1. `evidence_id`, change/ticket ID, requester, independently approving operator and approval role, approval timestamp, change window, abort owner, and runbook version/digest.
2. Redacted environment classification and identity fingerprint, account/instance isolation proof, explicit non-production assertion, and a declaration that historical baseline data is synthetic or de-identified.
3. Starting schema/migration boundary, intended target boundary, and a migration-asset manifest with the ID and SHA-256 of every regular migration and each separately ordered privileged SQL artifact. The receipt distinguishes this external manifest from the current ledger test, which proves append-only behavior but does not itself establish real migration-asset integrity.
4. Executor identity and least-privilege role assertions, exact command handles or retained command digests, start/end UTC times, exit outcomes, and any aborted or unexecuted steps.
5. Pre- and post-run acceptance assertions: legacy authorization-audit preservation, forward schema boundary, immutable-ledger behavior, duplicate-ID rejection, and a runtime purge role with `EXECUTE` only and no direct audit-table `SELECT` or `DELETE` privilege.
6. Cleanup proof for the rehearsal database and temporary roles, observed exceptions, independent review, and a final evidence state. `rollback_status` may be `not_claimed`; this protocol never treats a forward rehearsal as rollback, disaster-recovery, or zero-downtime proof.

1. `evidence_id`、change/ticket ID、申请人、独立审批 operator 与审批角色、审批时间、变更窗口、中止负责人以及 runbook version/digest。
2. 脱敏后的环境分类和 identity fingerprint、账户/实例隔离证明、明确的 non-production 声明，以及历史 baseline data 为 synthetic 或去标识化的声明。
3. 起始 schema/migration boundary、目标 boundary，以及 migration-asset manifest：每个 regular migration 与每个单独有序的 privileged SQL artifact 的 ID 和 SHA-256。该回执必须将外部 manifest 与现有 ledger test 区分开来：后者只证明 append-only 行为，不能自行证明真实 migration asset integrity。
4. Executor identity 与 least-privilege role assertion、精确 command handle 或已保留 command digest、开始/结束 UTC 时间、退出结果，以及所有中止或未执行 step。
5. 运行前后的验收 assertion：legacy authorization-audit 保留、forward schema boundary、immutable-ledger 行为、duplicate-ID 拒绝，以及 runtime purge role 只有 `EXECUTE` 且没有直接 audit-table `SELECT` 或 `DELETE` 权限。
6. rehearsal database 与 temporary role 的 cleanup proof、观察到的 exception、独立审阅与最终 evidence state。`rollback_status` 可以是 `not_claimed`；本协议绝不将 forward rehearsal 当作 rollback、disaster-recovery 或 zero-downtime proof。

A successful remote `verify` job also retains `contextlab-rehearsal-evidence`, containing a SHA-256 manifest of every checked-in regular migration, privileged SQL artifact, and the rehearsal runner, plus a SHA-256 of that manifest. This is technical asset-integrity input only. It does not prove a change/ticket, approval, non-production isolation, synthetic/de-identified data, executor identity, actual privileged execution, cleanup, an operator decision, production behavior, rollback/DR, or `observed_pass`.

成功的远端 `verify` job 还会保留 `contextlab-rehearsal-evidence`，其中包含每个已检入 regular migration、privileged SQL artifact 与 rehearsal runner 的 SHA-256 manifest，以及该 manifest 的 SHA-256。这只是一份技术 asset-integrity input。它不证明 change/ticket、审批、non-production isolation、synthetic/去标识化数据、executor identity、实际 privileged execution、cleanup、operator 决策、production behavior、rollback/DR 或 `observed_pass`。

## Review Procedure / 审阅流程

1. Verify the receipt is complete, redacted, immutable or retained, and independently reviewable. Mark missing or unverifiable evidence `inconclusive`.
2. For remote CI, match provider `head_sha` and workflow revision to the claimed snapshot, then require all configured required jobs and steps to succeed.
3. For an operator rehearsal, first prove non-production isolation and independent approval. Only then review the migration-asset manifest, forward-run assertions, role isolation, cleanup, and recorded exceptions.
4. Record the result in `docs/roadmap/goal-governance.md` with its `evidence_id`, state, binding, reviewer, and narrow conclusion. A failure remains recorded as `observed_fail`; it is never overwritten by a prose summary.

1. 验证回执完整、脱敏、不可变或已保留，并可独立审阅。缺失或不可验证的证据标记为 `inconclusive`。
2. 对远端 CI，将 provider `head_sha` 与 workflow revision 匹配到声称的 snapshot，然后要求所有已配置的 required job 和 step 成功。
3. 对 operator rehearsal，先证明 non-production isolation 与独立审批；之后才审阅 migration-asset manifest、forward-run assertion、角色隔离、cleanup 与记录的 exception。
4. 在 `docs/roadmap/goal-governance.md` 中记录结果，包括 `evidence_id`、state、binding、审阅者与狭义结论。失败必须以 `observed_fail` 保留，绝不能被叙述性总结覆盖。

## Current Status and Decision Rule / 当前状态与决策规则

As recorded in the governance ledger on 2026-07-14, the remote CI receipt is `unobserved`, the operator-approved rehearsal receipt is `unobserved`, and the local-worktree binding is `unavailable` because usable local Git metadata is absent. No remote CI success, operator approval, production deployment, production migration, rollback proof, or public-write readiness is claimed.

如 2026-07-14 治理账本所记，远端 CI 回执为 `unobserved`，operator 批准的 rehearsal 回执为 `unobserved`，local-worktree binding 为 `unavailable`，因为可用的本地 Git metadata 缺失。本文不声称远端 CI 成功、operator 批准、生产部署、生产 migration、rollback proof 或 public-write readiness。

Before the project may start a separate, evidence-based decision increment about public-write readiness, the remote CI receipt must be `observed_pass` with `remote_snapshot_binding` equal to `exact_commit`, and the operator rehearsal receipt must be independently reviewable `observed_pass` evidence. `provider_only` never satisfies the remote release gate. Any `unobserved`, `observed_fail`, or `inconclusive` result keeps the gate open. Even two passing receipts do not create a public write; a new Necessity Record must decide that question separately.

This open release gate has no authority to pause unrelated Context-first domain, storage, versioning, graph, evaluation, or design-system increments whose local dependencies and verification are complete.

只有 remote CI 回执为 `observed_pass` 且 `remote_snapshot_binding` 等于 `exact_commit`，并且 operator rehearsal 回执成为可独立审阅的 `observed_pass` 证据后，项目才能启动一个单独、基于证据的 public-write readiness 决策增量。`provider_only` 永远不能满足远端 release gate。任何 `unobserved`、`observed_fail` 或 `inconclusive` 结果都会使门禁保持打开。即使两份回执通过，也不会创建 public write；该问题仍必须由新的 Necessity Record 单独决策。

该开放的发布门禁无权暂停本地依赖与验证均已满足的 Context-first domain、storage、versioning、graph、evaluation 或 design-system 增量。
