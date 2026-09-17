# Private Benchmark Multi-Dataset PostgreSQL Breadth Receipt / 私有 Benchmark 多 Dataset PostgreSQL 宽度回执

## Necessity Record / 必要性记录

### Named completion criterion and charter principle / 命名完成条件与宪章原则

This increment directly supplies fresh local evidence for Criterion 3, Benchmark-driven
evaluation. The existing local benchmark domain and PostgreSQL workspace projection already
support sealed datasets, suites, runs, scorecards, regression thresholds, and evaluation-diff
projections; the missing evidence is that the multi-dataset/four-case breadth survives the real
PostgreSQL persistence and exact read boundary. This preserves Context-first, deterministic
evaluation, and provider-independent core principles without closing Criterion 3 or the active
long-term goal.

本增量直接为条件 3（Benchmark-driven evaluation）补充新鲜本地证据。现有 local benchmark domain 与
PostgreSQL workspace projection 已支持 sealed dataset、suite、run、scorecard、regression threshold 与
evaluation-diff projection；当前缺口是证明 multi-dataset/four-case breadth 能穿过真实 PostgreSQL
persistence 与 exact read boundary。本增量保持 Context-first、确定性评测与 provider-independent core
原则，但不关闭条件 3 或 active long-term goal。

### Unmet evidence, risk, and dependencies / 未满足证据、风险与依赖

The existing in-memory/static breadth receipt and the PostgreSQL receipts prove different
boundaries. The earlier port `55439` receipt persisted and read `BenchmarkDecisionEvidence`
directly, so it represents only direct decision/run persistence. It does not prove the sealed
workspace projection, dataset-case provenance, replay disposition, exact scope, or redaction
boundary. The corrected port `55441` receipt is the required projection-level evidence for two
datasets, four cases, baseline/revised evidence, scorecard coverage, and evaluation-diff facts
after PostgreSQL round-trip. Without that corrected receipt, Criterion 3 has a named breadth
evidence gap. The implementation must reuse existing sealed projection contracts and deterministic
fixtures; no provider, network, or secret is a dependency.

现有 in-memory/static breadth receipt 与 PostgreSQL receipts 覆盖的是不同边界。早期 port `55439` receipt
直接持久化并读取 `BenchmarkDecisionEvidence`，因此只代表 direct decision/run persistence，不能证明 sealed
workspace projection、dataset-case provenance、replay disposition、exact scope 或 redaction boundary。修正后的
port `55441` receipt 才是 two datasets、four cases、baseline/revised evidence、scorecard coverage 与
evaluation-diff facts 经过 PostgreSQL round-trip 后的 projection-level evidence。没有这份修正后的 receipt，
条件 3 仍有具名 breadth evidence gap。实现必须复用既有 sealed projection contract 与 deterministic fixture；
不依赖 provider、network 或 secret。

### Why now / 当前优先原因

This is the smallest dependency-ready evidence increment after the completed benchmark
persistence runtime receipt. It directly closes a named local Criterion 3 evidence gap and is
narrower than adding another benchmark route, SDK method, Web control, provider adapter, or
execution scheduler. It also provides a trustworthy storage foundation for later benchmark UI
and evaluation-diff work.

这是 benchmark persistence runtime receipt 完成后的最小依赖就绪 evidence 增量，直接收束条件 3 的具名 local
evidence gap；它比新增 benchmark route、SDK method、Web control、provider adapter 或 execution scheduler 更小，
也为后续 benchmark UI 与 evaluation-diff 工作提供可信的 storage foundation。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK write or read expansion, Web mutation, provider call,
  scheduler, migration, operator transport, second `GraphDiff` calculator, secret or `.env` read,
  Docker, browser, Git, remote CI, release, or production claim.
- No change to benchmark domain semantics, sealed projection DTOs, raw-case redaction, or API/SDK/Web
  contracts. If a test-local helper is needed, it must stay inside the PostgreSQL test boundary.
- A local PostgreSQL receipt is not public-write readiness, production readiness, or external release
  evidence; temporary-directory cleanup remains separately classified.

- 不新增 public REST/OpenAPI/public SDK write 或 read surface、Web mutation、provider call、scheduler、migration、
  operator transport、第二个 `GraphDiff` calculator、secret 或 `.env` read、Docker、browser、Git、remote CI、release
  或 production 声明。
- 不改变 benchmark domain semantics、sealed projection DTO、raw-case redaction 或 API/SDK/Web contract。若需要
  test-local helper，必须留在 PostgreSQL test boundary 内。
