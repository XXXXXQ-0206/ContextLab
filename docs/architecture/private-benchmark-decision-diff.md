# Private Benchmark Decision Pair Witness / 私有 Benchmark Decision Pair Witness

## Contract Status / 契约状态

This document records the private, read-only contract for a server-owned benchmark
decision pair witness. It is a documentation contract for the current wave; it does
not add a route, SDK surface, migration, or runtime evidence. / 本文记录为 server-owned
的 benchmark decision pair witness 私有只读契约。本文件是当前 wave 的文档契约；不新增
route、SDK surface、migration，也不新增 runtime evidence。

## Scope / 作用域

`decision_pair_witness` is optional at the response envelope level and is present only
for a decision-bound comparison. When present, it identifies the exact ordered pair
used by the server-side resolver:

`(project_id, context_id, baseline_commit_id, baseline_decision_id, revised_commit_id, revised_decision_id)`

`decision_pair_witness` 在 response envelope 层是可选的，且仅在 comparison 绑定具体
decision 时出现。当它出现时，标识 server-side resolver 使用的精确有序 pair：

`(project_id, context_id, baseline_commit_id, baseline_decision_id, revised_commit_id, revised_decision_id)`

The response shape is versioned and contains identity facts only:

```json
{
  "schema_version": 1,
  "project_id": "<uuid>",
  "context_id": "<uuid>",
  "baseline_commit_id": "<uuid>",
  "baseline_decision_id": "<uuid>",
  "revised_commit_id": "<uuid>",
  "revised_decision_id": "<uuid>"
}
```

该 response shape 有版本且只包含 identity fact：上述六个 scope identity 与数值型
`schema_version=1` 是冻结字段。

## Invariants / 不变量

1. The server is authoritative. The witness is produced from the resolved, exact
   decision-bound records and is not reconstructed from a cohort, dataset, UI state,
   or client-side selection. / server 保持权威。witness 必须来自已解析的精确
   decision-bound record，不得从 cohort、dataset、UI state 或 client-side selection 重建。
2. `project_id` and `context_id` match the request and the parent response scope.
   Both baseline and revised commit/decision identities match the requested pair. /
   `project_id` 与 `context_id` 必须匹配 request 及 parent response scope；baseline 与
   revised 的 commit/decision identity 必须匹配请求 pair。
3. `baseline` and `revised` are ordered roles. A commit or decision cannot be paired
   with itself. / `baseline` 与 `revised` 是有序角色；commit 或 decision 不得与自身配对。
4. The witness is immutable for a response and has no client-supplied authority.
   Client fields are selectors, never facts. / witness 在 response 内不可变，client 不拥有
   写入 authority；client field 只能是 selector，不能成为 fact。
5. `GraphDiff::between` remains the sole graph-diff calculator. The witness validates
   identity and scope; it does not calculate, normalize, or reinterpret a diff. /
   `GraphDiff::between` 仍是唯一 graph-diff calculator。witness 只校验 identity 与 scope，
   不计算、normalize 或重新解释 diff。

## Fail-Closed Rules / Fail-Closed 规则

The protected API and non-public local SDK reject the response or map it to a safe
unavailable/error state when any of the following is true: / 当出现以下任一情况时，
protected API 与非公开 local SDK 必须拒绝 response，或映射为安全的 unavailable/error
state：

| Invalid condition / 无效条件 | Required behavior / 必须行为 |
| --- | --- |
| Missing or null witness when comparison is decision-bound / decision-bound comparison 缺失或为 null | Fail closed; never infer decision identity. / fail closed，不得推断 decision identity。 |
| Unknown key, missing required key, wrong `schema_version`, malformed UUID, or invalid role / unknown key、缺字段、错误版本、UUID 格式错误或 role 无效 | Reject before presentation. / 在 presentation 前拒绝。 |
| Project, Context, commit, or decision scope mismatch / project、Context、commit 或 decision scope 不匹配 | Reject as scope drift; do not display a partial comparison. / 按 scope drift 拒绝，不展示 partial comparison。 |
| Baseline/revised self-pair or duplicate role / baseline/revised self-pair 或 role 重复 | Reject as invalid pair. / 按 invalid pair 拒绝。 |
| Resolver, repository, authorization, or audit unavailable / resolver、repository、authorization 或 audit 不可用 | Return only the safe mapped error; do not leak internals. / 仅返回安全映射错误，不泄露内部信息。 |

