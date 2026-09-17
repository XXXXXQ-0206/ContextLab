# Private Context Graph Review Witness / 私有 Context Graph 审查见证

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly advances Criterion 1 (Context-first coverage), Criterion 2 (replayable
version history), and Criterion 4 (Context Graph as the system skeleton). A version-backed graph
review must bind complete commit history, branch heads, and both immutable graph snapshots to one
backend-owned observation boundary before the existing graph review is projected.

本增量直接推进条件 1（Context-first coverage）、条件 2（可回放版本历史）与条件 4（Context Graph 系统骨架）。
版本化 graph review 必须先将完整 commit history、branch heads 与两份 immutable graph snapshot 绑定到后端拥有的同一
观察边界，再投影既有 graph review。

### Unmet dependency, risk, and why now / 未满足依赖、风险与当前优先性

The existing history-bound service validates history first but still reads graph lifecycle facts
through another repository call. That composition can observe different backend states and does not
prove that the reviewed snapshots belong to the same complete history witness. The concrete Memory
and PostgreSQL repositories already own the required state and transaction primitives, so this is
the smallest correctness increment before another Context consumer or editing surface.

现有 history-bound service 虽先校验 history，但仍通过另一个 repository call 读取 graph lifecycle facts。该组合可能观察到不同
backend 状态，不能证明被审阅的 snapshot 属于同一份完整 history witness。Memory 与 PostgreSQL concrete repository 已具备所需
state 与 transaction primitive，因此这是增加其他 Context consumer 或编辑面之前最小的正确性增量。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/SDK method, write route, Web mutation, migration, provider, operator transport, release, or production claim.
- No new diff algorithm. `GraphDiff::between` remains the sole graph-diff calculator.
- No PostgreSQL runtime receipt is manufactured; SQL/static evidence remains distinct from live Docker-backed evidence.

- 不新增 public REST/OpenAPI/SDK method、写入 route、Web mutation、migration、provider、operator transport、release 或生产声明。
- 不新增 Diff 算法；`GraphDiff::between` 继续是唯一 graph-diff calculator。
- 不伪造 PostgreSQL runtime 回执；SQL/static evidence 与 Docker-backed live evidence 保持分离。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

The implementation boundary is the reusable `crates/storage` witness domain/repository contract,
its Memory and PostgreSQL concrete adapters, the existing private graph review adapter, focused
storage/API compatibility tests, and bilingual architecture/roadmap receipts. Existing response,
OpenAPI, local SDK, BFF, and Web shapes remain unchanged unless a compile-only adapter is required.

实现边界限定为可复用 `crates/storage` witness domain/repository contract、Memory 与 PostgreSQL concrete adapter、现有
private graph review adapter、focused storage/API compatibility tests，以及双语 architecture/roadmap 回执。除非编译所需的
adapter 必须调整，既有 response、OpenAPI、local SDK、BFF 与 Web shape 保持不变。

### Fresh verification gate / 新鲜验证门槛

First observe a red contract proving that separate history/snapshot reads can be inconsistent, then
obtain green Memory witness tests and PostgreSQL SQL-shape tests. Before the next increment, run
focused API/storage tests, workspace Rust, format, strict offline Clippy, locked Rust 1.85, Web
checks, the local contract verifier, and the exact-one `GraphDiff` source check. Live PostgreSQL,
Docker, browser/visual, Git, remote CI, operator, release, and production remain unobserved or deferred.

先观测证明 history/snapshot 分读可能不一致的红 contract，再取得 Memory witness 与 PostgreSQL SQL-shape 的绿灯。下一增量前运行
API/storage focused tests、workspace Rust、格式、strict offline Clippy、锁定 Rust 1.85、Web checks、local contract verifier 与
exact-one `GraphDiff` source check。live PostgreSQL、Docker、browser/visual、Git、remote CI、operator、release 与 production
继续为未观测或延期。

## Implementation boundary / 实施边界

The repository witness returns complete `CommitHistory` (including branch heads) and exact source
and target `CommitGraphSnapshot` values. A separate application adapter validates scope and delegates
the pair to the existing versioned graph review service; storage never calculates a diff.

repository witness 返回包含 branch heads 的完整 `CommitHistory` 以及 exact source/target `CommitGraphSnapshot`。独立
application adapter 校验 scope 后将 pair 委托给既有 versioned graph review service；storage 不计算 Diff。

## Status / 状态

`completed / verified locally`。长期目标保持 `active`；本计划的完成不等于 ContextLab 项目完成。

## Completion receipt / 完成回执

The Memory and PostgreSQL adapters now return one validated history-plus-snapshot witness. The
protected version-backed route consumes only that witness repository and fails closed when the
dependency is absent; direct tests inject it explicitly. The PostgreSQL SQL-shape assertion is
whitespace-insensitive so formatting does not weaken the transaction contract. / Memory 与 PostgreSQL
adapter 现返回一个经过校验的 history-plus-snapshot witness。protected version-backed route 只消费该 witness repository，
依赖缺失时 fail closed；直接测试显式注入 witness。PostgreSQL SQL-shape assertion 已改为不依赖空白格式，避免格式变化削弱
transaction contract。

Fresh local evidence / 新鲜本地证据（2026-08-02T15:02:14+08:00 receipt window / 回执窗口）：

- storage composition: `4 passed`;
- PostgreSQL contract: `2 passed, 1 ignored` (live runtime requires disposable PostgreSQL);
- protected API graph-diff focused: `7 passed`;
- API exact-scope integration: `4 passed`, including missing-witness fail-closed;
- workspace Rust: `cargo test --workspace --quiet --offline -j 1` passed, storage `233 passed, 41 ignored`;
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` check passed;
- `pnpm check:web`: public SDK `15`, local SDK `148`, Web `298`, production build passed;
- `tests/contract/verify-local-contracts.test.ps1` fixture tests passed;
- source inspection: exactly one production `impl GraphDiff`.

本地非生产证据（2026-08-02T15:02:14+08:00 回执窗口）：storage composition `4 passed`；PostgreSQL contract
`2 passed, 1 ignored`（live runtime 需要 disposable PostgreSQL）；protected API graph-diff focused `7 passed`；API
exact-scope integration `4 passed`（包含 missing-witness fail-closed）；workspace Rust 通过且 storage 为
`233 passed, 41 ignored`；`cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` check 通过；
`pnpm check:web` 的 public SDK `15`、local SDK `148`、Web `298` 与 production build 通过；
`tests/contract/verify-local-contracts.test.ps1` fixture tests 通过；源码检查确认唯一一个 production `impl GraphDiff`。

PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and
production remain `ignored`, `unobserved`, or `deferred`. No public write, migration, provider, secret access,
operator transport, Web mutation, or second graph-diff calculator was added. / PostgreSQL runtime、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。
未新增 public write、migration、provider、secret access、operator transport、Web mutation 或第二个 graph-diff calculator。
