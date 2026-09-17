# Private Component Content Revisions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标:** Add a private, commit-bound component-content revision boundary so an existing Context component can receive replayable UTF-8 body updates without creating a public write surface.

**Architecture / 架构:** `context-core` owns opaque UTF-8 content and deterministic SHA-256 fingerprints; `versioning` records the previous and resulting fingerprint in an `UpdatedComponent` change. `contextlab-storage` attaches one immutable body revision to an existing guarded commit transaction, updates only the component's hash projection under the same transaction, and exposes a private read port for a revision addressed by `(context, commit, component)`. The HTTP route catalog, OpenAPI, TypeScript SDK, Web controls, and `GraphDiff` stay unchanged.

**Tech Stack / 技术栈:** Rust stable, Serde, SHA-256, Tokio, SQLx/PostgreSQL, existing guarded commit writer, Cargo tests.

---

## Necessity Record / 必要性记录

**Completion conditions / 服务的收束条件：** This increment directly advances `Context-first platform coverage` because current component records contain only `content_hash` and metadata, not reconstructible body content. It also advances `Versioning and diff workflows` because the existing `UpdatedComponent` change has no before/after content reference. The bilingual governance correction serves `Bilingual documentation` and makes the external evidence protocol an additional release/promotion proof for `Production security and collaboration` and `Reliable release gates`, rather than an implementation-wide stop condition.

本增量直接推进 `以 Context 为核心的平台覆盖`：当前 component record 只有 `content_hash` 和 metadata，没有可重建的正文内容；同时推进 `版本与 Diff 工作流`：既有 `UpdatedComponent` change 没有记录更新前后的内容引用。双语治理修正服务于 `中英双语文档`，并把外部证据协议定位为 `生产级安全与协作`、`可靠发布门禁` 的额外发布/推广证明，而非全部实现工作的停止条件。

**Gap and priority / 缺口与优先级：** `ContextComponentRepository` can inspect an existing component but `ComponentDetail` explicitly predates body-content storage. `ContextChangeKind::UpdatedComponent` exists while its data has no old/new hash. Existing guarded commits already provide authorization re-check, idempotency, and branch-head compare-and-swap, so a private revision can reuse a proven transaction instead of creating a write bypass. This is higher priority than a UI editor because the editor would otherwise own unpublished body/version logic.

`ContextComponentRepository` 可以检查既有 component，但 `ComponentDetail` 明确早于正文存储；`ContextChangeKind::UpdatedComponent` 已存在，却没有旧/新 hash 数据。既有 guarded commit 已提供 authorization 复核、idempotency 与 branch-head compare-and-swap，因此私有 revision 可以复用已验证的事务，而不是创建写入旁路。这优先于 UI editor，因为否则 editor 会持有未沉淀的正文/版本逻辑。

**External release evidence / 外部发布证据：** Remote disposable-CI and operator-approved rehearsal receipts remain `unobserved` and are deferred release evidence. They are rechecked only for public protected-write promotion, release, or production rollout. They do not block this private core increment and no receipt is fabricated, requested, or inferred from local tests.

远端 disposable-CI 与 operator 批准 rehearsal 回执仍为 `unobserved`，属于延期的发布证据；只在考虑受保护公开写入、发布或生产推广时重新检查。它们不阻断本私有核心增量，也不会由本地测试伪造、索取或推断回执。

**Non-goals / 非目标：** Do not create components, add public REST/SDK write methods, expose component bodies through existing public GET routes, add Web mutation controls, add operator transport, change GraphDiff, implement merge/rollback checkout, or promote a release. Existing historical components remain body-unavailable; no historical body is guessed.

不创建 component，不增加 public REST/SDK 写方法，不通过现有 public GET route 暴露正文，不增加 Web 写控件、operator transport，不修改 GraphDiff，不实现 merge/rollback checkout，也不做发布推广。既有历史 component 保持正文不可用，绝不猜测历史正文。

**Minimal boundary and bilingual documentation / 最小边界与双语文档：** Change only `context-core`, `versioning`, `storage`, the additive migration, targeted roadmap/README language, and this plan. Update the protocol and governance in both languages to distinguish release evidence from core delivery. No API, SDK, or Web files change.

**Fresh verification before the next increment / 下一增量前所需的新鲜验证：** Observe red tests before implementation; run focused core/versioning/storage tests, the local disposable PostgreSQL content-revision contract when Docker is available, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, the two artifact-shape scripts, and source-boundary searches proving no API/SDK/Web write surface or second graph-diff calculator was added.

## File Structure / 文件结构

