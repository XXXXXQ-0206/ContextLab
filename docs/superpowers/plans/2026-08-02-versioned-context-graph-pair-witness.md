# Versioned Context Graph Pair Witness / 版本化 Context Graph 成对见证

## Necessity Record / 必要性记录

### Named criterion / 对应完成条件

- **Criterion 2 / 条件 2:** exact Context commit history must be replayable and reviewable from one consistent, version-backed snapshot boundary.
  / **条件 2：** 精确的 Context commit history 必须能够从一致的、版本化的 snapshot boundary 回放与审阅。

This increment advances Criterion 2 only as a bounded local evidence and contract record. It does
not close Criterion 2, the remaining completion criteria, or the active long-term goal.

本增量仅作为有界的 local evidence 与 contract record 推进条件 2；不关闭条件 2、其余收束条件或 active long-term goal。

### Current implementation and gap / 当前实现与缺口

The current protected version-backed graph-diff read already returns a server-owned
`pair_witness` containing schema version, project, Context, baseline commit, and revised commit
identity. It rejects missing or identical commit ids, enforces one Context scope, reads persisted
graph snapshots through the existing review service, and delegates graph calculation to the sole
`GraphDiff::between` implementation. The storage pair-review contract also provides one pair read
boundary and rejects invalid or cross-scope records.

当前 protected version-backed graph-diff read 已返回 server-owned 的 `pair_witness`，其中包含 schema version、project、Context、baseline commit 与 revised commit identity。它拒绝缺失或相同的 commit id，强制单一 Context scope，通过既有 review service 读取 persisted graph snapshot，并委托给唯一的 `GraphDiff::between` 实现。storage pair-review contract 也已提供一次 pair read boundary，并拒绝无效或跨 scope record。

The remaining gap is evidence breadth, not a second diff implementation: the observed contract
tests use in-memory or test-owned fixtures. They do not establish a live PostgreSQL write-to-pair-
review round trip, an authenticated browser path, remote CI, or deployment readiness. A passing
local test must therefore be recorded as local contract evidence only.

剩余缺口是 evidence breadth，而不是新增 diff implementation：当前观测到的 contract tests 使用 memory 或 test-owned fixture。它们不能建立 live PostgreSQL write-to-pair-review round trip、authenticated browser path、remote CI 或 deployment readiness。因此，local test 通过只能记为 local contract evidence。

### Why now / 为何现在优先

Pair identity is the smallest evidence boundary attached directly to the existing versioned
Context Graph review. Recording it now prevents later consumers from treating two independently
assembled snapshots as one replayable history, and makes the exact `(project, context, baseline,
revised)` witness reviewable before further Context editing or consumer work. This is narrower and
more dependency-ready than adding a new UI, benchmark execution path, provider, or public write.

Pair identity 是直接附着于既有 versioned Context Graph review 的最小 evidence boundary。现在记录它，可以防止后续 consumer 把两个独立拼装的 snapshot 当作同一条可回放 history，并在继续增加 Context editing 或 consumer 前，让精确的 `(project, context, baseline, revised)` witness 可审阅。它比新增 UI、benchmark execution path、provider 或 public write 更窄，也更依赖就绪。

### Explicit non-goals / 明确非目标

- No new public REST, OpenAPI, public SDK write method, Web mutation, or public transport.
  / 不新增 public REST、OpenAPI、public SDK write method、Web mutation 或 public transport。
- No migration, provider call, secret access, Docker/PostgreSQL runtime claim, or second
  `GraphDiff` calculator.
  / 不新增 migration、provider call、secret access、Docker/PostgreSQL runtime 声明或第二个 `GraphDiff` calculator。
- No claim that in-memory fixtures, ignored runtime tests, or local checks substitute for remote
  CI, operator rehearsal, release, or production evidence.
  / 不声称 memory fixture、ignored runtime test 或 local check 可以替代 remote CI、operator rehearsal、release 或 production evidence。
- No closure of Criterion 2 or the long-term goal; this record is an admission boundary, not a
  completion receipt.
  / 不关闭条件 2 或长期目标；本记录是准入边界，不是完成回执。

### Smallest boundary and ownership / 最小边界与 ownership

This worker owns only this bilingual Necessity Record:
`docs/superpowers/plans/2026-08-02-versioned-context-graph-pair-witness.md`. No Rust, Web, SDK,
storage, migration, roadmap, verification, or governance file is changed by this worker.

本 worker 的 ownership 仅限本双语 Necessity Record：
`docs/superpowers/plans/2026-08-02-versioned-context-graph-pair-witness.md`。本 worker 不修改 Rust、Web、SDK、storage、migration、roadmap、verification 或 governance 文件。

The smallest future implementation boundary, if separately admitted, is the existing Rust-owned
pair snapshot read and its focused contract tests. It must preserve exact scope validation,
one-read atomicity, server-owned pair identity, and delegation to the existing graph-diff service;
public surface changes are outside that boundary.

如果后续单独准入 implementation，最小边界是既有 Rust-owned pair snapshot read 及其 focused contract tests。它必须保留 exact scope validation、one-read atomicity、server-owned pair identity，并继续委托既有 graph-diff service；public surface change 不在该边界内。

### Bilingual documentation and worker-turn verification / 双语文档与 worker 本轮验证

