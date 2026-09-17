# Private Versioned Context Diff Contract / 私有版本化 Context Diff 契约

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criterion 2 (replayable version history), Criterion 4 (Graph as the system backbone), and the charter's
Context-first, reusable-Rust-core, and stable-contract principles. Semantic, behavior, and evaluation
diffs must identify the exact immutable Context versions that produced them.

本增量直接推进条件 2（可回放版本历史）、条件 4（图谱作为系统骨架），以及章程中的 Context-first、可复用
Rust core 与稳定契约原则。Semantic、behavior 与 evaluation diff 必须标识产生它们的精确不可变 Context 版本。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The repository already
has validated `SemanticSnapshotV1`, `BehaviorSnapshotV1`, `EvaluationSnapshotV1`, and
`ContextDiffService`, but the private version-bound review request only carries a bare pair of
`CommitId` values and permits callers to supply snapshots without a typed project/Context scope.
That leaves room for cross-Context or cross-project review wiring to appear valid at the Rust
boundary, even though the graph snapshot contract now requires exact `(ProjectId, ContextId,
CommitId)` identity.

仓库已经具备经过校验的 `SemanticSnapshotV1`、`BehaviorSnapshotV1`、`EvaluationSnapshotV1` 与
`ContextDiffService`，但当前私有 version-bound review request 只有裸的 `CommitId` pair，调用方也可以在没有
typed project/Context scope 的情况下提供 snapshot。这会让跨 Context 或跨 project 的 review wiring 在 Rust
边界看似有效，尽管 graph snapshot contract 已要求精确的 `(ProjectId, ContextId, CommitId)` 身份。

**Why now / 为什么现在优先:** The commit-associated graph snapshot contract is green and the
existing graph route already resolves exact commit scopes. The next dependency-ready local gap is
to give the adjacent semantic/behavior/evaluation review contract the same identity and fail-closed
discipline before any future adapter or UI consumes it.

commit-associated graph snapshot contract 已经通过验证，现有 graph route 也已解析 exact commit scope。当前最接近且
依赖就绪的本地缺口，是让相邻的 semantic/behavior/evaluation review contract 具备同样的身份绑定与 fail-closed
纪律，再允许未来 adapter 或 UI 消费它。

**Minimal boundary / 最小边界:** Add one private, typed V1 version scope in
`crates/diff-engine`, bind it to the existing complete diff snapshot and version-bound projection,
reject mixed project/Context scopes and identical versions, preserve deterministic ordering, and
add focused domain/serialization tests. Keep the existing pure diff algorithm and result shape.

在 `crates/diff-engine` 增加一份私有 typed V1 version scope，将其绑定到现有 complete diff snapshot 与
version-bound projection；拒绝混用 project/Context scope 与 identical version；保持确定性排序，并增加聚焦 domain/
serialization test。保留现有纯 diff 算法和 result shape。

**Explicit non-goals / 明确非目标:** No public REST route, OpenAPI/SDK method, Web mutation or new
screen; no storage migration or provider call; no graph editing, branch/merge mutation, or raw
private content; no second graph-diff calculator. `GraphDiff::between` remains the sole graph diff
calculator. Docker, PostgreSQL runtime, release, production, and external receipts remain deferred.

不新增 public REST route、OpenAPI/SDK method、Web mutation 或新 screen；不做 storage migration、provider call、
graph editing、branch/merge mutation 或 raw private content；不增加第二个 graph-diff calculator。
`GraphDiff::between` 仍是唯一 graph diff calculator。Docker、PostgreSQL runtime、release、production 与外部回执继续延期。

**Ownership and bilingual documentation / 所有权与双语文档:** Rust contract and focused tests
belong exclusively to `crates/diff-engine/src/review.rs`, `crates/diff-engine/src/lib.rs` for the
admitted EOF-newline root-cause fix, their public crate re-exports, and
`crates/diff-engine/tests/versioned_diff_review_projection.rs`. This plan and the roadmap ledgers
are the only documentation boundary for the increment.

Rust contract 与聚焦测试独占 `crates/diff-engine/src/review.rs`、已准入 EOF-newline 根因修复所需的
`crates/diff-engine/src/lib.rs`、crate re-export，以及
`crates/diff-engine/tests/versioned_diff_review_projection.rs`。本计划与 roadmap ledger 是本增量唯一文档边界。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Focused
diff tests must prove exact typed scope preservation, mixed-scope rejection, immutable replay of
the request payload, deterministic semantic/behavior/evaluation ordering, and one assertion that
the comparison path still reaches `GraphDiff::between`. Then run `cargo fmt --all -- --check`,
workspace Rust tests, strict offline Clippy, and `pnpm check:web`; unobserved PostgreSQL, browser,
Git, remote, operator, release, and production evidence must remain explicitly labeled.

聚焦 diff test 必须证明 typed scope 精确保留、mixed-scope rejection、request payload 不可变 replay、semantic/
behavior/evaluation 的确定性排序，以及 comparison path 仍到达 `GraphDiff::between` 的 assertion。随后运行
`cargo fmt --all -- --check`、workspace Rust tests、strict offline Clippy 与 `pnpm check:web`；未观测到的 PostgreSQL、
browser、Git、remote、operator、release 与 production evidence 必须继续明确标注。

