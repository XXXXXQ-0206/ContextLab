# Shared Rate Limit Implementation Plan / 共享限流实施计划

**Goal / 目标：** Add a private, PostgreSQL-backed atomic limiter for protected Context writes so independently running API replicas enforce one issuer-scoped quota.

## Necessity Record / 必要性记录

**Criterion / 条件：** Production security and reliable release gates require a shared atomic multi-replica limiter before public protected writes can be considered.

**Gap / 缺口：** `InMemoryProtectedRouteRateLimiter` is atomic only within one process. Multiple replicas can each admit the full allowance, invalidating the protected-write quota contract.

**Why now / 为什么现在：** Disposable PostgreSQL, private purge execution, and forward migration rehearsal now have fresh local evidence. Shared atomic limiting is the remaining dependency-ready protected-write reliability gate before any public-write readiness decision.

**Non-goals / 非目标：** No public write promotion, API/OpenAPI/SDK/Web change, Redis/NATS introduction, cross-tenant aggregation, anonymous keying, or GraphDiff change.

**Smallest boundary / 最小边界：** Add a storage migration plus a private `ProtectedRouteRateLimiter` adapter using PostgreSQL transaction/row-lock semantics; keep the existing in-memory adapter for single-process test and development modes.

**Required evidence / 所需证据：** Prove two independently constructed adapters sharing one disposable PostgreSQL database admit at most the configured total; prove exact issuer+subject+operation isolation, expiry recovery, outage fail-closed behavior, and no route/API surface expansion. Run fresh storage, workspace, Web, and guarded database suites before another increment.

## Architecture / 架构

`contextlab-auth` continues to own the framework-independent `ProtectedRouteRateLimiter` port,
policy, key, and decision/error semantics. `contextlab-storage` supplies only a private
`PostgresProtectedRouteRateLimiter` adapter; it is not wired into an HTTP route or exported through
OpenAPI, the TypeScript SDK, or the Web application.

`00010_shared_protected_route_rate_limits.sql` will persist a singleton policy/clock guard and one
bounded timestamp queue per `(identity_source, principal_id, operation)`. Each check executes in one
PostgreSQL transaction under a transaction-scoped advisory lock: it validates the shared policy,
rejects backward database-clock observations, removes expired keys, applies the global active-key
capacity, locks the requested key row, prunes the sliding window, and either appends one timestamp or
returns a rounded-up retry delay. The intentionally narrow protected-write path favors a correctness
proof over premature lock-striping; a future throughput change must preserve this atomic contract and
add a separate Necessity Record.

`contextlab-auth` 继续拥有无框架依赖的 `ProtectedRouteRateLimiter` port、policy、key 与
decision/error 语义。`contextlab-storage` 只提供私有的
`PostgresProtectedRouteRateLimiter` adapter；它不会接入 HTTP route，也不会进入 OpenAPI、
TypeScript SDK 或 Web 应用。

`00010_shared_protected_route_rate_limits.sql` 将持久化 singleton policy/clock guard，以及每个
`(identity_source, principal_id, operation)` 的有界 timestamp queue。每次检查在一个
PostgreSQL transaction 内、受 transaction-scoped advisory lock 保护：验证共享 policy、拒绝
倒退的 database clock observation、删除过期 key、应用全局 active-key capacity、锁定请求 key row、
prune sliding window，然后 append 一个 timestamp 或返回向上取整的 retry delay。此处刻意狭窄的
protected-write path 优先选择正确性证明而不是过早 lock-striping；未来吞吐量改动必须保持该原子契约，
并另写 Necessity Record。

## Implementation Plan / 实施计划

### Task 1: Establish the failing shared-replica contract / 建立失败的多副本契约

**Files:**
- Modify: `crates/storage/src/postgres.rs`
- Modify: `scripts/verify-disposable-postgres-storage.sh`

- [ ] Add an ignored disposable-PostgreSQL test that constructs two pools and two future
  `PostgresProtectedRouteRateLimiter` instances against the same empty database. Start more checks
  than a policy with three permits allows, then assert exactly three `Allowed` decisions and only
  `Rejected` decisions for the remainder.
- [ ] Add a second ignored integration test for issuer/subject isolation, state expiry at the
  sliding-window boundary, and policy/configuration drift failing closed.
- [ ] Add a non-ignored closed-pool test showing the adapter maps storage acquisition failure to
  `RateLimitError::Unavailable` without exposing a connection string.
