# Private Persisted Context Diff Review Read
# 私有持久化 Context Diff Review 读取

## Necessity Record / 必要性记录

### Criterion and charter principle / 对应条件与宪章原则

- Criterion 2, Versioning and diff workflows: exact commit-bound semantic,
  behavior, and evaluation diff inputs must be consumable as one complete,
  replayable review projection.
- Criterion 4, Graph as the system backbone: API, SDK, and Web must consume the
  same persisted versioned Context contract rather than reconstructing or
  recalculating diff inputs in a page.
- Charter principle: `GraphDiff::between` and the reusable Rust diff services
  remain the only calculation boundary.

- 条件 2，版本与 Diff 工作流：按 exact commit 绑定的 semantic、behavior 与
  evaluation diff input 必须能作为一个完整、可回放的 review projection 被消费。
- 条件 4，Graph 作为系统骨架：API、SDK 与 Web 必须消费同一份持久化版本化
  Context contract，不得在页面中重建或重新计算 diff input。
- 宪章原则：`GraphDiff::between` 与可复用 Rust diff service 仍是唯一计算边界。

### Gap and dependency / 缺口与依赖

`ContextDiffSnapshotV1` persistence and `PersistedContextDiffReviewService` already
validate exact project/Context/commit scopes and delegate to the unified Rust diff
engine. A bounded audit found no private API, local SDK, or Web consumer for this
complete projection. The dependencies are ready: storage owns the repository and
review service, the protected API already has exact-scope read authentication and
private/no-store middleware, the local SDK has strict V1 parsers/client patterns,
and Web has shared data/presenter/screen primitives.

`ContextDiffSnapshotV1` 持久化与 `PersistedContextDiffReviewService` 已校验 exact
project/Context/commit scope，并委托统一 Rust diff engine。一次有界审计确认该完整
projection 尚无 private API、local SDK 或 Web consumer。依赖已就绪：storage 拥有
repository 与 review service，protected API 已有 exact-scope read authentication
及 private/no-store middleware，local SDK 已有 strict V1 parser/client pattern，Web
已有共享 data/presenter/screen primitive。

### Why now / 当前优先级

This is the smallest cross-layer increment that directly closes the named local
accessibility gap in Criteria 2 and 4. It consumes sealed, immutable records that
already exist, so it does not add another persistence model or diff algorithm. It
also turns the existing Rust evidence into a usable private workflow before any
public transport or mutation decision.

这是直接收束条件 2 与 4 已命名本地可访问性缺口的最小跨层增量。它消费已经存在
的 sealed immutable record，不新增另一套 persistence model 或 diff algorithm；并在
任何 public transport 或 mutation 决策前，把现有 Rust 证据变成可使用的 private workflow。

### Shared contract / 共享契约

- Private GET only: `/api/v1/local/projects/{project_id}/contexts/{context_id}/diff-review`.
- Required query: `source_commit_id`, `target_commit_id`; both commits must be
  distinct and share the requested project/Context scope.
- Response: serialized `VersionedContextDiffReviewProjectionV1`, schema V1,
  including exact source/target scope and the complete semantic/behavior/evaluation
  result. Raw benchmark or provider payloads are not introduced.
- Local SDK and Web must parse the response fail-closed, preserve exact scope and
  deterministic ordering, and send request-memory Bearer credentials with no cookies.
- Missing storage projection is an honest unavailable/empty state; upstream errors
  are structured and redacted.

- 仅 private GET：`/api/v1/local/projects/{project_id}/contexts/{context_id}/diff-review`。
- 必填 query：`source_commit_id`、`target_commit_id`；两个 commit 必须不同，并共享
  请求中的 project/Context scope。
- Response：序列化 `VersionedContextDiffReviewProjectionV1`、schema V1，包含 exact
  source/target scope 与完整 semantic/behavior/evaluation result；不引入 raw benchmark
  或 provider payload。
- local SDK 与 Web 必须 fail-closed 解析 response，保持 exact scope 与确定性排序，并仅
  发送 request-memory Bearer、禁止 cookie。
- storage projection 缺失时诚实表示 unavailable/empty；upstream error 必须结构化且脱敏。

### Non-goals / 明确非目标

- No public REST/OpenAPI/public SDK operation, write route, Web mutation, or
  snapshot authoring route.
- No new persistence writer, migration, provider call, secret access, operator
  transport, release/production evidence, or second diff calculator.
- No client-side semantic, behavior, evaluation, or graph diff calculation.

- 不新增 public REST/OpenAPI/public SDK operation、写入 route 或 Web mutation。
- 不新增 persistence writer、migration、provider call、secret access、operator transport、
  release/production evidence 或第二个 diff calculator。
