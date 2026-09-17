# Private Benchmark Definition Binding Inspection / 私有 Benchmark 定义绑定检查

> **Execution rule / 执行规则:** Use test-driven development. Add focused failing contract tests
> before production changes, keep all reads redacted, and preserve the distinction between local
> evidence and external release evidence.
>
> 使用测试驱动开发。所有 production change 前先增加聚焦失败 contract test；所有读取保持脱敏，
> 并保持本地证据与外部 release evidence 的边界。

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
advances Criterion 3, which requires persisted, queryable, and Web-visible benchmark suites and
datasets, and reinforces the Context-first/versioned-history principles. The authoring writer already
persists an immutable definition binding at an exact project/Context/commit scope, but the product
cannot yet discover or select that binding after the write response is gone.

本增量直接推进条件 3（benchmark suite 与 dataset 必须可持久化、可查询并在 Web workspace 中可见），
并强化 Context-first 与 versioned-history 原则。authoring writer 已经在精确 project/Context/commit scope
持久化不可变 definition binding，但 write response 消失后，产品还不能发现或选择该 binding。

**Gap, dependency, and risk / 缺口、依赖与风险:** `contextlab-storage` already owns exact
binding read/list ports and deterministic ordering. The missing layer is the protected local read
transport and presentation adapter. Reusing those ports avoids a second source of truth, but exposing
raw dataset cases, expected outputs, internal digests, or mutable "latest" semantics would violate the
existing redaction and immutable-scope contract.

`contextlab-storage` 已拥有 exact binding read/list port 与确定性排序。当前缺的是 protected local read
transport 与 presentation adapter。复用这些 port 可以避免第二个事实源；但暴露 raw dataset case、expected output、
internal digest 或可变的 "latest" 语义会违反现有脱敏与不可变 scope contract。

**Why now / 为什么现在做:** Wave 2 authoring transport is freshly verified, including atomic
write, replay, exact scope, public-surface exclusion, and the Web editor. Inspection is the nearest
dependency-ready Criterion 3 gap: it makes the newly authored resource queryable and lets later
execution orchestration select an exact immutable revision without inventing a new selection path.

Wave 2 authoring transport 已获得新鲜验证，包括原子写入、replay、精确 scope、public-surface exclusion 与 Web editor。
binding inspection 是条件 3 最近且依赖就绪的缺口：它让新创作的 resource 可查询，并使后续 execution orchestration
能够选择精确不可变 revision，而无需另造 selection path。

**Minimal affected boundary / 最小受影响边界:**

1. Add a redacted Rust read projection/application service over the existing exact binding repository
   ports. It returns stable binding metadata, definition schema version, suite ID/name, ordered dataset
   IDs/names, exact Context commit, branch, disposition, and capture time; raw cases and request digests
   remain private.
2. Add a protected, default-off local GET/list route and a non-public local SDK parser/client. The
   server owns authentication, `ContextPermission::Read`, audit, the dedicated read quota, exact
   project/Context/commit scope, and fail-closed storage mapping. Public OpenAPI/public SDK remain
   unchanged.
3. Add same-origin BFF and `data -> presenter -> screen` Web inspection/selection using shared
   design-system primitives, bilingual loading/error/empty/available/unavailable states, and no policy
   or diff calculation in the page.
4. Add bilingual API/architecture/user-flow evidence and focused contract verifiers.

1. 在现有 exact binding repository port 之上增加脱敏 Rust read projection/application service。返回稳定 binding
   metadata、definition schema version、suite ID/name、有序 dataset ID/name、精确 Context commit、branch、
   disposition 与 capture time；raw case 与 request digest 继续保持私有。
2. 增加受保护、默认关闭的 local GET/list route 与非公开 local SDK parser/client。authentication、
   `ContextPermission::Read`、audit、专用 read quota、精确 project/Context/commit scope 与 fail-closed storage
   mapping 由服务端负责。public OpenAPI/public SDK 保持不变。
3. 增加同源 BFF 与 `data -> presenter -> screen` Web inspection/selection，使用共享 design-system primitive、
   双语 loading/error/empty/available/unavailable state，页面不实现 policy 或 diff calculation。
4. 增加双语 API/architecture/user-flow evidence 与 focused contract verifier。

**Non-goals / 非目标:** No public REST/OpenAPI/public SDK write or read promotion, no definition
mutation, no Context commit mutation, no provider/evaluator execution, no benchmark scorecard or
regression recalculation, no raw case/output transport, no operator transport, no external receipt work,
and no new graph-diff calculator. `GraphDiff::between` remains the sole graph-diff calculator.

