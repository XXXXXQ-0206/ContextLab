# Private Benchmark Execution / 私有 Benchmark 执行

## Scope / 范围

ContextLab provides a default-off, protected local execution adapter for one immutable
benchmark-definition binding at one exact project, Context, and commit scope. The route is:

`POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-executions`

ContextLab 提供一条默认关闭、受保护的本地 execution adapter，用于在精确的 project、Context、commit scope
上执行一个不可变 benchmark-definition binding。route 为：

`POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-executions`

This route is installed only by the protected router. It is absent from the default public router,
checked-in OpenAPI, and public TypeScript SDK. It is local development capability, not a release or
production-readiness claim.

该 route 只由 protected router 安装，不进入 default public router、已检入 OpenAPI 或 public TypeScript SDK。
它是本地开发能力，不构成 release 或 production-ready 声明。

## Request / 请求

The JSON body is schema version `1` and contains only server-validated identities and execution
configuration:

JSON body 的 schema version 为 `1`，只包含由服务端校验的 identity 与 execution configuration：

- `binding_id`: immutable definition binding identity / 不可变 definition binding identity
- `decision_id`: immutable decision identity / 不可变 decision identity
- `model_version`, `temperature`: bounded run configuration / 有界 run configuration
- `evaluator_key`, `evaluator_version`: injected evaluator identity / 注入式 evaluator identity

The binding is loaded with the full project/Context/commit path scope. The caller cannot provide or
replace suite, dataset, case, input, expected output, policy, or graph-diff data.

服务端会使用完整 project/Context/commit path scope 加载 binding。caller 不能提供或替换 suite、dataset、case、
input、expected output、policy 或 graph-diff data。

`Idempotency-Key` is required. The API computes a canonical `sha256:` request digest and passes both
typed values through the storage execution service. Memory and PostgreSQL persist the receipt in
`benchmark_execution_idempotency` (migration `0021`) with the exact project/Context/commit scope.

`Idempotency-Key` 必填。API 会计算 canonical `sha256:` request digest，并将两个 typed value 贯穿 storage
execution service。Memory 与 PostgreSQL 会在迁移 `0021` 的 `benchmark_execution_idempotency` 中按精确
project/Context/commit scope 持久化 receipt。

## Response And Errors / 响应与错误

Successful responses contain redacted metadata only: schema, disposition (`created` or `replayed`),
exact scope, suite and dataset identities, decision identity, and the workspace cohort identity.
Every response is `Cache-Control: private, no-store`.

成功响应只包含脱敏 metadata：schema、`created` 或 `replayed` disposition、精确 scope、suite 与 dataset identity、
decision identity 以及 workspace cohort identity。所有 response 都带 `Cache-Control: private, no-store`。

The protected boundary authenticates before quota and requires `ContextPermission::Write`. Missing
authorization, unavailable authorization state, invalid requests, scope misses, evaluator
unavailability, and immutable conflicts map to stable redacted error codes. Evaluator diagnostics,
case payloads, raw inputs, outputs, traces, cookies, and credentials never cross this contract.

受保护边界先 authentication 后 quota，并要求 `ContextPermission::Write`。缺少认证、authorization state 不可用、
无效请求、scope miss、evaluator 不可用与 immutable conflict 都映射为稳定且脱敏的 error code。evaluator diagnostic、
case payload、raw input/output、trace、cookie 与 credential 不会跨过此 contract。

## Replay / 回放

The first request evaluates sealed cases and atomically persists evidence plus its idempotency
receipt. An identical key and digest returns `200` with `replayed`; the evaluator is not called
again. Reusing a key with a different digest returns a conflict before evaluator invocation. Replay
uses the stored evidence decision and timestamp so an HTTP retry does not manufacture a new timestamp
conflict. A missing workspace projection can therefore be repaired from sealed evidence without
recalling the evaluator.

第一次请求执行 sealed case，并原子持久化 evidence 与 idempotency receipt。相同 key 与 digest 返回 `200` 和
`replayed`，不会再次调用 evaluator。不同 digest 重用同一 key 会在 evaluator invocation 前返回 conflict。回放
使用已存储 evidence 的 decision 与 timestamp，HTTP retry 不会人为制造 timestamp conflict。因此缺失的 workspace
projection 可以由 sealed evidence 修复，而无需再次调用 evaluator。

`GraphDiff::between` remains the sole graph-diff calculator. This adapter does not calculate policy,
semantic diff, behavior diff, or evaluation diff in the API or Web layer.

`GraphDiff::between` 仍是唯一 graph-diff calculator。本 adapter 不会在 API 或 Web layer 计算 policy、semantic diff、
behavior diff 或 evaluation diff。

## Evidence Boundary / 证据边界

Fresh local evidence: API execution tests `3 passed`; storage full tests `169 passed, 39 ignored`;
`cargo fmt --all -- --check`; and API/storage compilation. PostgreSQL runtime, authenticated
browser-to-BFF-to-Axum execution, Git change-set evidence, remote CI, operator rehearsal, release,
and production promotion remain `ignored`, `unobserved`, or `deferred` where applicable.

新鲜本地证据：API execution tests `3 passed`；storage 全量 `169 passed, 39 ignored`；`cargo fmt --all -- --check`；
以及 API/storage 编译通过。PostgreSQL runtime、authenticated browser-to-BFF-to-Axum execution、Git change-set
evidence、remote CI、operator rehearsal、release 与 production promotion 继续按实际情况标记为 `ignored`、
`unobserved` 或 `deferred`。
