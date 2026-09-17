# Private Knowledge/Memory Projection / 私有 Knowledge/Memory 投影

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criterion 1 (Context-first platform coverage) and Criterion 3 (benchmark-driven, inspectable context
inputs) by making the existing provider-free Knowledge citation and Memory retention/replay contracts
consumable as an exact local inspection surface. Context remains the scope boundary; projections must
remain replayable, versioned, deterministic, and safe to present.

本增量直接推进条件 1（Context-first 平台覆盖）与条件 3（可由 benchmark 驱动且可检查的上下文输入），
将现有 provider-free Knowledge citation 与 Memory retention/replay contract 接入精确的本地 inspection surface。
Context 仍是 scope boundary；projection 必须可回放、有版本、确定且适合安全呈现。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** Rust domain contracts
already produce redacted citation and retention capability facts, but they are not exposed through a
stable private application contract or UI path. The gap is transport and composition, not a new
retrieval algorithm. Raw document text, memory bodies, embeddings, provider credentials, and client
policy must not cross the boundary.

Rust domain contract 已能产生脱敏 citation 与 retention capability facts，但尚未通过稳定的 private application
contract 或 UI path 暴露。缺口是 transport 与 composition，不是新增 retrieval algorithm。raw document text、memory
body、embedding、provider credential 与 client policy 都不得跨越边界。

**Why now / 为什么现在优先:** The protected Benchmark execution adapter is locally verified and
the next unclosed Context-first coverage gap with dependency-ready provider-free core contracts is
Knowledge/Memory inspection. Completing this bounded read keeps the existing domain work usable
without waiting for unavailable external deployment evidence or expanding public write surfaces.

protected Benchmark execution adapter 已完成本地验证；下一个依赖就绪且拥有 provider-free core contract 的
Context-first coverage 缺口就是 Knowledge/Memory inspection。完成这条有界 read 可以让既有 domain work 可用，
不依赖不可用的外部部署证据，也不会扩张 public write surface。

**Minimal boundary / 最小边界:** Define one versioned, exact Context-scoped redacted projection
contract in reusable Rust application/storage ports, then add only the private local API/SDK/BFF/Web
read adapters and shared design-system presenter/screen needed to inspect it. Reuse authentication,
RBAC/audit, rate limiting, request-scoped Bearer transport, `private, no-store`, and fail-closed
schema validation. Keep Knowledge and Memory projection data deterministic and provider-free.

定义一份可复用 Rust application/storage port 中的有版本、精确 Context-scoped 脱敏 projection contract，随后只增加
检查它所需的 private local API/SDK/BFF/Web read adapter 与共享 design-system presenter/screen。复用 authentication、
RBAC/audit、rate limit、request-scoped Bearer transport、`private, no-store` 与 fail-closed schema validation。Knowledge
与 Memory projection data 必须确定且 provider-free。

**Explicit non-goals / 明确非目标:** No public REST/OpenAPI/public SDK addition, mutation route,
provider call, raw knowledge or memory content, vector search endpoint, client-side retrieval or
retention policy, operator transport, production claim, or second GraphDiff calculator. `GraphDiff::between`
remains the sole graph-diff calculator.

不新增 public REST/OpenAPI/public SDK，不新增 mutation route、provider call、raw knowledge/memory content、vector search
endpoint、client-side retrieval/retention policy、operator transport 或 production claim，也不增加第二个 GraphDiff
calculator。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Focused
Rust contract tests must prove deterministic identifiers, exact Context scope, schema/version
compatibility, redaction, and fail-closed incompatibility. The private API/SDK/BFF/Web adapters must
then pass focused transport and accessibility tests plus the full Rust/Web gates. PostgreSQL runtime,
authenticated browser runtime, Git, remote CI, operator rehearsal, release, and production evidence
remain separately labeled according to what is actually observed.

聚焦 Rust contract test 必须证明确定性 identifier、精确 Context scope、schema/version compatibility、脱敏与 fail-closed
不兼容行为。随后 private API/SDK/BFF/Web adapter 必须通过 transport/accessibility 聚焦测试及完整 Rust/Web 门禁。
PostgreSQL runtime、authenticated browser runtime、Git、remote CI、operator rehearsal、release 与 production evidence
仍按真实观测结果单独标记。

## Ownership and verification / 所有权与验证

