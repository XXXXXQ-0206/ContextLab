# Private Context-to-Workflow Source Binding Plan / 私有 Context 到 Workflow 源绑定计划

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment directly advances Criterion 1 (Context-first platform coverage), Criterion 2 (replayable versioning), Criterion 4 (shared Context Graph backbone), and Criterion 9 (bilingual documentation). It implements the missing durable provenance link between a sealed Workflow definition revision and one exact Context commit with a materialized graph snapshot.

本增量直接推进条件 1（以 Context 为核心的平台覆盖）、条件 2（可回放版本）、条件 4（共享 Context Graph 骨架）与条件 9（双语文档）。它实现一个尚缺少的持久化溯源链接：将已封存 Workflow definition revision 绑定到一条具有 materialized graph snapshot 的精确 Context commit。

**Unmet gap / 未满足缺口：** `contextlab-workflow` can deterministically define, schedule, and replay a provider-free workflow, but it has no typed Context commit source. `contextlab-storage` has guarded Context commit/snapshot persistence, but no append-only repository contract that proves which immutable Workflow definition revision uses which immutable Context version. The present local workflow capability route correctly reports unavailable because no source repository exists.

`contextlab-workflow` 可以确定性地定义、调度和回放 provider-free workflow，但没有类型化的 Context commit source。`contextlab-storage` 已具备受保护的 Context commit/snapshot 持久化，但没有 append-only repository contract 来证明哪一份不可变 Workflow definition revision 使用哪一个不可变 Context 版本。当前 local workflow capability route 正确返回 unavailable，因为 source repository 尚不存在。

**Why now / 为什么现在做：** Workflow definitions, Context commits, immutable graph snapshots, exact Context scope checks, memory/PostgreSQL storage adapters, and deterministic scheduler replay are already present. A durable source-binding contract is the smallest dependency-ready prerequisite for a later detailed Workflow read path, scheduler integration, API/local SDK adapter, and design-system inspector. Creating those transports first would force them to guess source-version semantics.

Workflow definition、Context commit、不可变 graph snapshot、精确 Context scope check、memory/PostgreSQL storage adapter 与确定性 scheduler replay 均已存在。持久化 source-binding contract 是后续详细 Workflow read path、scheduler integration、API/local SDK adapter 与 design-system inspector 的最小依赖就绪前置。若先创建这些 transport，它们将不得不猜测 source-version 语义。

**Non-goals / 非目标：** No public REST, OpenAPI, public SDK, Web mutation control, operator transport, provider invocation, workflow execution transport, branch merge policy, detached/rebound workflow lifecycle, or second graph-diff calculator. This increment keeps the local capability default-unavailable until a separate protected transport admission record exists; `GraphDiff::between` remains the only graph-diff calculator.

不包含 public REST、OpenAPI、public SDK、Web mutation control、operator transport、provider invocation、workflow execution transport、branch merge policy、解绑或重绑 workflow 生命周期，也不新增第二个 graph-diff calculator。本增量在独立的受保护 transport 准入记录出现前，保持 local capability 默认 unavailable；`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Add a typed, sealed `WorkflowContextBinding` domain record in `crates/workflow`; add a storage repository port plus memory and PostgreSQL-equivalent adapters that accept only Context commits already materialized with a graph snapshot; and add focused architecture/roadmap evidence. The storage record owns atomic append-only persistence and idempotent identical replay. Exact authorization remains a future protected-transport composition concern: this private repository receives the canonical typed Context scope and never resolves a source by workflow ID alone.

在 `crates/workflow` 新增类型化、封存的 `WorkflowContextBinding` domain record；在 storage 中新增 repository port 及 memory/PostgreSQL-equivalent adapter，仅接受已经 materialized graph snapshot 的 Context commit；并补充聚焦的架构与路线图证据。storage record 负责原子、append-only 持久化与相同请求的幂等回放。精确 authorization 仍属于未来 protected-transport composition：该私有 repository 接收规范的类型化 Context scope，绝不只凭 workflow ID 解析 source。

**Fresh verification / 新鲜验证：** Before the next increment, observe red/green focused workflow and storage tests covering exact Context/commit scope, missing snapshot rejection, append-only conflict rejection, identical replay, deterministic listing, and memory/PostgreSQL contract parity. Then run `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, focused API authorization regression tests, and `pnpm check:web`. Docker-backed PostgreSQL runtime, browser E2E, remote CI, operator rehearsal, and production evidence remain unobserved/deferred unless an actual command observes them.

在开始下一增量前，必须观察到红绿聚焦 workflow 与 storage 测试，覆盖精确 Context/commit scope、缺失 snapshot 拒绝、append-only conflict 拒绝、相同请求重放、确定性 listing 以及 memory/PostgreSQL contract parity。随后运行 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、聚焦 API authorization regression test 与 `pnpm check:web`。除非真实命令实际观测到，Docker-backed PostgreSQL runtime、browser E2E、remote CI、operator rehearsal 与 production evidence 均保持 unobserved/deferred。

## Tasks / 任务

- [x] Add the sealed workflow-to-Context commit source-binding domain contract and red tests.
- [x] Add append-only storage port, memory persistence, exact-scope reads, and red/green tests.
- [x] Add PostgreSQL migration/adapter parity and compile-only disposable-test registration.
- [x] Document the private replay boundary and observed verification without claiming transport availability.
- [x] Run full workspace/Web verification, then select the next dependency-ready increment.

## Fresh Verification Record / 新鲜验证记录

- `cargo fmt --all -- --check` passed after formatter normalization.
- `cargo test -p contextlab-workflow --test workflow_context_binding` passed with `2 passed`.
- `cargo test -p contextlab-storage --test workflow_context_binding` passed with `3 passed`.
- `cargo test -p contextlab-storage --quiet` passed with `166 passed, 0 failed, 36 ignored` across the storage crate and its integration targets.
- The PostgreSQL adapter and migration compile as part of the storage crate. Docker-backed PostgreSQL runtime, browser E2E, remote CI, operator rehearsal, release, and production evidence remain unobserved/deferred.
- `cargo test --workspace --quiet` passed with all reported suites at `0 failed`; the storage suite remained `166 passed, 36 ignored`.
- `pnpm check:web` passed with public SDK `14`, local SDK `28`, Web `78`, TypeScript checks, and a production build.
- Wave 3 documentation QA returned `Paths=True,True` and `ExternalSuccessOverclaims=0`.

- `cargo fmt --all -- --check` formatter normalization 后通过。
- `cargo test -p contextlab-workflow --test workflow_context_binding` 通过，结果为 `2 passed`。
- `cargo test -p contextlab-storage --test workflow_context_binding` 通过，结果为 `3 passed`。
- `cargo test -p contextlab-storage --quiet` 通过，storage crate 及其 integration target 合计 `166 passed, 0 failed, 36 ignored`。
- PostgreSQL adapter 与 migration 已随 storage crate 编译。Docker-backed PostgreSQL runtime、browser E2E、remote CI、operator rehearsal、release 与 production evidence 仍为 unobserved/deferred。
- `cargo test --workspace --quiet` 通过，所有报告的 suite 均为 `0 failed`；storage suite 保持 `166 passed, 36 ignored`。
- `pnpm check:web` 通过，public SDK `14`、local SDK `28`、Web `78`，并完成 TypeScript check 与 production build。
- Wave 3 文档 QA 返回 `Paths=True,True` 与 `ExternalSuccessOverclaims=0`。