- Modify: `crates/context-core/src/component.rs` and `crates/context-core/src/lib.rs` — reusable UTF-8 body value and SHA-256 fingerprint construction.
- Modify: `crates/context-core/Cargo.toml` — use the workspace SHA-256 dependency.
- Modify: `crates/versioning/src/change.rs` — store optional prior/resulting body hashes for component updates.
- Create: `crates/storage/src/component_content_revision.rs` — private command, immutable revision record, read repository port, and command validation.
- Modify: `crates/storage/src/guarded_commit_write.rs` — carry an optional validated component-body revision through the existing guarded transaction.
- Modify: `crates/storage/src/memory.rs` and `crates/storage/src/postgres.rs` — atomically store the revision and update the hash projection after guarded checks.
- Modify: `crates/storage/src/lib.rs` — export the private storage contract and include the additive migration.
- Create: `crates/storage/migrations/0012_component_content_revisions.sql` — append-only revision storage and indexes.
- Modify: `docs/roadmap/goal-governance.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/long-term-roadmap.md`, `docs/roadmap/completion-criteria.md`, `docs/roadmap/external-release-evidence-protocol.md`, and `README.md` — bilingual governance scope and current increment status.

## Task 1: Correct the Release-Evidence Scope

**Files:**
- Modify: `docs/roadmap/goal-governance.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/external-release-evidence-protocol.md`
- Modify: `README.md`

- [ ] **Step 1: Replace global-blocker wording with the bilingual two-track rule**

State that `unobserved` remote CI and rehearsal receipts are deferred release evidence. They block only public protected-write promotion, release, or production rollout; private core increments continue through their own Necessity Records.

- [ ] **Step 2: Preserve the anti-forgery rule**

Keep `exact_commit`, redaction, independent review, non-production isolation, migration-asset integrity, role invariants, and cleanup as mandatory receipt fields. State explicitly that local tests cannot fabricate or replace those receipts.

- [ ] **Step 3: Update current core priority**

Name this private existing-component content-revision boundary as the active core increment. Keep all public write, OpenAPI, SDK, Web mutation, and GraphDiff changes out of scope.

- [ ] **Step 4: Verify the document boundary**

Run:

```powershell
rg -n "only admitted next increment|only current priority|deferred release evidence|public protected-write promotion" docs/roadmap README.md
```

Expected: no statement treats missing receipts as a global implementation stop; release-promotion requirements remain explicit in both languages.

## Task 2: Define Content and Replayable Change Semantics

**Files:**
- Modify: `crates/context-core/Cargo.toml`
- Modify: `crates/context-core/src/component.rs`
- Modify: `crates/context-core/src/lib.rs`
- Test: `crates/context-core/src/component.rs`
- Modify: `crates/versioning/src/change.rs`
- Test: `crates/versioning/src/change.rs`

- [ ] **Step 1: Write failing core tests**

```rust
#[test]
fn fingerprints_utf8_component_content_deterministically() {
    let content = ComponentContent::new("系统提示: 仅返回 JSON");
    assert_eq!(content.content_hash().as_str(), "sha256:EXPECTED_SHA256");
}
```

Add a test that two equal bodies have the same fingerprint and a different body has a different fingerprint.

- [ ] **Step 2: Run the core test to observe RED**

Run: `cargo test -p contextlab-context-core fingerprints_utf8_component_content_deterministically`

Expected: FAIL because `ComponentContent` does not exist.

- [ ] **Step 3: Implement the minimal core values**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentContent(String);