- Rust domain/application contract: `crates/knowledge`, `crates/memory`, and their focused tests.
- Private API/SDK/BFF/Web adapters: only after the Rust contract is green, with disjoint ownership
  assigned in the next integration wave.
- Public REST/OpenAPI/public SDK and `GraphDiff` remain out of scope.

## Closure receipt and next increment / 收束回执与下一增量

**Observed closure / 已观测收束:** The bounded private projection is implemented and locally
verified. The Rust domain and projection contracts preserve deterministic identifiers, exact
Context-scoped replay facts, redacted citation metadata, and fail-closed schema/scope validation.
The API and local SDK bind `source_project_id` and `source_commit_id`; the SDK uses allowlisted
authentication, authorization, and projection errors; the Web adapter presents loading, error, empty,
unavailable, and ready states through the shared presenter/screen boundary with request-race
protection. `content_fingerprint` remains redacted metadata and raw Knowledge/Memory content does
not cross the projection boundary.

本次有界 private projection 已实现并完成本地验证。Rust domain 与 projection contract 保持确定性
identifier、精确 Context-scoped replay facts、脱敏 citation metadata 以及 fail-closed schema/scope
校验。API 与 local SDK 绑定 `source_project_id` 和 `source_commit_id`；SDK 使用 allowlisted 的
authentication、authorization 与 projection error；Web adapter 通过共享 presenter/screen boundary
呈现 loading、error、empty、unavailable 和 ready 状态，并具备 request-race protection。
`content_fingerprint` 仍是脱敏 metadata，raw Knowledge/Memory content 不得越过 projection boundary。

**Fresh receipt / 新鲜回执:** `pnpm check:web` passed (Public SDK 14, Local SDK 85, Web 179, and
production build). Rust workspace tests passed, including API 182 tests and Storage 169 passed with
39 ignored; `cargo fmt --all -- --check` passed; strict offline Clippy passed. Focused API tests
passed (3), Knowledge Context projection tests passed (4), and Knowledge replay bridge tests passed
(9). Docker/PostgreSQL runtime, authenticated browser, visual, Git, remote CI, operator rehearsal,
release, and production evidence were not observed and remain deferred; no secrets were read.

**新鲜回执:** `pnpm check:web` 通过（Public SDK 14、Local SDK 85、Web 179，production build 通过）。
Rust workspace tests 通过，其中 API 182 项、Storage 169 项通过并有 39 项 ignored；
`cargo fmt --all -- --check` 通过；strict offline Clippy 通过。聚焦 API 测试 3 项、Knowledge Context
projection 测试 4 项、Knowledge replay bridge 测试 9 项通过。Docker/PostgreSQL runtime、authenticated
browser、visual、Git、remote CI、operator rehearsal、release 与 production evidence 未观测，继续延期；
未读取 secrets。

**Next Necessity Record / 下一增量必要性记录:** The next admitted increment is private,
read-only exact `(project, Context, commit)` persistence and replay. It is necessary because the
current request-time composition can prove the projection contract but does not yet prove that a
historical commit can be reconstructed from persisted Knowledge/Memory facts. Add a reusable storage
projection repository with Memory/PostgreSQL parity, persist and read the exact project/Context/commit
scope, and test deterministic replay without exposing raw content. This directly advances the
Context-first and replayable-version-history criteria.

下一项准入增量是 private、read-only 的精确 `(project, Context, commit)` persistence 与 replay。
原因是当前 request-time composition 已能证明 projection contract，却尚未证明可从持久化
Knowledge/Memory facts 重建历史 commit。新增可复用的 storage projection repository，保持
Memory/PostgreSQL parity，持久化并读取精确 project/Context/commit scope，并测试不暴露 raw content
的确定性 replay；该增量直接推进 Context-first 与可回放版本历史完成条件。

**Next boundary and non-goals / 下一边界与非目标:** The next slice remains private and read-only:
no public writes, new UI, provider calls, raw content, Docker requirement, production claim, operator
transport, or new GraphDiff implementation. Before admitting the following increment, run focused
repository tests for exact-scope isolation, idempotent replay, Memory/PostgreSQL contract parity where
the local environment permits, then rerun the relevant Rust and Web gates. External release evidence
remains deferred rather than manufactured or treated as a core-development blocker.

