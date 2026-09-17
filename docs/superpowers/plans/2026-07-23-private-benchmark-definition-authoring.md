# Private Benchmark Definition Authoring Plan / 私有 Benchmark 定义创作计划

> **Execution rule / 执行规则:** Use test-driven development. Every production change must be preceded by a focused failing test, and every wave must retain the public-contract and evidence boundaries below.
>
> 使用测试驱动开发。每项 production change 前必须先有聚焦失败测试；每个波次都必须保持下述 public contract 与证据边界。

**Goal / 目标:** Add a local-only, guarded workflow that authors immutable benchmark datasets and one suite, seals them atomically, and binds that exact definition revision to an immutable Context commit for later deterministic execution and inspection.

新增一条仅本地、受保护的工作流：创作不可变 benchmark dataset 与一条 suite，原子封存完整定义，并将该精确定义 revision 绑定到不可变 Context commit，供后续确定性执行与检查。

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment directly advances completion criterion 3, which requires persisted, queryable, Web-visible benchmark suites and datasets, and criterion 1's Context-first platform coverage. It preserves the charter's immutable version history, reusable Rust core, stable adapter boundaries, guarded writes, deterministic ordering, and bilingual product workflow.

本增量直接推进完成条件 3（benchmark suite 与 dataset 必须可持久化、可查询并在 Web workspace 中可见）以及条件 1 的 Context-first 平台覆盖；同时保持宪章要求的不可变版本历史、可复用 Rust core、稳定 adapter 边界、受保护写入、确定性排序与双语产品工作流。

**Gap / 缺口：** ContextLab can persist definitions only as a side effect of sealing evaluation decision evidence. A user cannot author reusable definitions before execution, and storage has no immutable record proving which complete dataset/suite revision was selected at an exact `project/context/commit` scope. Treating project-scoped definition UUIDs as an implicit version would allow scope drift and would not prove an intentional Context binding.

ContextLab 目前只能在封存 evaluation decision evidence 时附带持久化定义。用户无法在执行前创作可复用定义，storage 也没有不可变记录证明某个精确 `project/context/commit` scope 选择了哪一份完整 dataset/suite revision。若把 project-scoped definition UUID 当作隐式版本，会允许 scope drift，也无法证明显式 Context binding。

**Why now / 为什么现在做：** The durable execution producer, exact decision reads, evaluation diff, scorecard/regression projection, protected Benchmark workspace BFF, local SDK, and Web inspector are already locally verified. Definition authoring is therefore the nearest dependency-ready missing link in criterion 3; provider execution, richer dashboards, and unrelated plugin/workflow work all depend on or follow this source-of-truth boundary.

持久 execution producer、精确 decision read、evaluation diff、scorecard/regression projection、受保护 Benchmark workspace BFF、local SDK 与 Web inspector 已获得本地验证。因此 definition authoring 是条件 3 中最近且依赖就绪的缺环；provider execution、更丰富 dashboard 及无关 plugin/workflow 工作都依赖或晚于这条事实源边界。

**Minimal boundary / 最小边界：**

1. A Rust authoring command validates one suite and its complete dataset membership, explicit schema version, stable UUIDs, deterministic ordering, exact Context source, expected branch head, principal, idempotency key, request digest, and capture time.
2. Memory and PostgreSQL adapters atomically persist immutable definitions plus one exact Context binding. PostgreSQL rechecks active write authorization and branch-head equality inside the same transaction; replay requires the same idempotency digest and immutable payload.
3. Only after the core/storage contract passes may an explicitly composed, default-off protected local route, non-public local SDK, same-origin BFF, and design-system editor transport the typed intent. The server remains the sole source of authorization, binding, sealing, and conflict decisions.
4. Bilingual architecture/API/user-flow documentation records the private boundary and evidence labels.

1. Rust authoring command 校验一条 suite 及其完整 dataset membership、显式 schema version、稳定 UUID、确定性排序、精确 Context source、expected branch head、principal、idempotency key、request digest 与 capture time。
2. Memory 与 PostgreSQL adapter 原子持久化不可变定义及一条精确 Context binding。PostgreSQL 在同一 transaction 内复核 active write authorization 与 branch-head equality；replay 必须具有相同 idempotency digest 与不可变 payload。
3. 仅在 core/storage contract 通过后，才能由显式组合、默认关闭的 protected local route、非公开 local SDK、同源 BFF 与 design-system editor 传输 typed intent。authorization、binding、sealing 与 conflict decision 仍完全由服务端负责。
4. 双语 architecture/API/user-flow 文档记录 private boundary 与证据标签。

**Non-goals / 非目标：** No public REST route, checked-in public OpenAPI operation, public TypeScript SDK write, provider/evaluator call, scheduler, queue, arbitrary measurement upload, benchmark policy calculation outside `contextlab-evaluation`, Context commit mutation, graph-diff calculation, release claim, production promotion, or external receipt work. `GraphDiff::between` remains the sole graph-diff calculator.

