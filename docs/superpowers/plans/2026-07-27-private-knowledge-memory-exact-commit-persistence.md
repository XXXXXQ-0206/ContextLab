# Private Knowledge/Memory Exact-Commit Persistence / 私有 Knowledge/Memory 精确提交持久化

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criterion 1 (Context-first platform coverage), Criterion 2 (replayable version history), and
Criterion 5 (persistent, reusable Rust storage boundaries). A Knowledge/Memory inspection must be
derived from one immutable `(project, Context, commit)` fact set so that replay and later comparison
do not depend on request-time fixtures.

本增量直接推进条件 1（Context-first 平台覆盖）、条件 2（可回放版本历史）与条件 5（持久化、可复用的
Rust storage boundary）。Knowledge/Memory inspection 必须来自一个不可变的 `(project, Context, commit)`
事实集，回放与后续比较不能依赖请求时 fixture。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The current private
projection repository reconstructs deterministic Knowledge and Memory state on every read. That
keeps redaction tests green but cannot prove exact-commit persistence, replay identity, or parity
between memory and PostgreSQL. The minimum missing contract is a storage-owned immutable redacted
projection source with exact scope validation and replay-safe reads.

当前 private projection repository 在每次 read 时重新构造 deterministic Knowledge 与 Memory state。虽然
脱敏测试保持通过，但无法证明 exact-commit persistence、replay identity 或 Memory/PostgreSQL parity。当前
最小缺口是 storage-owned 的不可变脱敏 projection source，具备精确 scope 校验与可回放 read contract。

**Why now / 为什么现在优先:** The provider-free domain, redaction bridge, private API/SDK/BFF/Web
read adapters, and exact source envelope are already locally verified. This is the nearest direct
completion gap before adding any broader Knowledge/Memory workflow, and it reuses existing immutable
benchmark projection and Context commit storage patterns.

provider-free domain、脱敏 bridge、private API/SDK/BFF/Web read adapter 与 exact source envelope 已完成本地
验证。这是扩展 Knowledge/Memory workflow 前最近且直接的收束缺口，并可复用现有 immutable benchmark
projection 与 Context commit storage pattern。

**Minimal boundary / 最小边界:** Add a reusable Rust storage contract and in-memory/PostgreSQL
adapters for one immutable redacted Knowledge/Memory projection at exact project, Context, and
commit scope. Replace only the server's private repository composition behind the existing read
adapter after focused storage tests are green. Preserve deterministic ordering, schema/version
validation, source identity, and replay behavior.

增加可复用 Rust storage contract，以及 Memory/PostgreSQL 两个 adapter，为精确 project、Context、commit
scope 保存一份不可变脱敏 Knowledge/Memory projection。只有聚焦 storage test 通过后，才在现有 private
read adapter 后替换 server composition。保持确定性排序、schema/version 校验、source identity 与 replay
语义。

**Explicit non-goals / 明确非目标:** No public REST/OpenAPI/public SDK write, no mutation route,
no provider call, no raw document or memory body, no vector search endpoint, no client-side policy,
no new UI, no Docker or production claim, no external receipt work, and no second GraphDiff
calculator. `GraphDiff::between` remains the sole graph-diff calculator.

不新增 public REST/OpenAPI/public SDK write、mutation route、provider call、raw document 或 memory body、
vector search endpoint、client-side policy、新 UI、Docker/production claim 或外部回执工作，也不增加第二个
GraphDiff calculator。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Expected bilingual documentation / 预期双语文档:** Update this plan, the active long-term goal,
completion criteria, and parallel-development ledger with the real receipt and evidence boundary.

更新本计划、active long-term goal、completion criteria 与 parallel-development ledger，记录真实回执及证据边界。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Add
focused red/green contract tests for exact scope, deterministic source identity, immutable conflict,
replay, schema fail-closed behavior, and Memory/PostgreSQL parity. Then run `cargo fmt --all
-- --check`, focused storage tests, workspace Rust tests, strict Clippy, and `pnpm check:web`.
PostgreSQL runtime, authenticated browser runtime, Git, remote CI, operator rehearsal, release, and
production remain separately labeled according to observed evidence.

增加聚焦红绿 contract test，覆盖 exact scope、deterministic source identity、immutable conflict、replay、
schema fail-closed 与 Memory/PostgreSQL parity。随后运行 `cargo fmt --all -- --check`、聚焦 storage test、
workspace Rust test、strict Clippy 与 `pnpm check:web`。PostgreSQL runtime、authenticated browser runtime、
Git、remote CI、operator rehearsal、release 与 production 仍按真实观测单独标记。

## Implementation Boundary / 实施边界

- Rust contract and adapters: `crates/storage/src/knowledge_memory_projection.rs`, its focused tests,
  and only the required `lib.rs`/repository delegations.
- Server composition: `server/api/src/knowledge_memory.rs` and the narrow repository wiring only
  after storage contract tests pass.
- No public REST/OpenAPI/public SDK surface changes.

- Rust contract 与 adapter：`crates/storage/src/knowledge_memory_projection.rs`、聚焦测试，以及必要的
  `lib.rs`/repository delegation。
- Server composition：storage contract test 通过后，仅修改 `server/api/src/knowledge_memory.rs` 与窄范围
  repository wiring。
- 不修改 public REST/OpenAPI/public SDK surface。