The client must not silently downgrade a missing or drifted witness to a cohort-only
identity. / client 不得将缺失或漂移的 witness 静默降级为仅 cohort identity。

## Redaction And Non-Public Boundary / 脱敏与非公开边界

The response may contain the witness and the already-admitted redacted decision-diff
summary. It must not expose dataset IDs, raw cases, inputs, expected outputs, measured
values, model outputs, provider payloads, credentials, tokens, secrets, storage errors,
or resolver internals. Safe error codes are allowed only when they are part of the
existing private transport contract. / response 可以包含 witness 与已准入的脱敏
decision-diff summary，但不得暴露 dataset ID、raw case、input、expected output、测量值、
model output、provider payload、credential、token、secret、storage error 或 resolver
internal。只有既有 private transport contract 已定义的安全 error code 才可以返回。

The allowed path is: protected private local API -> non-public local SDK -> same-origin
BFF -> Web `data -> presenter -> screen`. Bearer credentials are request-scoped,
cookies/credentials are omitted, and success/error responses are `private, no-store`.
The public REST catalog, OpenAPI, and public SDK remain unchanged. There is no public
write, Web mutation, operator transport, migration, provider, scheduler, or release
surface in this contract. / 允许路径为：protected private local API -> non-public local
SDK -> same-origin BFF -> Web `data -> presenter -> screen`。Bearer credential 仅限
request scope，省略 cookies/credentials，成功与错误 response 均为 `private, no-store`。
public REST catalog、OpenAPI 与 public SDK 保持不变。本契约不包含 public write、Web
mutation、operator transport、migration、provider、scheduler 或 release surface。

## Current Wave Ownership / 当前 Wave Ownership

This is the ownership for the documentation-only wave. It does not silently transfer
write ownership for implementation files. / 以下是本次仅文档 wave 的 ownership；不会默默
转移 implementation file 的写入 ownership。

| Owner / Owner | Current responsibility / 当前职责 | Exclusive write boundary / 独占写入边界 |
| --- | --- | --- |
| H Docs/Contribution/QA | Record and reconcile the bilingual private contract, public boundary, evidence vocabulary, and wave ledger. / 记录并对账双语私有契约、public boundary、证据词汇与 wave ledger。 | `docs/api/wave-1-integration-contracts.md`, `docs/architecture/private-benchmark-decision-diff.md`, `docs/roadmap/parallel-development-plan.md` only. |
| Benchmark/Evaluation domain owner | Remains the authority for resolved benchmark decision/evidence identity if a future implementation wave is separately admitted. / 若未来单独准入 implementation wave，继续负责 resolved benchmark decision/evidence identity 的 authority。 | No write admitted by this documentation update. / 本次文档更新不准入写入。 |
| Integration Lead | Owns any future protected transport composition and admission review after an explicit implementation decision. / 未来若明确准入 implementation，负责 protected transport composition 与 admission review。 | No API, SDK, Web, or code write admitted by this documentation update. / 本次文档更新不准入 API、SDK、Web 或代码写入。 |
| Web/Design System owner | Consumes an admitted server-owned DTO through existing presenters and screens; never calculates identity or Diff. / 通过既有 presenter/screen 消费已准入的 server-owned DTO；绝不计算 identity 或 Diff。 | No Web write admitted by this documentation update. / 本次文档更新不准入 Web 写入。 |

No other roadmap file, source file, manifest, secret, or external service is in this
wave's ownership. / 本 wave 不拥有任何其他 roadmap 文件、source file、manifest、secret 或
external service。

## Evidence Boundary / 证据边界

This record is contract and documentation evidence only. No test, PostgreSQL runtime,
Docker runtime, authenticated browser/visual smoke, Git change-set, remote CI, operator
rehearsal, release, or production result is asserted by this document update. External
runtime and release remain `unobserved`/`deferred`. / 本记录仅是契约与文档证据。本次文档
更新不声称 test、PostgreSQL runtime、Docker runtime、authenticated browser/visual smoke、
Git change-set、remote CI、operator rehearsal、release 或 production 结果。外部 runtime
与 release 继续为 `unobserved`/`deferred`。
