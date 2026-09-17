# Private Benchmark Execution Boundary / 私有 Benchmark 执行边界

The local benchmark execution path keeps business logic in reusable Rust storage/application
contracts and treats Axum as an adapter:

本地 benchmark execution path 将业务逻辑保留在可复用 Rust storage/application contract 中，Axum 只作为 adapter：

```text
protected Axum route
    -> authentication / RBAC / audit / quota
    -> exact binding repository lookup
    -> StorageBenchmarkExecutionAdapter
    -> BenchmarkExecutionService
    -> sealed evaluator port
    -> BenchmarkEvidenceWriter + workspace projection writer
```

The route requires `ContextPermission::Write` because execution persists immutable evidence and a
workspace projection. Authorization happens before body parsing and protected-route quota. The
evaluator is injected; an absent evaluator is a service-unavailable failure with no evidence write.

该 route 要求 `ContextPermission::Write`，因为 execution 会持久化 immutable evidence 与 workspace projection。
authorization 先于 body parsing 与 protected-route quota。evaluator 通过 port 注入；evaluator 缺失时返回
service-unavailable，且不写入 evidence。

## Durable Idempotency / 持久化幂等

`BenchmarkExecutionRequest` carries optional typed `IdempotencyKey` and `RequestDigest` values.
`PersistBenchmarkEvaluationEvidence` carries the same pair to the writer. Memory and PostgreSQL
implement the read receipt contract and persist `benchmark_execution_idempotency` in migration
`0021_benchmark_execution_idempotency.sql`. The receipt is append-only and foreign-keyed to the
exact sealed decision evidence.

`BenchmarkExecutionRequest` 携带可选 typed `IdempotencyKey` 与 `RequestDigest`。`PersistBenchmarkEvaluationEvidence`
将同一 pair 传入 writer。Memory 与 PostgreSQL 实现 receipt read contract，并在迁移
`0021_benchmark_execution_idempotency.sql` 中持久化 `benchmark_execution_idempotency`。receipt append-only，
并通过 foreign key 绑定精确的 sealed decision evidence。

The service first checks the exact receipt. Matching digest replays stored evidence and normalizes
the replay request to the stored decision/timestamp. A changed digest fails closed. The writer keeps
evidence, its receipt, sealed runs, and definitions in the same storage transaction; projection
materialization remains an evidence-backed follow-up that can be repaired by replay.

service 会先检查精确 receipt。digest 匹配时回放已存储 evidence，并将 replay request 规范化到已存储的
decision/timestamp；digest 变化则 fail closed。writer 在同一 storage transaction 中保持 evidence、receipt、
sealed run 与 definition 的一致性；projection materialization 仍是 evidence-backed follow-up，可由 replay 修复。

No public write contract is introduced. The default router, OpenAPI, public SDK, provider gateway,
operator transport, and Web mutation controls remain unchanged. `GraphDiff::between` remains the
only graph-diff calculator.

没有新增 public write contract。default router、OpenAPI、public SDK、provider gateway、operator transport 与
Web mutation control 均保持不变。`GraphDiff::between` 仍是唯一 graph-diff calculator。