不新增 public REST route、已检入 public OpenAPI operation、public TypeScript SDK write、provider/evaluator call、scheduler、queue、任意 measurement upload、`contextlab-evaluation` 之外的 benchmark policy calculation、Context commit mutation、graph-diff calculation、release 声明、production promotion 或外部回执工作。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Focused evaluation/storage/API/local-SDK/Web tests; PostgreSQL migration and exact ignored runtime test when the local disposable server is available; `cargo fmt --all -- --check`; `cargo test --workspace --quiet`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo +1.85.0 check --workspace --all-targets --locked`; `pnpm check:web`; both local contract verifiers; and browser visual/a11y smoke only if a real local runtime is started. PostgreSQL/browser/Git evidence not actually observed remains `ignored` or `unobserved`; external release/production evidence remains `deferred`.

运行聚焦 evaluation/storage/API/local-SDK/Web 测试；本地 disposable server 可用时运行 PostgreSQL migration 与精确 ignored runtime test；再运行 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo +1.85.0 check --workspace --all-targets --locked`、`pnpm check:web`、两项 local contract verifier；只有启动真实本地 runtime 后才执行 browser visual/a11y smoke。未实际观测的 PostgreSQL/browser/Git 证据保持 `ignored` 或 `unobserved`；外部 release/production 证据保持 `deferred`。

## Shared Contract / 共享契约

- Definitions remain project-scoped, immutable, sealed domain values; the new revision binding is Context- and commit-scoped and never means mutable "latest".
- The command's suite membership must exactly equal the supplied dataset IDs. Cases, dataset IDs, thresholds, and read results use deterministic ordering and reject duplicates.
- The expected branch head must equal the bound immutable commit when the transaction runs. A moved or foreign branch fails closed before definition persistence.
- `(principal identity source, principal id, Context, branch, idempotency key)` identifies a retry. The same digest and payload replay; a changed digest or immutable payload conflicts without partial writes.
- Read projections may expose definition metadata and authoring disposition, but raw cases/oracles remain private to the authoring/editor and execution core.

- 定义保持 project-scoped、不可变且 sealed；新增 revision binding 具有 Context 与 commit scope，绝不表示可变的 "latest"。
- command 的 suite membership 必须与所提交 dataset ID 完全一致。case、dataset ID、threshold 与读取结果都采用确定性排序并拒绝重复。
- transaction 执行时，expected branch head 必须等于绑定的不可变 commit。branch 已移动或属于其他 Context 时，在 definition persistence 前 fail closed。
- `(principal identity source, principal id, Context, branch, idempotency key)` 标识 retry。相同 digest 与 payload 返回 replay；digest 或不可变 payload 改变时无部分写入地冲突。
- read projection 可暴露 definition metadata 与 authoring disposition；raw case/oracle 仍只属于 authoring/editor 与 execution core 的私有边界。

## Implementation Waves / 实施波次

### Wave 1: Rust domain and repository contract / Rust 领域与仓储契约

- [x] Add failing tests for validation, deterministic membership, explicit version, exact source, replay, digest conflict, stale/foreign head, forbidden principal, and no partial definition persistence.
- [x] Add the private authoring command, immutable binding/result/disposition, writer and exact-read ports in `contextlab-storage` while reusing `contextlab-evaluation` definitions and guarded idempotency types.
- [x] Implement parity in memory and PostgreSQL with migration `0020`, a single transaction, append-only constraints, exact composite foreign keys, transaction-local authorization recheck, branch-head lock/CAS check, deterministic reads, and PostgreSQL microsecond capture-time normalization.
- [x] Run focused Rust tests, format, strict storage Clippy, migration contract tests, workspace tests, and the exact ignored PostgreSQL test. The isolated UTF-8 loopback run then executed all 26 named storage tests with `26/26 passed`; ordinary workspace storage remains `166 passed, 38 ignored`.

### Wave 2: Protected local product flow / 受保护本地产品流程

- [x] Add red API tests proving default public absence, explicit protected composition, authentication before a dedicated write quota, write authorization/audit, fail-closed storage mapping, idempotency, exact scope, and `private, no-store` responses.
- [x] Add a strict non-public local SDK parser/client and same-origin BFF with request-memory Bearer credentials, no cookie authentication/forwarding, `credentials: "omit"`, and exact server-owned error semantics.
- [x] Add `data -> presenter -> screen -> editor` Web composition with shared design-system primitives, bilingual labels, loading/error/empty/available/conflict states, field validation, keyboard/focus semantics, responsive layout, and no page-local policy.
- [x] Update local contract verifiers so the route remains absent from public OpenAPI/public SDK and no second GraphDiff calculation appears.

### Wave 3: Integration and documentation / 集成与文档

- [x] Update `ARCHITECTURE.md`, local API contract, storage documentation, user workflow, active goal, completion criteria, and parallel-development evidence in English and Chinese.
- [x] Run the full fresh verification matrix, record exact passed/ignored/unobserved/deferred evidence, and select the next dependency-ready completion item without closing the long-term goal.

