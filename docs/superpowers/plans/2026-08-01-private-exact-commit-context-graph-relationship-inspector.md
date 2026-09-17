# Private Exact-Commit Context Graph Relationship Inspector / 私有精确提交 Context Graph 关系检查器

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与宪章原则

- **Criterion 1 / 条件 1:** Context Graph relationships need a reusable persistence/API/SDK contract and at least one exact-Context-commit UI inspection path.
  / **条件 1：** Context Graph relationship 需要可复用的持久化/API/SDK contract，以及至少一条精确 Context commit 的 UI inspection path。
- **Criterion 2 / 条件 2:** A versioned Context commit must be replayable and inspectable without losing its immutable graph identity.
  / **条件 2：** 版本化 Context commit 必须可回放、可检查，并保持不可变 graph identity。
- **Criterion 4 / 条件 4:** Web, API, SDK, and storage must consume the same Context Graph contract; `GraphDiff::between` remains the sole graph-diff calculator.
  / **条件 4：** Web、API、SDK 与 storage 必须消费同一套 Context Graph contract；`GraphDiff::between` 继续是唯一 graph-diff calculator。

### Gap, dependency, and evidence / 缺口、依赖与证据

The existing private lifecycle read already returns an exact `(Context, commit)` graph snapshot, and storage has tested immutable Memory/PostgreSQL snapshot contracts. A cross-layer audit found that the Rust snapshot serializes top-level `project_id`, while the local SDK lifecycle parser rejects that field as unknown. The Web exact-commit helper also derives only `Uses` relationships, so the existing graph contract is not fully inspectable at the selected commit. This is a concrete contract defect and a dependency-ready consumer gap; no new persistence or transport primitive is required.

现有 private lifecycle read 已返回精确 `(Context, commit)` graph snapshot，storage 也已有经过测试的不可变 Memory/PostgreSQL snapshot contract。跨层审阅发现 Rust snapshot 序列化顶层 `project_id`，而 local SDK lifecycle parser 将其错误判定为 unknown field。Web 的 exact-commit helper 还只派生 `Uses` 关系，因此选定 commit 上的既有 graph contract 尚不能被完整检查。这是明确的 contract defect 与依赖就绪 consumer gap，不需要新增持久化或 transport primitive。

### Why now / 为何现在优先

The preceding pair-read increment established a consistent version-backed read boundary. Repairing the SDK contract before adding a graph consumer prevents real lifecycle responses from failing closed for the wrong reason, and the relationship inspector is the smallest visible consumer that directly demonstrates exact commit graph state. It is higher priority than new editing, merge writers, benchmark execution, or public transport expansion.

前一项 pair-read 增量已经建立一致的版本化读取边界。先修复 SDK contract，可避免真实 lifecycle response 因错误 unknown field 而 fail closed；relationship inspector 是最小可见 consumer，直接证明 exact commit graph state。它优先于新的编辑、merge writer、benchmark execution 或 public transport 扩张。

### Explicit non-goals / 明确非目标

- No new REST route, OpenAPI operation, public SDK method, write, mutation, merge/rollback, migration, provider call, or operator transport.
  / 不新增 REST route、OpenAPI operation、public SDK method、write、mutation、merge/rollback、migration、provider call 或 operator transport。
- No raw component content, metadata, credentials, diagnostics, or private payload in the relationship projection; graph inspection is redacted to stable node/edge facts.
  / relationship projection 不暴露 component raw content、metadata、凭据、diagnostic 或 private payload；graph inspection 只保留稳定 node/edge facts。
- No page-local graph or diff algorithm; the Web consumes the existing lifecycle graph snapshot and `GraphDiff::between` remains the only calculator.
  / 不在页面内实现 graph 或 diff algorithm；Web 消费既有 lifecycle graph snapshot，`GraphDiff::between` 仍是唯一 calculator。
- No Docker/PostgreSQL runtime, authenticated browser, visual, Git, remote CI, release, or production claim.
  / 不声称 Docker/PostgreSQL runtime、authenticated browser、visual、Git、remote CI、release 或 production 证据。

### Smallest boundary, ownership, and bilingual docs / 最小边界、所有权与双语文档

- Repair the local SDK lifecycle snapshot parser and its focused regression tests in `packages/local-sdk/src/types.ts` and `packages/local-sdk/src/client.test.ts`.
  / 在 `packages/local-sdk/src/types.ts` 与 `packages/local-sdk/src/client.test.ts` 修复 local SDK lifecycle snapshot parser，并补 focused regression test。
