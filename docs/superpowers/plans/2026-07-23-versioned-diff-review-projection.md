# Versioned Diff Review Projection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a minimal local-only Rust projection that binds the existing structured semantic, behavior, and evaluation diff result to two exact Context commit identifiers.

**Architecture:** `contextlab-diff-engine` owns one version-aware review application boundary and depends only on the stable `contextlab-versioning::CommitId` identity. The boundary validates the source/target pair, delegates all comparison work to `ContextDiffService`, and wraps the complete result with an explicit review schema version; `GraphDiff::between` remains the sole graph-diff calculator inside the existing semantic comparison path.

**Tech Stack:** Rust 1.85, Serde, existing `contextlab-diff-engine`, `contextlab-versioning`, and `contextlab-graph` contracts.

---

## Necessity Record / 必要性记录

### Criterion and principle served / 服务的条件与原则

This increment directly advances Completion Criterion 2, **Versioning and diff workflows**: semantic, behavior, and evaluation diffs already exist as reusable tested engine contracts, but their complete result is not yet represented as a review artifact bound to exact source and target commits. It also preserves the architecture rule that future version-backed review must reuse the existing diff engine rather than calculate graph changes in another layer.

本增量直接推进收束条件 2“**版本与 Diff 工作流**”：semantic、behavior 与 evaluation diff 已作为可复用、经过测试的 engine contract 存在，但完整结果尚未被表达为绑定精确 source/target commit 的 review artifact。本增量同时遵守架构约束：后续基于版本的 review 必须复用既有 diff engine，不能在其他层重新计算 graph change。

### Unmet gap and risk / 未满足的缺口与风险

`ContextDiffRequestV1` compares two complete snapshots but carries no immutable version identity. Consumers therefore cannot prove which commits a structured result reviews without adding ad hoc wrapper logic. The main risks are swapped or identical endpoints, a silently changed projection schema, partial output after a comparability failure, unstable ordering, and accidental duplication of `GraphDiff::between`.

`ContextDiffRequestV1` 能比较两份完整 snapshot，但不携带不可变 version identity。因此，consumer 若不增加临时 wrapper logic，就无法证明结构化结果审阅的是哪两个 commit。主要风险包括 source/target 颠倒或相同、projection schema 被静默改变、comparability failure 后仍返回部分结果、顺序不稳定，以及意外重复 `GraphDiff::between`。

### Why this is next / 为什么现在实施

All local dependencies are ready: `CommitId` is stable and serializable, the unified diff contract already produces all three required dimensions, deterministic ordering is already engine-owned, and its semantic path already delegates graph comparison to `GraphDiff::between`. A version-bound projection is therefore the smallest dependency-ready Criterion 2 increment before persistence, transport, UI, merge policy, or broader replay workflow work.

本地依赖均已就绪：`CommitId` 已稳定且可序列化，统一 diff contract 已生成所需的三个维度，确定性顺序已归 engine 所有，semantic path 也已委托 `GraphDiff::between` 进行 graph comparison。因此，在 persistence、transport、UI、merge policy 或更广泛 replay workflow 之前，绑定 version 的 projection 是当前推进条件 2 的最小依赖就绪增量。

### Explicit non-goals / 明确非目标

No graph-diff algorithm, second `GraphDiff` calculation, commit ancestry validation, branch/merge/replay policy, persistence, provider/evaluator call, public or private transport, API, SDK, Web, UI, Docker, migration, secret handling, release, or production claim. This projection does not prove that snapshots were loaded from storage; its caller must supply the exact snapshots corresponding to the exact version identifiers.

不新增 graph-diff algorithm 或第二次 `GraphDiff` 计算；不处理 commit ancestry validation、branch/merge/replay policy、persistence、provider/evaluator call、公有或私有 transport、API、SDK、Web、UI、Docker、migration、secret、release 或 production 声明。本 projection 不证明 snapshot 已从 storage 加载；caller 必须提供与精确 version identifier 对应的精确 snapshot。

### Smallest boundary and documentation / 最小边界与文档

Add one focused review module and one integration-test file under `crates/diff-engine`, plus the direct local path dependency on `contextlab-versioning`. No versioning behavior changes are required. This bilingual plan is the complete documentation boundary for the local increment; roadmap and architecture documents are intentionally unchanged under worker ownership constraints.

在 `crates/diff-engine` 中增加一个聚焦的 review module 与一个 integration test 文件，并增加对 `contextlab-versioning` 的直接本地 path dependency。不需要改变 versioning 行为。本双语计划是该本地增量的完整文档边界；受 worker ownership 约束，roadmap 与 architecture 文档保持不变。

### Fresh verification required / 必须取得的新鲜验证

Observe a RED compile failure for the absent review contract, then GREEN focused tests covering exact version identity, explicit schema version, deterministic structured output, same-version rejection, deserialization rejection, and propagation of structured diff errors. Before handoff, run:

先观察 review contract 缺失导致的 RED 编译失败，再观察 GREEN 聚焦测试，覆盖 exact version identity、显式 schema version、确定性结构化输出、same-version 拒绝、反序列化拒绝与 structured diff error 传播。交付前运行：

```powershell
cargo test -p contextlab-diff-engine --test versioned_diff_review_projection
cargo test -p contextlab-diff-engine
cargo test -p contextlab-versioning
cargo fmt --all -- --check
cargo clippy -p contextlab-diff-engine -p contextlab-versioning --all-targets --all-features -- -D warnings
rg -n "GraphDiff::between" crates/diff-engine/src crates/versioning/src
```

