# Private Context Diff Pair Read Receipt / 私有 Context Diff 成对读取回执

## Scope / 范围

This receipt covers only the private Rust storage boundary for reading two exact commit-scoped Context diff snapshots consistently. It does not add a route, SDK method, Web mutation, migration, provider, secret access, or public-write claim.

本回执仅覆盖私有 Rust storage boundary：一致读取两个按 commit 精确绑定的 Context diff snapshot。不新增 route、SDK method、Web mutation、migration、provider、secret access 或 public-write 声明。

## Implementation / 实现

- `ContextDiffSnapshotV1Pair` carries immutable source and target records.
- `ContextDiffSnapshotV1PairRepository` is a separate read port, preserving compatibility for the existing single-snapshot repository contract.
- `PersistedContextDiffReviewService` makes one pair-port call, validates both exact scopes/schema versions, and delegates calculation only to `VersionedContextDiffReviewService`.
- Memory adapters use one read lock for both records; PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction for both exact reads.

- `ContextDiffSnapshotV1Pair` 携带 immutable source 与 target record。
- `ContextDiffSnapshotV1PairRepository` 是独立 read port，保留既有 single-snapshot repository contract 的兼容性。
- `PersistedContextDiffReviewService` 只调用一次 pair port，校验两份 exact scope/schema，并且只委托 `VersionedContextDiffReviewService` 计算。
- Memory adapter 对两份 record 使用一个 read lock；PostgreSQL 对两次 exact read 使用一个 `REPEATABLE READ READ ONLY` transaction。

## Fresh evidence / 新鲜证据

| Check / 检查 | Result / 结果 |
| --- | --- |
| `cargo test -p contextlab-storage --test context_diff_review --offline` | `9 passed` |
| `cargo test -p contextlab-storage --test context_diff_snapshot_repository --offline` | `5 passed` |
| `cargo test -p contextlab-storage --test context_diff_snapshot_postgres_contract --offline` | `1 passed, 2 ignored` |
| `cargo test -p contextlab-storage --lib --offline` | `212 passed, 39 ignored` |
| `cargo check -p contextlab-storage --offline` | passed |
| `cargo fmt --all` | passed |

The ignored PostgreSQL tests require a disposable runtime and are not treated as passes. Workspace tests, strict offline Clippy, locked Rust `1.85.0`, Web (`15/134/272 + production build`), local verifier scoped checks, and `GRAPH_DIFF_IMPL_COUNT=1` all passed. The live verifier remains `overall=unobserved` only because no unified diff input was supplied. Docker/virtualization, browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

PostgreSQL ignored test 需要 disposable runtime，不计为通过。Workspace tests、strict offline Clippy、锁定 Rust `1.85.0`、Web（`15/134/272 + production build`）、local verifier scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。live verifier 仅因未提供 unified diff input 而保持 `overall=unobserved`。Docker/virtualization、browser、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。