- local PostgreSQL receipt 不等于 public-write readiness、production readiness 或 external release evidence；临时目录
  cleanup 必须独立分类。

### Smallest affected boundary and bilingual documentation / 最小影响边界与双语文档

The smallest code boundary is the existing ignored PostgreSQL benchmark test module and its
test-only fixture helper. The plan, active long-term goal, completion audit, and parallel
development ledger must record the bilingual evidence and provenance. No production adapter,
migration, API, SDK, or Web file is in scope unless a failing test proves an exactly minimal
test-boundary repair is required.

最小代码边界是既有 ignored PostgreSQL benchmark test module 与 test-only fixture helper。计划、active
long-term goal、completion audit 与 parallel development ledger 必须记录双语 evidence 与 provenance。除非失败
测试证明必须进行严格最小的 test-boundary 修复，否则 production adapter、migration、API、SDK 与 Web 文件均不在范围内。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First run the breadth test without a configured URL and record the intentional `ignored` result if
applicable. Then run it against one fresh empty loopback PostgreSQL database, with a separate fresh
database for every other ignored runtime test. The receipt must observe exact two-dataset/four-case
projection, baseline/revised scope, scorecard threshold/coverage, evaluation-diff evidence, raw
payload redaction, immutable replay, and wrong-scope rejection. Afterward run focused storage
contracts, workspace Rust, format, strict offline Clippy, locked Rust 1.85, `pnpm check:web`, and
the local contract verifier. Stop temporary processes and classify directory cleanup separately.

先在未配置 URL 时运行 breadth test，并在适用时记录设计上的 `ignored`。随后在一个 fresh empty loopback
PostgreSQL database 上运行它；其他 ignored runtime test 必须各自使用独立 fresh database。回执必须观测 exact
two-dataset/four-case projection、baseline/revised scope、scorecard threshold/coverage、evaluation-diff evidence、
raw payload redaction、immutable replay 与 wrong-scope rejection。随后运行 focused storage contract、workspace Rust、
format、strict offline Clippy、锁定 Rust 1.85、`pnpm check:web` 与 local contract verifier。临时 process 停止后，
directory cleanup 单独分类。

### Root-cause correction recorded during verification / 验证期间记录的根因修正

The first breadth test on port `55439` was narrower than this record stated: it persisted and read
`BenchmarkDecisionEvidence` directly. An independent review found that it did not exercise the
`BenchmarkWorkspaceProjectionV1` write/read boundary, projection redaction, replay disposition,
wrong-scope rejection, or the exact dataset-case provenance after PostgreSQL round-trip. The
corrected test-only projection assertions were then observed on port `55441`; only that receipt
supports the full projection/provenance/replay/scope/redaction boundary. The earlier `55439`
receipt remains direct decision/run evidence only.

port `55439` 上的首个 breadth test 比本记录所述边界更窄：它直接持久化并读取 `BenchmarkDecisionEvidence`。
独立复核发现，它没有执行 `BenchmarkWorkspaceProjectionV1` write/read boundary、projection redaction、replay
disposition、wrong-scope rejection，或 PostgreSQL round-trip 后的精确 dataset-case provenance。随后在 port `55441`
观测到修正后的 test-only projection assertions；只有该 receipt 支持完整的 projection/provenance/replay/scope/
redaction boundary。早期 `55439` receipt 仍仅是 direct decision/run evidence。

The minimal admissible fix is confined to `crates/storage/src/postgres.rs` test helpers and one
ignored PostgreSQL test. It constructs the existing sealed projection command, persists both
baseline and revised receipts, reads the comparison projection, asserts four case links and safe
redaction, replays one identical command, and rejects a wrong project scope. No new transport or
domain contract is admitted. The next fresh PostgreSQL run must observe the focused test as
`1 passed`; no-URL remains intentionally `1 ignored`.

最小准入修复仅限 `crates/storage/src/postgres.rs` 的 test helper 与一个 ignored PostgreSQL test。它构造既有
sealed projection command，持久化 baseline 与 revised receipt，读取 comparison projection，断言四条 case link 与
安全 redaction，重放一次相同 command，并拒绝错误 project scope。不新增 transport 或 domain contract。下一次
新鲜 PostgreSQL run 必须观测 focused test `1 passed`；无 URL 仍按设计为 `1 ignored`。

## Execution checklist / 执行清单