- 不在 client 端计算 semantic、behavior、evaluation 或 graph diff。

### Minimal boundary and bilingual documentation / 最小边界与双语文档

The shared API DTO and protected route belong to `server/api`; the SDK adapter and
tests belong to `packages/local-sdk`; the BFF/data/presenter/screen and tests belong
to the existing Web app. Ownership must remain disjoint. This bilingual plan,
the shared Rust contract docs, and the final roadmap receipt are the documentation
boundary.

共享 API DTO 与 protected route 归 `server/api`；SDK adapter 与 tests 归
`packages/local-sdk`；BFF/data/presenter/screen 与 tests 归现有 Web app。ownership
必须互不重叠。本双语 plan、共享 Rust contract 文档与最终 roadmap receipt 构成文档边界。

### Fresh verification required / 必须取得的新鲜验证

Before selecting the next increment, observe focused storage/API/SDK/Web contract
tests, full Rust/Web gates, format, strict offline Clippy, locked Rust 1.85,
public-surface and sole-GraphDiff checks. Docker/PostgreSQL runtime, browser, Git,
remote CI, operator, release, and production remain explicitly unobserved/deferred.

选择下一项增量前，必须新鲜观察 focused storage/API/SDK/Web contract tests、完整 Rust/Web
门禁、format、strict offline Clippy、锁定 Rust 1.85、public-surface 与唯一 GraphDiff 检查。
Docker/PostgreSQL runtime、browser、Git、remote CI、operator、release 与 production 继续明确
标记为 `unobserved/deferred`。

## Implementation checklist / 实施清单

- [x] Add the private API route and AppState repository wiring.
- [x] Add the strict local SDK V1 adapter and tests.
- [x] Add the same-origin BFF plus shared Web data/presenter/screen states and tests.
- [x] Run fresh cross-stack verification and update the bilingual roadmap receipt.

- [x] 增加 private API route 与 AppState repository wiring。
- [x] 增加 strict local SDK V1 adapter 与 tests。
- [x] 增加同源 BFF 与 shared Web data/presenter/screen states 及 tests。
- [x] 运行新鲜跨栈验证并更新双语 roadmap receipt。

## Completion receipt / 收束回执

`completed / verified locally` / `completed / 本地已验证`. The private route is wired to the
existing persisted Context diff-review repository and preserves exact project, Context, source
commit, target commit, and schema scope. The local SDK uses the strict V1 parser and request-memory
Bearer contract. The Web BFF and `data -> presenter -> screen` composition expose only the
redacted review projection and keep loading, error, empty, unavailable, and available states in
the shared design-system boundary.

`completed / verified locally` / `completed / 本地已验证`。private route 已接入既有持久化 Context diff-review repository，保持
project、Context、source commit、target commit 与 schema 的 exact scope。local SDK 使用 strict V1 parser 与 request-memory Bearer
契约。Web BFF 与 `data -> presenter -> screen` composition 只暴露脱敏 review projection，并将 loading、error、empty、unavailable、
available 状态留在共享 design-system 边界内。

Fresh evidence / 新鲜证据：focused API diff-review `6 passed`; `cargo fmt --all -- --check`;
`cargo test --workspace --quiet --no-fail-fast` with storage `205 passed, 39 ignored`;
`cargo clippy --workspace --all-targets --offline -- -D warnings`; locked Rust `1.85.0`
workspace check; `pnpm check:web` with public SDK `15`, local SDK `117`, Web `245`, TypeScript,
lint, and production build; `GRAPH_DIFF_IMPL_COUNT=1`; public OpenAPI diff-review hits `0`; private
handler hits `1`.

新鲜证据：focused API diff-review `6 passed`；`cargo fmt --all -- --check`；workspace Rust 通过（storage `205 passed, 39 ignored`）；
strict offline Clippy；锁定 Rust `1.85.0` workspace check；`pnpm check:web` 通过（public SDK `15`、local SDK `117`、Web `245`、
TypeScript、lint 与 production build）；`GRAPH_DIFF_IMPL_COUNT=1`；public OpenAPI diff-review 命中 `0`；private handler 命中 `1`。

Boundary / 边界：no public REST/OpenAPI/public SDK method, write route, Web mutation, provider,
migration, secret access, operator transport, or second `GraphDiff` calculator was added. Docker/
PostgreSQL runtime, authenticated browser, visual smoke, Git change-set, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`.

边界：未新增 public REST/OpenAPI/public SDK method、写入 route、Web mutation、provider、migration、secret access、operator transport
或第二个 `GraphDiff` calculator。Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator
rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。本切片推进 Criteria 2 与 4，但不关闭任一 criterion 或长期目标。
