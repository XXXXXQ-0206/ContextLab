# Private Knowledge/Memory PostgreSQL Runtime Receipt / 私有 Knowledge/Memory PostgreSQL 运行时回执

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与宪章原则

This increment directly advances Criterion 1 (Context-first platform coverage) and Criterion 8
(reliable release gates) by producing fresh local runtime evidence for the already-composed,
private, provider-free Knowledge/Memory projection path. It does not close either criterion or
the active long-term goal.

本增量直接推进条件 1（Context-first platform coverage）与条件 8（可靠 release gate），为已经组合完成的 private、provider-free Knowledge/Memory projection path 生成新鲜本地运行时证据。不关闭任一条件或 active long-term goal。

### Gap and dependencies / 缺口与依赖

The domain contract, deterministic memory adapter, PostgreSQL adapter, migration `0022`, private
API route, local SDK, BFF, and Web inspection path already exist. The remaining local evidence gap
is a fresh disposable PostgreSQL receipt proving `Created`, identical `Replayed`, cross-connection
read, immutable conflict, exact-scope rejection, and raw-content rejection for the projection.

domain contract、确定性 memory adapter、PostgreSQL adapter、`0022` migration、private API route、local SDK、BFF 与 Web inspection path 均已存在。当前本地证据缺口是新鲜 disposable PostgreSQL receipt，证明该 projection 的 `Created`、相同 `Replayed`、跨连接读取、immutable conflict、exact-scope rejection 与 raw-content rejection。

Dependencies are existing storage contracts, `PostgresContextGraphRepository`, migration `0022`,
and an explicitly supplied loopback disposable PostgreSQL URL. No repository `.env`, credential,
or secret is read.

依赖是既有 storage contract、`PostgresContextGraphRepository`、`0022` migration 与显式提供的 loopback disposable PostgreSQL URL。不读取仓库 `.env`、credential 或 secret。

### Why now / 为什么现在优先

This is the smallest dependency-ready increment after the workflow-status app composition. It
consumes existing contracts without expanding product surface and closes a named local runtime
evidence gap before any larger API/browser or provider work.

这是 workflow-status app composition 之后最小的依赖就绪增量。它只消费既有 contract，不扩大产品 surface，并在更大的 API/browser 或 provider 工作之前收束一个已命名的本地运行时证据缺口。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK surface or write method.
- No Web mutation, provider/network call, scheduler, execution producer, operator transport, release, production, authenticated browser, visual smoke, or Git claim.
- No raw knowledge/memory content, query, vector, secret, or second `GraphDiff` calculator.
- No migration change unless the fresh fixture exposes a real root cause; any such repair must be recorded with a red regression and a new receipt.

- 不新增 public REST/OpenAPI/public SDK surface 或写方法。
- 不新增 Web mutation、provider/network call、scheduler、execution producer、operator transport、release、production、authenticated browser、visual smoke 或 Git 声明。
- 不保存或返回 raw knowledge/memory content、query、vector、secret，也不新增第二个 `GraphDiff` calculator。
- 除非 fresh fixture 暴露真实根因，否则不修改 migration；若需修复，必须记录红回归与新回执。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

The implementation ownership is limited to one storage integration-test fixture under
`crates/storage/tests/` and its focused verification. The Integration Lead owns any shared helper
or migration-root correction, the final quality matrix, and bilingual roadmap/completion/parallel
receipts. No API, SDK, Web, or domain production code changes are admitted.

实现 ownership 仅限 `crates/storage/tests/` 下一个 storage integration-test fixture 及 focused verification。Integration Lead 负责任何 shared helper 或 migration-root correction、最终质量矩阵与双语 roadmap/completion/parallel receipt。不准入 API、SDK、Web 或 domain production code change。

### Fresh verification before the next increment / 下一增量前的新鲜验证