- Extend the existing exact-commit Web presenter/data contract to preserve all graph edge kinds and render incoming/outgoing relationships through shared design-system primitives. Keep the existing lifecycle API/BFF route unchanged.
  / 扩展既有 exact-commit Web presenter/data contract，保留全部 graph edge kind，并通过 shared design-system primitive 呈现入边/出边；保持 lifecycle API/BFF route 不变。
- Integration Lead owns this plan, API/public-boundary checks, bilingual roadmap/completion receipts, and final cross-stack verification.
  / Integration Lead 负责本计划、API/public-boundary checks、双语 roadmap/completion 回执与最终跨栈验证。

### Fresh verification required / 下一增量前的新鲜验证

First observe a red local SDK parser test using the actual Rust-shaped `project_id` snapshot fixture. Then observe green SDK scope/unknown-field tests and Web presenter/inspector tests covering all edge kinds, deterministic ordering, empty state, exact commit identity, raw-content exclusion, and bilingual accessibility states. Finally run `cargo fmt --all -- --check`, offline workspace Rust tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, the local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL runtime, browser, Git, and external release evidence remain unobserved/deferred.

先使用真实 Rust-shaped `project_id` snapshot fixture 观察 local SDK parser red，再观察 SDK scope/unknown-field tests 与 Web presenter/inspector tests 变绿，覆盖全部 edge kind、确定性排序、empty state、exact commit identity、raw-content exclusion 与双语 accessibility state。最后运行 `cargo fmt --all -- --check`、offline workspace Rust test、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL runtime、browser、Git 与 external release evidence 继续为 unobserved/deferred。

## Execution Checklist / 执行清单

- [x] Observe the parser red phase and add the minimal SDK contract repair.
  / 观察 parser red 阶段并完成最小 SDK contract 修复。
- [x] Preserve all exact-commit graph edge kinds in the Web data -> presenter -> screen path with focused regressions.
  / 在 Web data -> presenter -> screen 路径保留 exact-commit 的全部 graph edge kind，并补 focused regression。
- [x] Run fresh local verification, update the bilingual roadmap/criteria receipts, and keep the long-term goal active.
  / 运行新鲜 local verification，更新双语 roadmap/criteria 回执，并保持长期目标 active。

## Observed Result / 已观测结果

The parser red phase was observed with a Rust-shaped lifecycle snapshot containing `project_id`; the
private SDK now preserves and validates that UUID. The Web relationship projection is owned by the
presenter and renders every supported edge kind through the existing design-system table and
definition primitives. It keeps exact commit identity, deterministic source/target/kind ordering,
empty state, bilingual accessible labels, and stable node/edge facts only.

已使用包含 `project_id` 的 Rust-shaped lifecycle snapshot 观测到 parser red phase；私有 SDK 现会保留并校验该 UUID。Web relationship projection
由 presenter 负责，并通过既有 design-system table 与 definition primitive 呈现全部支持的 edge kind。它保持 exact commit identity、源/目标/类型的确定性排序、
empty state、双语可访问标签，并且只保留稳定 node/edge fact。

Fresh local evidence / 新鲜本地证据：

- Focused Web presenter/editor: `29 passed`; local SDK: `135 passed` and TypeScript check passed.
- `cargo fmt --all -- --check`: passed; `cargo test --workspace --quiet`: passed, including storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `135`, Web `276`, TypeScript/lint and production build.
- `scripts/verify-local-contracts.ps1`: scoped checks passed; `overall=unobserved` because no unified diff input was supplied.
- `GRAPH_DIFF_IMPL_COUNT=1`: passed.

新鲜本地证据：

- Web presenter/editor 聚焦测试：`29 passed`；local SDK：`135 passed`，TypeScript check 通过。
- `cargo fmt --all -- --check` 通过；`cargo test --workspace --quiet` 通过，其中 storage `212 passed, 39 ignored`。
- strict offline Clippy 通过；锁定 Rust `1.85.0` workspace check 通过。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `276`，TypeScript/lint 与 production build 均通过。
- `scripts/verify-local-contracts.ps1` 的 scoped checks 通过；因未提供 unified diff input，`overall=unobserved`。
- `GRAPH_DIFF_IMPL_COUNT=1` 通过。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
operator transport, or second graph-diff calculator was added. Docker/PostgreSQL runtime,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain `unobserved` or `deferred`. This plan is locally verified and does not close the long-term
goal; the next implementation needs a new bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、operator transport 或第二个 graph-diff calculator。
Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。
本计划已在本地验证，但不关闭长期目标；下一项实现必须先有新的双语 Necessity Record。