## 2026-07-27 Wave 2 closure boundary / 2026-07-27 Wave 2 收束边界

Wave 2 now has one canonical private local mutation route:
`POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`.
The non-public local SDK and same-origin Web BFF use the same project, Context, commit, branch, suite, and dataset scope. The browser keeps the Bearer token in request memory, omits cookies, and uses `credentials: "omit"`; BFF responses are `private, no-store`. The route is default-off and remains behind server-owned authentication, authorization/audit, the dedicated authoring quota, idempotency, and atomic storage checks.

Wave 2 现在只有一条 canonical private local mutation route：
`POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`。
非公开 local SDK 与同源 Web BFF 使用相同的 project、Context、commit、branch、suite 与 dataset scope。浏览器只在请求内存中保存 Bearer token，不携带 cookie，并使用 `credentials: "omit"`；BFF response 为 `private, no-store`。该 route 默认关闭，继续由服务端负责 authentication、authorization/audit、专用 authoring quota、idempotency 与原子 storage check。

The former context-only Web route `/api/local/contexts/{contextId}/commits/{commitId}/benchmark-definitions` is retired with a fail-closed `410` response and never contacts an upstream service. This prevents a second scope contract; the canonical project-scoped route remains the only authoring transport. The public router, checked-in OpenAPI, public TypeScript SDK, provider execution surface, and `GraphDiff::between` are unchanged.

旧的 context-only Web route `/api/local/contexts/{contextId}/commits/{commitId}/benchmark-definitions` 已退役，并以 fail-closed `410` response 结束，绝不触达 upstream service。这样可以避免第二套 scope contract；canonical project-scoped route 是唯一 authoring transport。public router、已检入 OpenAPI、public TypeScript SDK、provider execution surface 与 `GraphDiff::between` 均未改变。

Fresh verification observed `cargo fmt --all -- --check`, workspace Rust tests with storage `167 passed, 39 ignored`, strict workspace Clippy, the locked Rust `1.85.0` check, `pnpm check:web` with public SDK `14`, local SDK `68`, Web `151`, and the production build. The full Web receipt includes both the retired context-only route regression and canonical project-scoped route tests; the authoring data/presenter/screen focus is `11/11 passed`. The wave2 verifier self-test passed, and the live verifier reports `wave2_local_contracts=passed`, `graph_diff_calculators=passed count=1`, with `overall=unobserved` only because Git change-set evidence is unavailable. The workspace Playwright verifier remains unobserved because no local Web server was running; Docker/PostgreSQL runtime, authenticated browser-to-BFF-to-Axum runtime, Git binding, remote CI, operator rehearsal, release, and production evidence remain `ignored`, `unobserved`, or `deferred`.

新鲜验证已观测到 `cargo fmt --all -- --check`、workspace Rust test（storage `167 passed, 39 ignored`）、strict workspace Clippy、锁定 Rust `1.85.0` check，以及 `pnpm check:web`（public SDK `14`、local SDK `68`、Web `151`，并完成 production build）。Web 全量回执包含退役 context-only route regression 与 canonical project-scoped route test；authoring data/presenter/screen 聚焦为 `11/11 passed`。wave2 verifier 自测已通过，实际 verifier 报告 `wave2_local_contracts=passed`、`graph_diff_calculators=passed count=1`，仅因 Git change-set evidence 不可用而报告 `overall=unobserved`。workspace Playwright verifier 因未启动本地 Web server 而保持 unobserved；Docker/PostgreSQL runtime、authenticated browser-to-BFF-to-Axum runtime、Git binding、remote CI、operator rehearsal、release 与 production evidence 仍为 `ignored`、`unobserved` 或 `deferred`。

## Gate Repair Record / 门禁根因修复记录

**2026-07-27 PostgreSQL precision regression / 2026-07-27 PostgreSQL 精度回归：** The first fresh loopback PostgreSQL 16.14 authoring runtime reached the writer and failed only when comparing the created binding with the rehydrated binding: PostgreSQL persisted captured_at at microsecond precision while the in-memory command retained nanoseconds. The minimum repair is adapter-local normalization to (nanoseconds / 1000) * 1000 before insert, matching the existing benchmark evidence/projection precision rule. The authoring runtime test is the regression proof; the database was UTF-8, loopback-only, and stopped after the failure. This is a local storage compatibility repair, not production or release evidence.

**2026-07-27 PostgreSQL 精度回归：** 首次新鲜 loopback PostgreSQL 16.14 authoring runtime 已到达 writer，唯一失败发生在 created binding 与 rehydrated binding 比较时：PostgreSQL 以微秒精度持久化 captured_at，而内存 command 仍保留纳秒。最小修复是在 insert 前由 adapter 将时间规范化为 (nanoseconds / 1000) * 1000，与既有 benchmark evidence/projection precision rule 一致。authoring runtime test 作为回归证明；数据库使用 UTF-8、仅 loopback，失败后已停止。该记录是本地 storage compatibility 修复，不是 production 或 release evidence。