Require a fresh disposable PostgreSQL run covering migration application, deterministic seed,
`Created`, identical `Replayed`, second-connection read, immutable conflict, exact-scope rejection,
and raw-content rejection. Then run the focused storage contract, workspace Rust, format, strict
offline Clippy, locked Rust 1.85 check, `pnpm check:web`, and the existing local contract verifier.
Local PostgreSQL evidence remains local-only; remote CI, operator rehearsal, release, and
production remain `unobserved` or `deferred`.

必须进行新鲜 disposable PostgreSQL 运行，覆盖 migration application、确定性 seed、`Created`、相同 `Replayed`、第二连接读取、immutable conflict、exact-scope rejection 与 raw-content rejection。随后运行 focused storage contract、workspace Rust、format、strict offline Clippy、锁定 Rust 1.85 check、`pnpm check:web` 与既有 local contract verifier。本地 PostgreSQL evidence 仅限本地；remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

## Execution checklist / 执行清单

- [x] Add the opt-in ignored PostgreSQL integration fixture without reading secrets.
- [x] Observe the fresh disposable run or record the precise environment gap; do not manufacture runtime evidence.
- [x] Run the focused and full local verification matrix.
- [x] Update bilingual roadmap/completion/parallel receipts and choose the next active increment; keep the long-term goal active.

- [x] 增加不读取 secrets 的 opt-in ignored PostgreSQL integration fixture。
- [x] 观测 fresh disposable run，或记录精确环境缺口；不得制造 runtime evidence。
- [x] 运行 focused 与完整本地验证矩阵。
- [x] 更新双语 roadmap/completion/parallel receipt 并选择下一项 active 增量；保持长期目标 active。

## Completion receipt / 完成回执

The fixture is complete and locally verified. The ignored test accepts only a loopback PostgreSQL
host, applies the existing migration and seed, and proves immutable `Created`/`Replayed`, a
cross-connection read, redacted stored JSON, exact project/context/commit scope enforcement, an
immutable conflict, and fail-closed decoding of a malformed stored projection. It does not change
production or migration assets.

fixture 已完成并通过本地验证。ignored test 只接受 loopback PostgreSQL host，应用既有 migration 与 seed，并证明 immutable `Created`/`Replayed`、跨连接读取、脱敏 stored JSON、exact project/context/commit scope enforcement、immutable conflict 与 malformed stored projection 的 fail-closed decode。不修改 production 或 migration asset。

Fresh evidence / 新鲜证据：

- Fresh local loopback PostgreSQL 16 cluster: `postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay` -> `1 passed`.
- Without an explicitly supplied URL, the same test is `1 ignored` by design.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `218 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web` (Web `284/284` plus production build), and `pwsh -NoProfile -File .\tests\contract\verify-local-contracts.test.ps1`: passed.

新鲜证据：

- fresh local loopback PostgreSQL 16 cluster：`postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay` -> `1 passed`。
- 未显式提供 URL 时，同一 test 按设计为 `1 ignored`。
- `cargo test --workspace --quiet --no-fail-fast --offline` 通过；storage `218 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`（Web `284/284` 与 production build）及 `pwsh -NoProfile -File .\tests\contract\verify-local-contracts.test.ps1` 通过。

The temporary cluster was stopped after the receipt. The tool policy rejected recursive removal of
the verified temporary data directory, so filesystem cleanup is recorded as `unobserved`; no
database process remains running. This is local non-production evidence only. Remote CI, operator
rehearsal, release, production, authenticated browser/visual smoke, and Git evidence remain
`unobserved` or `deferred`; the long-term goal remains `active`.

临时 cluster 在回执后已停止。工具策略拒绝删除已核验的临时 data directory，因此 filesystem cleanup 记录为 `unobserved`；没有数据库进程继续运行。本证据仅限本地非生产环境。remote CI、operator rehearsal、release、production、authenticated browser/visual smoke 与 Git evidence 继续为 `unobserved` 或 `deferred`；长期目标保持 `active`。