下一切片仍为 private、read-only：不新增 public write、新 UI、provider call、raw content、Docker requirement、
production claim、operator transport 或新的 GraphDiff implementation。在进入再下一增量前，运行 exact-scope
isolation、幂等 replay 的 repository 聚焦测试；在本地环境允许时验证 Memory/PostgreSQL contract parity，
随后重跑相关 Rust 与 Web 门禁。外部 release evidence 继续延期，不伪造，也不把它当作核心开发阻塞。

## 2026-07-27 Exact-Commit Persistence Closure / 2026-07-27 精确提交持久化收束

The admitted exact-commit storage increment is implemented. `crates/storage` now owns an immutable
`KnowledgeMemoryProjectionV1Repository` with exact `(project, Context, commit)` scope, deterministic
redacted `KnowledgeMemoryContextProjectionV1` payloads, `Created`/`Replayed`/`Conflict` semantics,
fail-closed schema decoding, and Memory/PostgreSQL adapters. Migration `0022` stores only the
redacted JSON projection, uses append-only protection, and composite foreign keys bind project to
Context and Context to commit. The private API read path now uses the storage-backed adapter in
PostgreSQL mode; the local in-memory adapter seeds once and replays the persisted exact source.

本次准入的 exact-commit storage 增量已实现。`crates/storage` 现拥有不可变的
`KnowledgeMemoryProjectionV1Repository`，强制精确 `(project, Context, commit)` scope、确定性脱敏的
`KnowledgeMemoryContextProjectionV1` payload、`Created`/`Replayed`/`Conflict` 语义与 fail-closed schema
decode，并提供 Memory/PostgreSQL adapter。迁移 `0022` 只保存脱敏 JSON projection，使用 append-only 保护，
并用 composite foreign key 将 project 绑定到 Context、将 Context 绑定到 commit。private API read path 在
PostgreSQL mode 使用 storage-backed adapter；local in-memory adapter 只在首次缺失时 seed，随后 replay
已持久化的 exact source。

Fresh local evidence is focused storage `2/2`, focused API Knowledge/Memory `6/6`, workspace Rust
(`API 182 passed`; `storage 171 passed, 39 ignored`), `cargo fmt --all -- --check`, strict offline
Clippy, and `pnpm check:web` with public SDK `14`, local SDK `85`, Web `179`, and production build.
Migration contract assertions and a scope search confirm no public write/OpenAPI/SDK/Web surface was
added and `GraphDiff::between` remains the sole graph-diff calculator. No secrets were read. `psql`
is unavailable, Docker/virtualization is disabled, and PostgreSQL runtime, authenticated browser,
Git, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or
`deferred`; this increment advances the criteria but does not close them.

新鲜本地证据为 focused storage `2/2`、focused API Knowledge/Memory `6/6`、workspace Rust（`API 182 passed`；
`storage 171 passed, 39 ignored`）、`cargo fmt --all -- --check`、strict offline Clippy，以及
`pnpm check:web`（public SDK `14`、local SDK `85`、Web `179`、production build）。migration contract assertion
与 scope search 证明没有新增 public write/OpenAPI/SDK/Web surface，`GraphDiff::between` 仍是唯一 graph-diff
calculator。未读取 secrets。`psql` 不可用，Docker/virtualization 当前关闭；PostgreSQL runtime、authenticated
browser、Git、remote CI、operator rehearsal、release 与 production evidence 仍为 `unobserved` 或 `deferred`；
本增量推进完成条件但不关闭它们。

**Next admitted increment / 下一项准入增量:** Before implementation, add a bilingual Necessity
Record for the commit-associated ContextGraph snapshot domain/repository contract. It must bind one
snapshot to an immutable Context commit, preserve existing snapshot/branch-head/replay boundaries,
and then expose version-backed comparison only after focused tests and contract verification. No
public write, new GraphDiff calculator, or release claim is admitted by that pointer.

**下一项准入增量：** 在实现前，为 commit-associated ContextGraph snapshot domain/repository contract 增加
双语 Necessity Record。它必须将一份 snapshot 绑定到不可变 Context commit，保持现有 snapshot/branch-head/replay
边界，并且只有在聚焦测试与 contract verification 通过后才接入 version-backed comparison。该指针不准入
public write、新的 GraphDiff calculator 或 release claim。