**Root-cause correction discovered during admission / 准入期间发现的根因修复：** The focused
Diff review found that `TextDiff` used `str::lines()`, which erases an end-of-file terminator and
could render a known semantic-document modification as an empty diff. This directly blocks the
criterion's deterministic review evidence. The minimal fix preserves a terminal newline as one
explicit empty diff line and adds a regression test; it changes neither graph diffing nor transport.

聚焦 Diff 审查发现 `TextDiff` 使用 `str::lines()`，会抹去文件末尾 terminator，可能把已知的 semantic-document
修改呈现为空 diff。这会直接阻断条件所需的确定性 review evidence。最小修复将 terminal newline 保留为一个显式的空
diff line，并增加回归测试；它不改变 graph diff 或 transport。

## Implementation Steps / 实施步骤

- [x] Add the typed V1 exact version scope and validated request constructor.
- [x] Reject cross-project/cross-Context pairs and preserve the existing review projection shape.
- [x] Add red/green tests for scope, replay, ordering, serialization, and GraphDiff delegation.
- [x] Run focused and workspace verification; record actual receipts in this plan and roadmap ledgers.

- [x] 增加 typed V1 exact version scope 与 validated request constructor。
- [x] 拒绝跨 project/跨 Context pair，并保持现有 review projection shape。
- [x] 增加 scope、replay、ordering、serialization 与 GraphDiff delegation 的红绿测试。
- [x] 运行聚焦与 workspace 验证，并在本计划与 roadmap ledger 记录真实回执。

## Implementation and Verification Receipt / 实现与验证回执

`VersionedContextScopeV1` now binds each private review side to exact `ProjectId`, `ContextId`, and
`CommitId`. `VersionedContextDiffReviewRequestV1` and its projection preserve these scopes, reject a
mixed project/Context pair and an identical exact scope before any diff is calculated, and retain the
existing semantic/behavior/evaluation result shape. The non-empty graph regression proves the
versioned wrapper still reaches the unified `ContextDiffService` path and its sole
`GraphDiff::between` call. `TextDiff` also preserves an EOF newline as an explicit empty changed
line, so a semantic document modification cannot be reported with an empty text diff.

`VersionedContextScopeV1` 现将每个 private review side 绑定到精确的 `ProjectId`、`ContextId` 与
`CommitId`。`VersionedContextDiffReviewRequestV1` 与其 projection 会保留这些 scope，在计算任何 diff 前拒绝混用
project/Context pair 和 identical exact scope，并保持既有 semantic/behavior/evaluation result shape。非空 graph
回归证明 versioned wrapper 仍走统一的 `ContextDiffService` 路径及其唯一的 `GraphDiff::between` 调用。`TextDiff` 还会
将 EOF newline 保留为显式的空 changed line，因此 semantic document 修改不会再被呈现为 empty text diff。

Fresh local evidence passed: `cargo fmt --all -- --check`; `cargo test -p contextlab-diff-engine
--all-targets` with `20 passed`; `cargo test --workspace --quiet` with API `184 passed` and storage
`179 passed, 39 ignored`; `cargo clippy --workspace --all-targets --offline -- -D warnings`; and
`cargo +1.85.0 check --workspace --all-targets --locked`. `pnpm check:web` also passed with public
SDK `14`, local SDK `85`, Web `179`, and the production build. The local contract verifier observed
`graph_diff_application=passed count=1`; its aggregate remains `unobserved` only for Git, browser,
and production evidence that was not run. PostgreSQL runtime, Docker/virtualization, authenticated
browser, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.
No secret was read and no API, OpenAPI, SDK, Web mutation, storage migration, or second graph-diff
calculator was added.

新鲜本地证据均通过：`cargo fmt --all -- --check`；`cargo test -p contextlab-diff-engine --all-targets`
为 `20 passed`；`cargo test --workspace --quiet`（API `184 passed`、storage `179 passed, 39 ignored`）；
`cargo clippy --workspace --all-targets --offline -- -D warnings`；以及
`cargo +1.85.0 check --workspace --all-targets --locked`。`pnpm check:web` 也已通过，其中 public SDK `14`、
local SDK `85`、Web `179`，并完成 production build。本地 contract verifier 观测到
`graph_diff_application=passed count=1`；其 aggregate 仅因未运行 Git、browser 与 production evidence 而保持
`unobserved`。PostgreSQL runtime、Docker/virtualization、authenticated browser、remote CI、operator rehearsal、
release 与 production 仍为 `unobserved` 或 `deferred`。未读取 secret，也未新增 API、OpenAPI、SDK、Web mutation、
storage migration 或第二个 graph-diff calculator。

**Next dependency pointer / 下一依赖指针:** The review found that the existing public
version-backed graph-diff read path needs an explicit access-governance decision before it can be
treated as safe for private Context metadata. A separate bilingual Necessity Record must first
establish the intended public/private compatibility boundary, authentication/RBAC/no-store
requirements, regression strategy, and migration impact. The unified diff contract intentionally
does not create a persistence bridge: exact semantic inputs, sealed behavior outcomes, and fully
observed evaluation evidence remain separate future dependencies.

**下一依赖指针：** 审查发现现有 public version-backed graph-diff read path 在被视为 private Context metadata 的安全
读取前，需要一份明确的 access-governance 决策。必须先另写双语 Necessity Record，定义预期的 public/private
兼容边界、authentication/RBAC/no-store 要求、回归策略与迁移影响。统一 diff contract 有意不创建 persistence bridge：
exact semantic input、sealed behavior outcome 与 fully observed evaluation evidence 仍是后续独立依赖。