impl ComponentContent {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }

    #[must_use]
    pub fn content_hash(&self) -> ContentHash { /* sha256:<lowercase hex> */ }
}
```

Export the type and use the workspace `sha2` dependency. Keep `ContentHash::new` for legacy hashes.

- [ ] **Step 4: Extend `ContextChange` with component-content hash references**

Add optional `previous_content_hash` and `resulting_content_hash` fields plus `updated_component_content(component_id, kind, previous, resulting, summary)`. Existing constructors keep both fields absent so persisted historical JSON remains deserializable.

- [ ] **Step 5: Run the focused domain tests to observe GREEN**

Run:

```powershell
cargo test -p contextlab-context-core
cargo test -p contextlab-versioning
```

Expected: PASS, with an update change preserving component identity, kind, prior hash, and resulting hash.

## Task 3: Add the Private Revision Contract and Migration

**Files:**
- Create: `crates/storage/src/component_content_revision.rs`
- Modify: `crates/storage/src/lib.rs`
- Create: `crates/storage/migrations/0012_component_content_revisions.sql`
- Test: `crates/storage/src/component_content_revision.rs`

- [ ] **Step 1: Write failing storage-contract tests**

```rust
#[test]
fn revision_requires_a_matching_updated_component_change() {
    let revision = ComponentContentRevisionWrite::new(component_id, previous, body, timestamp);
    assert!(revision.validate_against(&commit).is_err());
}
```

Cover a matching update change, duplicate component revision rejection, and the derived resulting hash.

- [ ] **Step 2: Run the contract test to observe RED**

Run: `cargo test -p contextlab-storage revision_requires_a_matching_updated_component_change`

Expected: FAIL because the private revision contract does not exist.

- [ ] **Step 3: Define the private revision types**

`ComponentContentRevisionWrite` owns `ComponentId`, component kind, expected prior `ContentHash`, `ComponentContent`, and capture time. `ComponentContentRevision` adds the Context and commit identities after validation. `ComponentContentRevisionRepository` reads a revision by exact `(context_id, commit_id, component_id)` and is not attached to `AppState`.

- [ ] **Step 4: Add the additive migration**

Create `context_component_content_revisions` with one immutable row per `(commit_id, component_id)`, foreign keys to the existing commit and component rows, `previous_content_hash`, `content_hash`, UTF-8 `content`, `created_at`, and a context/component lookup index. The existing `context_components.content_hash` remains the current projection and historical rows get no guessed body.

- [ ] **Step 5: Run the storage contract tests to observe GREEN**

Run: `cargo test -p contextlab-storage component_content_revision`

Expected: PASS without requiring a database.

## Task 4: Persist Revisions Through the Existing Guarded Commit Transaction

**Files:**
- Modify: `crates/storage/src/guarded_commit_write.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`
- Test: `crates/storage/src/memory.rs`
- Test: `crates/storage/src/postgres.rs`

- [ ] **Step 1: Write failing in-memory behavior tests**

```rust
#[tokio::test]
async fn guarded_component_content_update_replays_body_revision() {
    let result = repository.create_guarded_commit_snapshot(command).await?;
    assert_eq!(result.disposition, GuardedCommitWriteDisposition::Created);
    let revision = repository.get_component_content_revision(context, commit, component).await?;
    assert_eq!(revision.content().as_str(), "new body");
}
```

Add separate tests for stale expected hash, idempotent replay, and a component from another Context.

- [ ] **Step 2: Run the focused in-memory tests to observe RED**

Run: `cargo test -p contextlab-storage guarded_component_content_update`

Expected: FAIL because guarded writes cannot carry a body revision.

- [ ] **Step 3: Carry one validated revision through `GuardedContextCommitWrite`**

Add an optional builder that accepts one `ComponentContentRevisionWrite`. The guarded writer validates that the commit has exactly the matching `UpdatedComponent` identity, kind, prior hash, and resulting hash. Existing guarded writes keep `None` and retain their exact behavior.

- [ ] **Step 4: Implement atomic memory persistence**

Within the existing guarded write lock, verify the active component belongs to the Context and its projected hash equals the expected prior hash; insert the immutable revision keyed by Context/commit/component; then replace only the in-memory current hash projection. On failure, leave commits, snapshots, revisions, and branch heads unchanged.

- [ ] **Step 5: Implement atomic PostgreSQL persistence**

After authorization, idempotency, and branch-head checks but before transaction commit, lock the active component row with its Context and expected hash; insert the revision; update `context_components.content_hash`; retain the existing commit/snapshot, branch-head, and idempotency operations in the same transaction. An idempotent replay returns the original snapshot without another revision.

- [ ] **Step 6: Run focused storage tests to observe GREEN**

Run:

```powershell
cargo test -p contextlab-storage guarded_component_content_update
cargo test -p contextlab-storage component_content_revision
```

Expected: PASS for create, replay, stale-hash rejection, foreign-context rejection, and exact revision lookup.

## Task 5: Verify, Document, and Preserve Public Boundaries

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Test: existing Rust, shell, and Web checks

- [ ] **Step 1: Document the private boundary in English and Chinese**

State that component bodies are revisioned only through the private guarded storage contract. Existing public component routes still expose metadata and hashes only; historical components without a captured body remain unavailable.

- [ ] **Step 2: Run focused and broad verification**

```powershell
cargo fmt --all -- --check
cargo test -p contextlab-context-core
cargo test -p contextlab-versioning
cargo test -p contextlab-storage
cargo test --workspace
pnpm check:web
bash scripts/verify-ci-evidence-artifact.test.sh
bash scripts/verify-rehearsal-evidence-artifact.test.sh
```

When Docker is available, additionally run the named disposable PostgreSQL content-revision test through the existing guarded test environment.

- [ ] **Step 3: Check non-goal boundaries**

```powershell
rg -n "ComponentContentRevision|component_content_revisions" server/api packages/ts-sdk apps/web/src docs/api/openapi.json
rg -n "GraphDiff" crates/diff-engine crates/graph server/api packages/ts-sdk apps/web/src
```

Expected: no new public transport, SDK, Web mutation, or graph-diff calculator reference; `GraphDiff` remains the sole calculator.

## Plan Self-Review / 计划自审

- Coverage: Tasks 1-5 cover the governance correction, content domain, replayable change metadata, private immutable storage, existing guarded transaction, bilingual documentation, and fresh verification.
- Scope: Component creation, public mutation transport, UI editing, graph changes, and release promotion are explicitly excluded.
- Consistency: `ComponentContentRevisionWrite`, `ComponentContentRevision`, and `ComponentContentRevisionRepository` are used consistently; the command always carries an `UpdatedComponent` prior/resulting hash pair.