This record is intentionally bilingual and keeps evidence states explicit. The following checks
were run during this worker turn in this workspace on 2026-08-02. They are worker-local results,
not the final integrated verification state.

本记录按要求使用中英双语并明确区分 evidence state。以下检查于 2026-08-02 在本工作区由本 worker
本轮运行。它们属于 worker-local result，不代表最终 integrated verification state。

| Check / 检查 | Worker-turn result / Worker 本轮结果 | Boundary / 边界 |
| --- | --- | --- |
| `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture` | `3 passed; 0 failed` | Protected route authentication, exact scope, stable diff, pair witness, and public-route retirement contract. / protected route authentication、exact scope、stable diff、pair witness 与 public-route retirement contract。 |
| `cargo test -p contextlab-storage --test context_diff_review --offline -- --nocapture` | `9 passed; 0 failed` | Pair read, exact-scope rejection, replay determinism, and delegation contract. / pair read、exact-scope rejection、replay determinism 与 delegation contract。 |
| `rg` source inspection for `pair_witness` and `GraphDiff` ownership | observed matches | Structural inspection only; not runtime or release evidence. / 仅为结构检查，不是 runtime 或 release evidence。 |
| `cargo fmt --all -- --check` | **failed in this worker turn / 本 worker 本轮未通过** | Existing formatting difference at `server/api/src/routes.rs:2358`; this is an intermediate worker result and must remain distinct from any later integrated verification. / 现有格式差异位于该路径；这是 worker 中间结果，必须与后续 integrated verification 分开记录。 |

At the end of this worker turn, full workspace tests, strict Clippy, locked-toolchain check,
`pnpm check:web`, live PostgreSQL, browser/visual E2E, and the unified-diff-input verifier had not
been run by this worker. The focused green tests and the worker-turn `cargo fmt` failure are
historical results from this verification boundary. Any later integrated green verification must
be recorded separately with its exact command, timestamp, and commit or worktree boundary; it
supersedes the worker-turn status only for checks it actually reran and does not establish
unobserved runtime, CI, release, or production evidence.

截至本 worker 本轮结束时，full workspace tests、strict Clippy、locked-toolchain check、
`pnpm check:web`、live PostgreSQL、browser/visual E2E 以及带 unified diff input 的 verifier 均未由本
worker 运行。focused green tests 与本轮 `cargo fmt` 失败都是该验证边界内的历史结果。任何后续
integrated green verification 都必须单独记录其确切命令、时间戳以及 commit 或 worktree boundary；
它只对实际重新运行的检查覆盖本轮状态，不能据此建立尚未观测的 runtime、CI、release 或
production evidence。

### Final integrated verification / 最终 integrated verification

At `2026-08-02T03:33:12.7143168+08:00`, the Integration Lead reran the final local gates against
the current worktree after repairing the API/SDK UTC timestamp contract and tightening SDK error-code
and node-category validation. `cargo test --workspace --quiet` passed with `220 passed, 41 ignored`;
`cargo fmt --all -- --check`, strict offline Clippy, and the locked Rust `1.85.0` check passed;
`pnpm check:web` passed with public SDK `15`, local SDK `141`, Web `294`, and production build;
the fixture contract verifier passed. The scope verifier passed its source/graph/DTO/route checks and
reported `graph_diff_application=passed count=1`, while `overall=unobserved` remained because no
unified diff input was supplied. Git change-set binding is not observed in this worktree.

在 `2026-08-02T03:33:12.7143168+08:00`，Integration Lead 在当前 worktree 中修复 API/SDK UTC timestamp
契约并加强 SDK error-code 与 node-category 校验后，重新运行了最终本地门禁。`cargo test --workspace --quiet`
通过（`220 passed, 41 ignored`）；`cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0`
check 通过；`pnpm check:web` 通过（public SDK `15`、local SDK `141`、Web `294` 与 production build）；
fixture contract verifier 通过。范围 verifier 的 source/graph/DTO/route checks 与
`graph_diff_application=passed count=1` 通过，但因未提供 unified diff input，`overall=unobserved`；本 worktree
未观测 Git change-set binding。

This final result is distinct from the worker-turn `cargo fmt` failure above and does not establish
PostgreSQL runtime, authenticated browser/visual, Git, remote CI, operator, release, or production
evidence. The long-term goal remains active.

该最终结果与上述 worker 本轮 `cargo fmt` 失败相互独立，也不能建立 PostgreSQL runtime、authenticated
browser/visual、Git、remote CI、operator、release 或 production evidence。长期目标保持 active。

### External evidence deferred / 外部证据延期

Remote disposable CI, operator-approved rehearsal, public protected-write promotion, release, and
production evidence remain explicitly `deferred` or `unobserved` because their external operating
conditions are unavailable. They are future deployment prerequisites, not active work for this
record, and local tests must not simulate or substitute for them. The long-term goal remains
active; the next implementation increment requires its own bilingual Necessity Record and fresh
verification.

由于外部运行条件不可用，remote disposable CI、operator-approved rehearsal、public protected-write promotion、release 与 production evidence 继续明确标为 `deferred` 或 `unobserved`。它们是未来 deployment prerequisites，不是本记录的 active work；local tests 不得模拟或替代这些证据。长期目标保持 active；下一项 implementation increment 必须拥有自己的双语 Necessity Record 与 fresh verification。