Expected boundary evidence is exactly one production call to `GraphDiff::between`, located in the existing semantic comparison implementation. Docker/PostgreSQL, browser, remote CI, release, and production evidence remain unobserved and outside this increment.

预期边界证据是 production code 中只有一处 `GraphDiff::between` 调用，且仍位于既有 semantic comparison 实现中。Docker/PostgreSQL、browser、remote CI、release 与 production evidence 均保持 unobserved，且不属于本增量。

---

## Contract Design / 契约设计

The public Rust contract will contain:

- `DiffReviewProjectionSchemaVersion::V1`, serialized explicitly as `"v1"`.
- `VersionedContextDiffReviewRequestV1`, containing the schema version, exact `source_version_id`, exact `target_version_id`, and the corresponding complete diff snapshots.
- `VersionedContextDiffReviewProjectionV1`, containing the same schema and exact ordered version pair plus one complete `ContextDiffResultV1`.
- `VersionedContextDiffReviewError`, rejecting identical version IDs and wrapping an underlying `ContextDiffError` together with both exact IDs.
- `VersionedContextDiffReviewService::project`, which validates first and then calls `ContextDiffService::compare` exactly once.

公开 Rust contract 包含：显式序列化为 `"v1"` 的 schema enum；携带 schema、精确 source/target version ID 与完整 snapshot 的 request；携带同一 schema、同一有序 version pair 和完整 `ContextDiffResultV1` 的 projection；拒绝相同 version ID 并连同精确 pair 包装底层 `ContextDiffError` 的结构化错误；以及先校验、再且仅调用一次 `ContextDiffService::compare` 的 projection service。

Request deserialization uses a strict wire representation with unknown-field denial and constructor validation, so Serde cannot bypass the same-version invariant. Deterministic change ordering remains entirely inherited from existing `BTreeMap`/`BTreeSet`-backed diff contracts.

request 反序列化使用拒绝未知字段的严格 wire representation，并重新执行 constructor validation，因此 Serde 无法绕过 same-version invariant。确定性的 change ordering 完全继承既有基于 `BTreeMap`/`BTreeSet` 的 diff contract。

## TDD Tasks / TDD 任务

### Task 1: Observe RED for the absent projection

**Files:**
- Create: `crates/diff-engine/tests/versioned_diff_review_projection.rs`
- Modify: `crates/diff-engine/Cargo.toml`

- [x] Add focused integration tests against the wished-for public API.
- [x] Run the focused test and record the expected unresolved-import compile failure.

### Task 2: Implement the minimal projection

**Files:**
- Create: `crates/diff-engine/src/review.rs`
- Modify: `crates/diff-engine/src/lib.rs`
- Modify: `crates/diff-engine/Cargo.toml`

- [x] Add only the schema, request, projection, error, and delegating service required by the failing tests.
- [x] Run the focused test and record GREEN.
- [x] Run all tests for both owned crates.

### Task 3: Verify quality and ownership boundaries

**Files:**
- Modify: `docs/superpowers/plans/2026-07-23-versioned-diff-review-projection.md`

- [x] Run formatting check and strict Clippy for both owned crates.
- [x] Search production source for the sole `GraphDiff::between` call.
- [x] Record exact observed evidence and remaining unobserved boundaries below.

## Verification Receipt / 验证回执

**RED observed / 已观察到 RED：** `cargo test -p contextlab-diff-engine --test versioned_diff_review_projection` exited `1` before production implementation. Rust emitted `E0432` for exactly the absent `DiffReviewProjectionSchemaVersion`, `VersionedContextDiffReviewError`, `VersionedContextDiffReviewRequestV1`, and `VersionedContextDiffReviewService` exports. This is the expected missing-feature compile failure, not an unrelated setup error.

**GREEN observed / 已观察到 GREEN：** after the minimal implementation, the same focused command exited `0` with `4 passed; 0 failed`. It covers exact ordered source/target version IDs, serialized `schema_version: "v1"`, inherited deterministic semantic/behavior/evaluation order, same-version rejection, strict unknown/unsupported/identical request decoding, and exact IDs on a wrapped `EvaluationComparabilityMismatch`.

**Owned-crate verification / 所有权 crate 验证：**

- `cargo fmt -p contextlab-diff-engine -- --check` exited `0` after formatting only the owned crate.
- `cargo test -p contextlab-diff-engine` exited `0`: unit `5`, behavior/evaluation `4`, receipt integration `2`, unified contract `1`, versioned review `4`, doc tests `0` failures.
- `cargo test -p contextlab-versioning` exited `0`: `23 passed; 0 failed` plus doc tests.
- `cargo clippy -p contextlab-diff-engine -p contextlab-versioning --all-targets --all-features -- -D warnings` exited `0`.
- `rg -n "GraphDiff::between" crates/diff-engine/src crates/versioning/src` found the only production call at `crates/diff-engine/src/application.rs:86`; the other three hits are existing unit-test assertions in `crates/diff-engine/src/lib.rs`.

**Global formatting boundary / 全局格式化边界：** `cargo fmt --all -- --check` was run and exited `1` because unrelated files under `apps/cli`, `crates/evaluation`, `crates/knowledge`, and `crates/workflow` have formatting drift. Its first run also identified the newly added diff files before the owned-crate formatter was run. Those external files were not modified. This is a repository-wide formatting blocker, not a compile failure in this increment.

**Unobserved and out of scope / 未观测且超出范围：** Docker/PostgreSQL runtime, browser, remote CI, release, production, persistence-backed snapshot loading, transport, UI, provider/evaluator calls, and commit ancestry validation remain unobserved or intentionally absent. No graph-diff algorithm or second `GraphDiff` calculation was added.