- [x] Add or adapt only a test-local PostgreSQL breadth fixture and red/green assertions.
- [x] Correct the receipt boundary with test-only projection, replay, scope, provenance, and redaction assertions.
- [x] Run no-URL ignored and fresh loopback PostgreSQL breadth verification with independent databases.
- [x] Run cross-stack quality gates and the local contract verifier.
- [x] Update bilingual roadmap/completion/parallel receipts; keep the long-term goal `active`.
- [x] Re-audit the next dependency-ready named criterion before any further implementation.

- [x] 仅新增或调整 test-local PostgreSQL breadth fixture 与红/绿断言。
- [x] 通过 test-only projection、replay、scope、provenance 与 redaction assertion 修正回执边界。
- [x] 运行 no-URL ignored 与 fresh loopback PostgreSQL breadth verification，并使用独立 database。
- [x] 运行跨栈质量 gate 与 local contract verifier。
- [x] 更新双语 roadmap/completion/parallel receipt；保持长期目标为 `active`。
- [x] 任何后续实现前重新审计下一项依赖就绪的命名条件。

## Fresh receipt / 新鲜回执

- The test-only PostgreSQL breadth fixture is in `crates/storage/src/postgres.rs`; no production
  adapter, migration, transport, or projection contract changed.
- Without a configured test URL, the focused breadth run observed the intentional `1 ignored` result.
- Against a fresh loopback PostgreSQL 16 cluster on port `55439`, the earlier direct
  decision/run persistence test observed `1 passed`; this receipt does not carry the full
  projection/provenance/replay/scope/redaction coverage.
- Against a separate fresh loopback PostgreSQL 16 cluster on port `55441`, the corrected
  test-only projection breadth receipt observed `1 passed`. It covered two datasets, four cases,
  baseline/revised commit scopes, scorecard threshold and coverage, evaluation-diff evidence,
  replay/readback, exact-scope rejection, and redacted payloads.
- The temporary PostgreSQL processes were stopped. Temporary-directory cleanup remains `unobserved`;
  the existing PostgreSQL service on port `5432` was neither inspected nor used.
- `cargo fmt --all -- --check`, strict offline workspace Clippy, locked Rust `1.85.0` checks,
  `cargo test --workspace --quiet --no-fail-fast --offline` (`220 passed`, `40 ignored`),
  `pnpm check:web` (`15` public SDK, `135` local SDK, `284/284` Web, production build),
  `tests/contract/verify-local-contracts.test.ps1`, and `scripts/verify-local-contracts.ps1`
  all passed. The static verifier correctly classified missing unified-diff, Git, browser, and
  production receipts as `unobserved`.

- test-only PostgreSQL breadth fixture 位于 `crates/storage/src/postgres.rs`；未改变 production
  adapter、migration、transport 或 projection contract。
- 未配置 test URL 时，focused breadth run 观测到设计预期的 `1 ignored`。
- 在 port `55439` 的 fresh loopback PostgreSQL 16 cluster 上，早期 direct decision/run persistence test
  观测到 `1 passed`；该 receipt 不承载完整 projection/provenance/replay/scope/redaction coverage。
- 在独立的 port `55441` fresh loopback PostgreSQL 16 cluster 上，修正后的 test-only projection breadth
  receipt 观测到 `1 passed`，覆盖 two datasets、four cases、baseline/revised commit scope、scorecard
  threshold/coverage、evaluation-diff evidence、replay/readback、exact-scope rejection 与 redacted payload。
- 临时 PostgreSQL processes 已停止；临时目录 cleanup 仍为 `unobserved`；既有 port `5432` 的 PostgreSQL
  未被检查或使用。
- `cargo fmt --all -- --check`、strict offline workspace Clippy、锁定 Rust `1.85.0` check、
  `cargo test --workspace --quiet --no-fail-fast --offline`（`220 passed`、`40 ignored`）、
  `pnpm check:web`（public SDK `15`、local SDK `135`、Web `284/284`、production build）、
  `tests/contract/verify-local-contracts.test.ps1` 与 `scripts/verify-local-contracts.ps1` 均通过。
  static verifier 正确将缺少 unified-diff、Git、browser 与 production receipt 标为 `unobserved`。

This is local non-production evidence only. Remote CI, operator rehearsal, release, production,
authenticated browser/visual smoke, Git change-set, and filesystem cleanup remain `unobserved` or
deferred. The long-term goal remains `active`; this receipt does not close Criterion 3 or the project.

本回执仅是本地非生产证据。remote CI、operator rehearsal、release、production、authenticated
browser/visual smoke、Git change-set 与 filesystem cleanup 继续为 `unobserved` 或 `deferred`。长期目标
保持 `active`；本回执不关闭条件 3，也不关闭项目。
