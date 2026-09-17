# Local Contract Verification Plan / 本地契约验证计划

**Goal / 目标：** Admit one deterministic, evidence-only verifier for the local-only boundary. It checks an explicit supplied unified diff for public write additions, verifies the sole application-layer `GraphDiff::between` invocation, and rejects raw benchmark or knowledge payload fields from declared safe local DTO declarations.

## Necessity Record / 必要性记录

### Criterion and charter principle / 服务的完成条件与宪章原则

This increment advances Completion Criterion 8, reliable local quality gates, and the architecture rule that `GraphDiff` remains the sole graph-diff calculator. It also preserves the local-only admission decision: the default public API and SDK surface remain read-only while safe benchmark and knowledge projections must not expose private payloads.

本增量推进完成条件 8（可靠的本地质量门禁）以及架构中 `GraphDiff` 必须保持唯一图差异计算器的规则。它也维护本地准入决定：默认 public API 与 SDK surface 保持只读，安全的 benchmark 与 knowledge projection 不得暴露私有 payload。

### Unmet gap and risk / 未满足缺口与风险

Existing wave verifiers establish related repository and diff checks, but there is no one focused local receipt that binds three current drift risks: a newly added public write, a second application-layer graph calculation, or a raw field added to an explicitly safe local DTO. Without a deterministic check, later local composition work can make a boundary regression look like a harmless projection change.

现有 wave verifier 已覆盖相关的仓库与 diff 检查，但还没有一份聚焦的本地回执把三类当前漂移风险绑定起来：新增 public write、第二个 application-layer graph calculation，或在明确标为安全的 local DTO 中加入 raw field。缺少确定性检查时，后续本地 composition work 可能把边界回归伪装成无害的 projection change。

### Why this is next / 为什么现在实施

The relevant local source contracts already exist and have stable, narrow declaration locations: the application-layer graph comparison, benchmark workspace projection, local citation projection, and knowledge-to-memory replay projection. A static verifier and fixture test can therefore add regression evidence without extending product behavior, changing public contracts, or requiring unavailable external conditions.

相关本地 source contract 已存在，并有稳定且狭窄的 declaration location：application-layer graph comparison、benchmark workspace projection、local citation projection 与 knowledge-to-memory replay projection。因此，静态 verifier 与 fixture test 可以增加回归证据，而无需扩展产品行为、改变 public contract 或依赖不可用的外部条件。

### Explicit non-goals / 明确非目标

No application source change, Cargo or package manifest change, roadmap change, application-test change, Git inspection, browser automation, Docker, database, network request, secret access, `.env` read, public-write promotion, release claim, or production claim. The verifier does not prove runtime behavior or repository history; public-write evidence is only `passed` when a caller supplies a non-secret unified diff.

不修改 application source、Cargo 或 package manifest、roadmap 或 application test；不进行 Git inspection、browser automation、Docker、database、network request、secret access、`.env` read、public-write promotion、release claim 或 production claim。该 verifier 不证明 runtime behavior 或 repository history；只有调用方提供非 secret 的 unified diff 时，public-write evidence 才能标为 `passed`。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

Only these new files may be added: `scripts/verify-local-contracts.ps1`, `tests/contract/verify-local-contracts.test.ps1`, and this plan. The verifier reads only declared source files plus an optional caller-provided diff. It uses PowerShell text scanning, not runtime execution, repository metadata, environment files, or external tools.

只允许新增以下文件：`scripts/verify-local-contracts.ps1`、`tests/contract/verify-local-contracts.test.ps1` 与本计划。verifier 只读取已声明的 source file 与可选的调用方提供 diff。它使用 PowerShell text scanning，不执行 runtime、不读取 repository metadata、environment file，也不调用外部工具。

### Fresh verification required / 所需新鲜验证

Strict TDD requires the focused fixture test to fail before the verifier exists, then pass after the minimal script is added. Before handoff, run the focused test, run the verifier against the actual local source with no diff and record the public-write state as unobserved, and parse both PowerShell scripts with the PowerShell parser. Record only observed local static output.

严格 TDD 要求：在 verifier 尚不存在时，聚焦 fixture test 必须先失败；加入最小 script 后再通过。交付前运行聚焦 test、在实际本地 source 上不带 diff 运行 verifier 并将 public-write state 记录为 unobserved，以及使用 PowerShell parser 解析两个脚本。只记录已观察到的本地静态输出。

## TDD Steps / TDD 步骤

- [x] Add the fixture-first focused test for accepted and prohibited static inputs.
- [x] Observe the expected RED because the verifier script is absent.
- [x] Add the smallest deterministic PowerShell verifier.
- [x] Observe GREEN for the fixture test and actual local no-diff verification.
- [x] Parse both PowerShell scripts and record the exact local output below.

## Evidence Record / 证据记录

RED observed locally on 2026-07-23: `powershell -NoProfile -File tests/contract/verify-local-contracts.test.ps1` exited `1` because `scripts/verify-local-contracts.ps1` did not exist. The failure was the intended missing-verifier failure, before implementation.

This section will distinguish observed static evidence from intentionally unobserved Git, browser, Docker, network, CI, release, and production evidence.

- GREEN: the same focused command exited `0` and printed `verify-local-contracts fixture tests passed`. Its isolated fixtures prove acceptance of a safe diff and rejection of a public API write addition, a raw safe-DTO payload field, and a missing application-layer `GraphDiff::between` invocation.
- Actual local source: `powershell -NoProfile -File scripts/verify-local-contracts.ps1 -Root .` exited `0` and printed `local_contract_source=passed`, `graph_diff_application=passed count=1`, and `safe_local_dto_fields=passed`. Because no unified diff was supplied, it printed `public_write_additions=unobserved reason=no-unified-diff-input` and `overall=unobserved`.
- Syntax: the PowerShell parser accepted both new scripts and printed `powershell_syntax=passed files=2`.
- Git history/change evidence, browser, Docker, network, remote CI, release, and production evidence remain unobserved. No secret or `.env` input was read. This static receipt does not promote a public write or prove runtime behavior.

2026-07-23 本地已观察到 RED：`powershell -NoProfile -File tests/contract/verify-local-contracts.test.ps1` 以 `1` 退出，因为 `scripts/verify-local-contracts.ps1` 尚不存在。该失败是实现前预期的“缺少 verifier”失败。

本节将区分已观察到的静态证据与刻意保持未观测的 Git、browser、Docker、network、CI、release 和 production 证据。

- GREEN：同一聚焦命令以 `0` 退出，并打印 `verify-local-contracts fixture tests passed`。隔离 fixture 证明：safe diff 会被接受；public API write addition、safe DTO raw payload field 与缺失 application-layer `GraphDiff::between` invocation 均会被拒绝。
- 实际本地 source：`powershell -NoProfile -File scripts/verify-local-contracts.ps1 -Root .` 以 `0` 退出，并打印 `local_contract_source=passed`、`graph_diff_application=passed count=1` 与 `safe_local_dto_fields=passed`。由于未提供 unified diff，它打印 `public_write_additions=unobserved reason=no-unified-diff-input` 与 `overall=unobserved`。
- 语法：PowerShell parser 接受两个新脚本，并打印 `powershell_syntax=passed files=2`。
- Git history/change evidence、browser、Docker、network、remote CI、release 与 production evidence 保持未观测。未读取 secret 或 `.env` input。本静态回执不推广 public write，也不证明 runtime behavior。
