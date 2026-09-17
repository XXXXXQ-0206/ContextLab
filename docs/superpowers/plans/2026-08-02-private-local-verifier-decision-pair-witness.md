# Private Local Verifier Decision-Pair Witness Alignment / 私有 Local Verifier Decision-Pair Witness 对齐

## Necessity Record / 必要性记录

### Named criterion and charter principle / 对应条件与章程原则

This increment directly serves Criterion 8 (reproducible local contract and quality evidence) and
the charter's stable, fail-closed contract principle. The static local verifier must understand the
already-approved optional redacted `decision_pair_witness` without weakening its unknown-field or
raw-payload checks.

本增量直接服务条件 8（可复现本地 contract 与质量证据）以及章程的稳定、fail-closed contract 原则。静态 local verifier 必须理解
已经批准的可选脱敏 `decision_pair_witness`，同时不能放宽 unknown-field 或 raw-payload 检查。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The product response, API tests, local SDK, and Web tests already enforce an optional identity-only
decision-pair witness. The verifier and its fixture still allow only the former six-field response,
so the full verifier reports `overall=blocked` and cannot serve as a trustworthy regression gate.
The evidence gap is exact nested witness allowlisting plus fixture coverage for present and absent
witness states.

产品 response、API tests、local SDK 与 Web tests 已约束可选且仅含 identity 的 decision-pair witness；verifier 与 fixture 仍只允许旧的
六字段 response，导致完整 verifier 报告 `overall=blocked`，不能作为可靠回归门禁。证据缺口是 nested witness 的精确 allowlist，
以及 witness present/absent 两种 fixture 覆盖。

### Why now / 为什么现在优先

Independent review confirmed this is the direct cause of the only current local verifier block,
and it is smaller than adding another product surface. Aligning the evidence tool now lets future
Context/version/Diff changes be checked by a valid local gate. It does not depend on Docker,
PostgreSQL, browser, Git, remote CI, operator, release, or production conditions.

独立审查确认这是当前 local verifier 唯一阻塞的直接原因，且范围小于新增产品 surface。现在对齐 evidence tool，才能让后续
Context/version/Diff 变更由有效的本地 gate 检查；它不依赖 Docker、PostgreSQL、browser、Git、remote CI、operator、release 或 production 条件。

### Explicit non-goals / 明确非目标

- No Rust domain, API route, OpenAPI, SDK, Web, benchmark behavior, storage, migration, or response-shape change.
- No relaxation of unknown-field, raw-case, raw-output, or public-boundary checks; no secrets or external services.
- No Docker/PostgreSQL runtime, browser, Git, remote CI, operator rehearsal, release, production, or public-write claim.
- No second GraphDiff calculator; this tool only verifies source-shape contracts.

- 不改变 Rust domain、API route、OpenAPI、SDK、Web、benchmark behavior、storage、migration 或 response shape。
- 不放宽 unknown-field、raw-case、raw-output 或 public-boundary 检查；不读取 secrets、不访问外部服务。
- 不宣称 Docker/PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release、production 或 public-write 证据。
- 不新增第二个 GraphDiff calculator；本工具只验证 source-shape contract。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

Only `scripts/verify-local-contracts.ps1`, its fixture test, and this bilingual plan/roadmap receipt
are in scope. The verifier must accept exactly the optional witness envelope and reject unknown or
raw nested fields. Existing product files are read-only inputs to the static check.

范围仅限 `scripts/verify-local-contracts.ps1`、其 fixture test 与本双语 plan/roadmap 回执。verifier 必须精确接受可选 witness envelope，
并拒绝 unknown 或 raw nested field。既有产品文件只是 static check 的只读输入。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

Observe the current full verifier `overall=blocked`, then make its direct run report
`safe_local_dto_fields=passed` and `overall=unobserved` without a unified diff. Run the fixture
test plus focused API decision-pair tests, full Web checks, Rust format/tests/Clippy/MSRV as needed,
and confirm `GRAPH_DIFF_IMPL_COUNT=1`. The unobserved/deferred external boundaries remain separate.

先观测当前完整 verifier 的 `overall=blocked`，再使直接运行在无 unified diff 时报告 `safe_local_dto_fields=passed` 与
`overall=unobserved`。运行 fixture test、API decision-pair focused tests、完整 Web checks、必要的 Rust format/tests/Clippy/MSRV，并确认
`GRAPH_DIFF_IMPL_COUNT=1`。unobserved/deferred 外部边界继续独立记录。

## Implementation Checklist / 实施清单

- [x] Align the exact nested witness allowlist and fixture shape.
- [x] Add present/absent/unknown/raw witness regression coverage.
- [x] Run the verifier and record fresh bilingual evidence.

- [x] 对齐 nested witness exact allowlist 与 fixture shape。
- [x] 增加 present/absent/unknown/raw witness regression coverage。
- [x] 运行 verifier 并记录双语新鲜证据。

## Status / 状态

`completed / verified locally`; the external release evidence remains deferred and does not block
this local evidence-tool repair. / `completed / verified locally`；外部发布证据继续延期，不阻断本地 evidence-tool 修复。

## Implementation Receipt / 实施回执

The verifier now accepts exactly the optional `decision_pair_witness` field on
`LocalBenchmarkWorkspaceResponse`, requires the nested witness fields
`schema_version`, `project_id`, `context_id`, `baseline`, and `revised`, and inspects private
nested Rust structs as well as public DTOs. Fixture regressions cover a safe present/absent-capable
shape, unknown nested fields, and raw `payload` fields; all expected blocked cases remain blocked.

verifier 现在精确接受 `LocalBenchmarkWorkspaceResponse` 上可选的 `decision_pair_witness` 字段，要求 nested witness 精确包含
`schema_version`、`project_id`、`context_id`、`baseline` 与 `revised`，并同时检查 private nested Rust struct 与 public DTO。fixture
regression 覆盖 safe present/absent-capable shape、unknown nested field 与 raw `payload` field；预期的 blocked case 继续保持 blocked。

Fresh evidence: before the repair, the direct verifier reported `safe_local_dto_fields=blocked`
and `overall=blocked` for the stale allowlist. After the repair,
`tests/contract/verify-local-contracts.test.ps1` passed and the direct verifier reported
`safe_local_dto_fields=passed`, all local source/route/SDK/public-boundary checks passed,
`public_write_additions=unobserved`, and `overall=unobserved` without unified diff input. / 新鲜证据：修复前 direct verifier 因
stale allowlist 报告 `safe_local_dto_fields=blocked` 与 `overall=blocked`。修复后
`tests/contract/verify-local-contracts.test.ps1` 通过，direct verifier 报告 `safe_local_dto_fields=passed`，local source/route/SDK/public-boundary
checks 均通过，`public_write_additions=unobserved`，且无 unified diff input 时 `overall=unobserved`。

No product route, API/SDK/Web behavior, migration, provider, secret, or GraphDiff calculation
changed. PostgreSQL/Docker, authenticated browser/visual smoke, Git, remote CI, operator
rehearsal, release, production, and public-write readiness remain `ignored`, `unobserved`, or
`deferred`; the long-term goal remains active. / 未改变产品 route、API/SDK/Web behavior、migration、provider、secret 或 GraphDiff
calculation。PostgreSQL/Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与
public-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`；长期目标保持 active。