- [ ] Run the focused test target and record the expected red compilation failure because the private
  PostgreSQL adapter and migration do not yet exist.

### Task 2: Add the persistence contract and adapter / 增加持久化契约与 adapter

**Files:**
- Create: `crates/storage/migrations/0010_shared_protected_route_rate_limits.sql`
- Create: `crates/storage/src/protected_route_rate_limit.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/postgres.rs`

- [ ] Create one migration-owned configuration row that fixes the policy values and monotonic
  database observation for all replicas, plus a case-sensitive, bounded state table keyed by identity
  source, subject, and protected operation. Include a timestamp-array cardinality bound and an expiry
  index; do not introduce a privileged role or public transport.
- [ ] Add `PostgresProtectedRouteRateLimiter::from_pool(PgPool, ProtectedRouteRateLimitPolicy)` and
  implement `contextlab_auth::ProtectedRouteRateLimiter`. Map every SQLx failure and policy drift to
  `RateLimitError::Unavailable`.
- [ ] In one transaction, acquire a fixed transaction-scoped PostgreSQL advisory lock, obtain the
  database clock, validate/update the shared configuration, delete expired state, enforce the active
  key bound, lock the requested row, prune timestamps at the exact `<= window boundary`, and commit
  either the appended permit or the pruned rejection state.
- [ ] Round retry delays up to a positive whole second and keep storage SQL values internal; no
  database error text may cross the auth port.

### Task 3: Prove the adapter and migration / 证明 adapter 与 migration

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/postgres.rs`
- Modify: `scripts/verify-disposable-postgres-storage.sh`

- [ ] Add migration-asset assertions for both tables, the identity/operation primary key, bounded
  policy fields, expiry index, and the absence of `SECURITY DEFINER`.
- [ ] Run the previously red focused tests until they pass. Add the two ignored PostgreSQL test names
  to the reset-per-test disposable suite so CI runs each from a fresh schema.
- [ ] Confirm the new adapter has no API-route, OpenAPI, TypeScript SDK, or Web references.

### Task 4: Record the boundary and evidence / 收束边界与证据记录

**Files:**
- Modify: `README.md`
- Modify: `ARCHITECTURE.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/goal-governance.md`

- [ ] Update English and Chinese documentation to distinguish the private shared adapter from any
  public write readiness decision, list local disposable evidence precisely, and retain remote CI and
  operator-approved production rehearsal as open gates.
- [ ] If verification finds a root cause, amend this Necessity Record and the governance decision log
  with the blocked condition, smallest repair, and regression proof before continuing.

### Task 5: Fresh cross-stack verification / 新鲜跨栈验证

**Files:**
- Verify only

- [ ] Run the named disposable PostgreSQL suite through Git Bash and remove its temporary Docker
  resources after completion.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, the two shell
  regression scripts, and shell syntax checks.
- [ ] Inspect route catalogs, OpenAPI, SDK, and Web references to prove this private reliability
  increment did not expand the public surface. A remote GitHub Actions result and an
  operator-approved production change rehearsal remain explicitly open unless fresh external evidence
  is obtained.

## Gate-Review Correction / 门禁审阅修正

**Blocked criterion / 被阻塞条件：** The shared adapter must improve multi-replica protected-write
reliability without introducing a global availability bottleneck or accepting malformed future state.

**Root cause / 根因：** The initial correct-by-serialization implementation acquired one global
transaction advisory lock for every request, including independent existing keys. That turns the
protected path into a single queue and can exhaust a replica's database pool under contention. Its
timestamp queue also trusted a future value written outside the adapter, which could silently loosen a
window after a wall-clock anomaly. The persisted policy remains intentionally fail-closed on drift;
automatic policy rotation is a non-goal because changing a shared quota requires an operator-approved
deployment procedure.

**最小修复 / Minimal fix：** Keep the same state table and auth port, but acquire a deterministic
per-key advisory lock for every check. Use the existing global advisory lock only when an absent key
needs global expiry reclamation, capacity accounting, and insertion. Treat a requested key whose latest
stored timestamp is later than the database observation as `RateLimitError::Unavailable`. Keep policy
drift fail-closed and document that the limit is a count of active identity-operation keys, not unique
subjects.

**Regression proof / 回归证明：** Add a disposable PostgreSQL test that holds the global admission
lock while an existing key is still admitted through another pool, and one that injects a future state
timestamp and expects fail-closed. Re-run both existing multi-pool tests, the 15-item reset suite,
workspace checks, and the public-surface search before this increment can close.