不新增 public REST/OpenAPI/public SDK write 或 read promotion，不修改 definition，不修改 Context commit，不执行
provider/evaluator，不重算 benchmark scorecard 或 regression，不传输 raw case/output，不增加 operator transport、
不进行 external receipt work，也不增加新的 graph-diff calculator。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** Focused Rust/API/local-SDK/Web
tests; route/public-surface contract verifiers; `cargo fmt --all -- --check`; `cargo test --workspace --quiet`;
strict workspace Clippy; locked Rust `1.85.0` check; `pnpm check:web`; and the live local contract verifier.
Only a real local server may produce browser evidence. Docker/PostgreSQL, Git, remote CI, operator rehearsal,
release, and production remain `ignored`, `unobserved`, or `deferred` unless directly observed.

聚焦 Rust/API/local-SDK/Web test；route/public-surface contract verifier；`cargo fmt --all -- --check`；
`cargo test --workspace --quiet`；strict workspace Clippy；锁定 Rust `1.85.0` check；`pnpm check:web`；
以及 live local contract verifier。只有真实本地 server 才能产生 browser evidence。Docker/PostgreSQL、Git、remote CI、
operator rehearsal、release 与 production 在未实际观测时保持 `ignored`、`unobserved` 或 `deferred`。

## Implementation Waves / 实施波次

### Wave 1: Rust projection and read application / Rust projection 与读取 application

- [x] Add failing tests for exact scope, deterministic ordering, raw-field rejection, missing binding,
  unknown Context/commit, and no "latest" substitution.
- [x] Add the redacted read projection and application port by reusing
  `BenchmarkDefinitionBindingRepository`; do not duplicate storage lookup or policy logic.
- [x] Add memory parity tests and typed error mappings.

### Wave 2: Protected local transport / 受保护本地传输

- [x] Add protected GET/list route tests for authentication-before-quota, RBAC/audit, exact scope,
  `private, no-store`, rate-limit mapping, and public OpenAPI/public SDK exclusion.
- [x] Add the non-public local SDK parser/client and same-origin BFF with request-memory Bearer,
  `credentials: "omit"`, cookie omission, strict schema validation, and redacted failures.

### Wave 3: Web selection and evidence / Web 选择与证据

- [x] Add `data -> presenter -> screen` inspection/selection with shared primitives and all bilingual
  states. Selection must return the exact binding ID and commit, never mutable latest state.
- [x] Update the bilingual API/architecture/user-flow evidence and run the full verification matrix.
- [x] Select the next dependency-ready local increment; do not close the long-term goal.

## Fresh Verification Receipt / 新鲜验证回执

Observed after the implementation was settled: `cargo fmt --all -- --check` passed; focused storage
tests passed with `167 passed, 39 ignored`; the Wave 2 contract verifier reported
`wave2_local_contracts=passed` and `graph_diff_calculators=passed count=1`; and `pnpm check:web`
passed with public SDK `14`, local SDK `70`, Web `156`, and the production build. The live verifier
reported `overall=unobserved` only because no safe Git change-set evidence was available. No Web
server was running for the local Playwright verifier, so authenticated browser evidence remains
unobserved. Docker/PostgreSQL runtime, remote CI, operator rehearsal, release, and production remain
ignored, unobserved, or deferred. Six requested `gpt-5.6-luna` review workers could not be scheduled
because the agent thread limit was full; this is a scheduling receipt, not a product result.

实现稳定后观察到：`cargo fmt --all -- --check` 通过；storage 聚焦测试通过，结果为
`167 passed, 39 ignored`；Wave 2 contract verifier 报告 `wave2_local_contracts=passed` 与
`graph_diff_calculators=passed count=1`；`pnpm check:web` 通过，public SDK `14`、local SDK `70`、
Web `156`，并完成 production build。live verifier 的 `overall=unobserved` 仅因没有可用的安全
Git change-set evidence。local Playwright verifier 未运行，因为没有启动 Web server，因此
authenticated browser evidence 仍为未观测。Docker/PostgreSQL runtime、remote CI、operator rehearsal、
release 与 production 仍为 ignored、unobserved 或 deferred。请求的 6 个 `gpt-5.6-luna` review
worker 因 agent thread limit 已满而未能调度；这是调度回执，不是产品结果。

## Next Admitted Increment / 下一项准入增量

The next local increment is recorded in
`docs/superpowers/plans/2026-07-27-private-benchmark-binding-execution-selection.md`.
It turns one selected immutable definition binding into a provider-free, replayable
execution-selection contract for the existing Benchmark execution service. It does not invoke an
evaluator, persist evidence, add transport, or close Criterion 3 or the long-term goal.

下一项本地增量已记录在
`docs/superpowers/plans/2026-07-27-private-benchmark-binding-execution-selection.md`。
它把一条已选定的不可变 definition binding 转换为现有 Benchmark execution service 可消费的
provider-free、可回放 execution-selection contract。不调用 evaluator、不持久化 evidence、不增加
transport，也不关闭条件 3 或长期目标。
