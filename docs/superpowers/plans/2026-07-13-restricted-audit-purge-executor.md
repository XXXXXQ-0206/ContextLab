# Restricted Audit Purge Executor Implementation Plan / 受限审计清理执行器实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal / 目标：** Prove a private, database-role-isolated executor can purge only retention-eligible authorization-audit rows while preserving immutable, redacted manifest evidence and all versioning records.

**Architecture / 架构：** Keep the normal `CONTEXT_PLATFORM_MIGRATION` free of privileged functions and runtime-role assumptions. A separately provisioned PostgreSQL role contract creates a non-login function owner with only the purge data privileges and an executor role with `EXECUTE` only; a privileged SQL artifact owns the `SECURITY DEFINER` procedure. A private storage port is implemented by an adapter constructed with a dedicated executor `PgPool`, never the application's general repository pool. The procedure selects one workspace/policy/cutoff batch under row locks, appends a manifest and redacted items, sets a transaction-local manifest marker, and deletes exactly those rows. No Rust transport, public REST/SDK/OpenAPI method, Web control, or graph-diff behavior is introduced.

**Tech Stack / 技术栈：** Rust stable, Tokio, SQLx, PostgreSQL 16, PL/pgSQL, GitHub Actions, Cargo tests, Bash.

---

## Necessity Record / 必要性记录

**Completion condition and charter principle / 服务的完成条件与宪章原则：** This closes the next named protected-write reliability dependency in `docs/roadmap/goal-governance.md`: a separately proven restricted purge executor under isolated runtime database roles. It advances production security and reliable release-gate evidence while preserving clean domain/storage boundaries, reproducibility, and bilingual documentation.

**Unmet dependency, risk, and evidence gap / 未满足的依赖、风险与证据缺口：** Migration `0008_context_authorization_audit_governance.sql` has immutable retention revisions and append-only selection manifests, but no deletion procedure. The append-only event trigger blocks every ordinary delete; a future `SECURITY DEFINER` function without a non-login owner and execute-only caller would let the executor bypass its own intended restriction. Current local CI evidence proves base storage only, not an isolated privileged deletion boundary.

**Why this is next / 为何此时优先：** `docs/roadmap/long-term-roadmap.md` and `docs/roadmap/completion-criteria.md` identify the isolated role contract and restricted purge executor as the next closure priority after fresh disposable PostgreSQL evidence. Public write promotion, editing, benchmark work, and all other roadmap areas depend on this gate or are lower priority under the convergence-first rule.

**Explicit non-goals / 明确非目标：** No public commit write, operator transport, audit-review route, REST/OpenAPI/SDK method, Web mutation control, automatic scheduler, tenant-facing purge UI, production migration approval, shared multi-replica rate limiter, retention-policy management API, or second graph-diff implementation. `GraphDiff` remains the only graph-diff calculator.

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Add private SQL role/procedure artifacts under `crates/storage`, static migration-contract tests and one ignored disposable PostgreSQL integration test in the storage crate, the named CI test list, and bilingual architecture/roadmap/storage documentation. Do not change `server/api`, `packages/ts-sdk`, `docs/api/openapi.json`, or `apps/web`.

**Fresh verification required before another increment / 下一增量前必须取得的新鲜验证：** Run a fresh PostgreSQL 16 disposable container with the new isolated-role test after a per-test schema reset; run `cargo fmt --all -- --check`, `cargo test --workspace`, the storage script regression, `pnpm check:web`, and a remote GitHub Actions execution. Production migration rehearsal remains a separate evidence requirement and must stay open.

**Gate root-cause update / 门禁根因更新：** The preceding disposable-PostgreSQL gate exposed unsupported `jsonb_object_length`, nanosecond snapshot precision mismatch, and insufficient schema-reset target validation. They were repaired with PostgreSQL-native JSONB counting, microsecond normalization, loopback/role reset guards, and fresh regression evidence in the governance decision log. This executor work must not weaken any of those fixes.

**Task 1 review root cause / Task 1 审查根因：** Specification review found that `ALTER FUNCTION ... OWNER TO contextlab_audit_purge_owner` needs the prospective owner to hold `CREATE` on `public` during ownership transfer, while the role bootstrap granted only `USAGE`. It also found unqualified private table/function DDL, which can create objects outside `public` under an operator-specific `search_path`. The minimum repair grants `CREATE` only immediately before the ownership transfer and revokes it immediately after, explicitly qualifies every private table/function/trigger target as `public.*`, and extends the static contract test to fail without those strings. The regression proof is the focused static test before and after the change plus the disposable database role-isolation test in Task 2.

**Task 1 审查根因（中文）：** 规格审查发现，`ALTER FUNCTION ... OWNER TO contextlab_audit_purge_owner` 要求候选 owner 在所有权转移期间拥有 `public` 的 `CREATE` 权限，而当前 role bootstrap 只授予了 `USAGE`。审查还发现私有 table/function DDL 没有显式限定 schema，operator 的自定义 `search_path` 可能把对象创建到 `public` 之外。最小修复是在转移所有权前临时授予 `CREATE`、转移后立即撤销，显式将所有私有 table/function/trigger target 限定为 `public.*`，并扩展静态契约测试，使缺少这些字符串时失败。回归证明是修改前后的 focused static test，以及 Task 2 的 disposable database 角色隔离测试。

**Task 2 gate root cause / Task 2 门禁根因：** The first real executor call failed because PostgreSQL requires `UPDATE` privilege for `FOR UPDATE SKIP LOCKED`, even though the function never performs an update. The non-login definer received only `SELECT` and `DELETE`. The minimal fix grants `UPDATE` on the audit-event table to the definer alone; its no-login/no-membership boundary and the append-only trigger still prevent an interactive actor from updating a row. The regression proof is the same dedicated runtime-role test, which must pass the function call while direct runtime `DELETE` and `SELECT` remain rejected.

**Task 2 门禁根因（中文）：** 首次真实 executor 调用失败，是因为 PostgreSQL 对 `FOR UPDATE SKIP LOCKED` 要求 `UPDATE` 权限，即使函数本身从不执行更新。无登录 definer 角色当前只获得了 `SELECT` 与 `DELETE`。最小修复仅向 definer 授予 audit-event table 的 `UPDATE`；其无登录/无成员关系边界与 append-only trigger 仍会阻止交互式 actor 更新行。回归证明仍是同一条 dedicated runtime-role test：函数调用必须通过，而 runtime 的直接 `DELETE` 与 `SELECT` 仍必须被拒绝。

**Task 2 second root cause / Task 2 第二个根因：** The next real call reached manifest creation and showed that `INSERT ... RETURNING id` additionally needs `SELECT` on the returned manifest column. The definer already had `INSERT`, while the runtime retained no table privileges. The minimal fix adds `SELECT` only on the immutable manifest table to the non-login definer and extends the static contract assertion; the same runtime-role test proves the call can proceed without granting the runtime table access.

**Task 2 第二个根因（中文）：** 下一次真实调用已到达 manifest 创建阶段，并证明 `INSERT ... RETURNING id` 还需要对返回的 manifest column 具备 `SELECT` 权限。definer 已拥有 `INSERT`，runtime 仍未拥有任何表权限。最小修复只向无登录 definer 授予不可变 manifest table 的 `SELECT`，并扩展静态契约断言；同一 runtime-role test 将证明调用可继续，而无需向 runtime 授予表访问权限。

## File Map / 文件映射

- Create: `crates/storage/authorization_audit_purge.rs` — private storage command/result types and `AuthorizationAuditPurgeExecutor` port.
- Create: `crates/storage/privileged/contextlab_audit_purge_roles.sql` — idempotent operator-owned role bootstrap for a non-login purge-function owner and execute-only purge caller.
- Create: `crates/storage/privileged/contextlab_audit_purge_executor.sql` — private `SECURITY DEFINER` function, redacted manifest items, and narrow trigger exception.
- Modify: `crates/storage/src/lib.rs` — expose privileged SQL artifacts without adding them to `CONTEXT_PLATFORM_MIGRATION`; add static contract tests.
- Modify: `crates/storage/src/postgres.rs` — add a disposable PostgreSQL role-isolation and purge-invariant integration test.
- Modify: `scripts/verify-disposable-postgres-storage.sh` — add the new ignored PostgreSQL test to the per-test reset list.
- Modify: `scripts/verify-disposable-postgres-storage.test.sh` — update the expected isolated invocation count.
- Modify: `ARCHITECTURE.md`, `README.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/long-term-roadmap.md`, `docs/roadmap/completion-criteria.md`, `docs/roadmap/goal-governance.md` — record private scope, role isolation, verification evidence, and remaining production-migration gap in English and Chinese.

### Task 1: Define privileged role and procedure contracts / 定义特权角色与过程契约

**Files:**
- Create: `crates/storage/privileged/contextlab_audit_purge_roles.sql`
- Create: `crates/storage/privileged/contextlab_audit_purge_executor.sql`
- Modify: `crates/storage/src/lib.rs`

- [ ] **Step 1: Write static contract tests first**

Add focused assertions to `crates/storage/src/lib.rs` that require the privileged artifacts to remain separate from `CONTEXT_PLATFORM_MIGRATION` and require all of the following strings:

```rust
assert!(!CONTEXT_PLATFORM_MIGRATION.contains("SECURITY DEFINER"));
assert!(CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP.contains("NOLOGIN"));
assert!(CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP.contains("contextlab_audit_purge_owner"));
assert!(CONTEXTLAB_AUDIT_PURGE_EXECUTOR.contains("SECURITY DEFINER"));
assert!(CONTEXTLAB_AUDIT_PURGE_EXECUTOR.contains("REVOKE ALL ON FUNCTION"));
assert!(CONTEXTLAB_AUDIT_PURGE_EXECUTOR.contains("GRANT EXECUTE ON FUNCTION"));
assert!(CONTEXTLAB_AUDIT_PURGE_EXECUTOR.contains("context_authorization_audit_purge_manifest_items"));
```

- [ ] **Step 2: Verify the static contract test fails**

Run: `cargo test -p contextlab-storage privileged_audit_purge --lib`

Expected: FAIL because the privileged constants and SQL artifacts do not exist yet.

- [ ] **Step 3: Add the role bootstrap artifact**

Create `contextlab_audit_purge_roles.sql` as an idempotent operator bootstrap. It creates `contextlab_audit_purge_owner` as `NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION`, and `contextlab_audit_purge_executor` as an executor that receives no membership in the owner role. It grants the owner only `USAGE` on `public`, read access required to locate eligible events, `INSERT` on manifests/items, and `DELETE` only on `context_authorization_audit_events`; it grants the executor no direct table mutation privilege.

Use guarded role creation so a disposable test cluster can run the artifact repeatedly:

```sql
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'contextlab_audit_purge_owner') THEN
        CREATE ROLE contextlab_audit_purge_owner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'contextlab_audit_purge_executor') THEN
        CREATE ROLE contextlab_audit_purge_executor NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION;
    END IF;
END;
$$;
```

- [ ] **Step 4: Add the restricted procedure artifact**

Create `contextlab_audit_purge_executor.sql`. It must:

```sql
CREATE TABLE context_authorization_audit_purge_manifest_items (
    manifest_id UUID NOT NULL REFERENCES context_authorization_audit_purge_manifests(id) ON DELETE RESTRICT,
    event_id UUID NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    context_id UUID NOT NULL,
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write')),
    decision TEXT NOT NULL CHECK (decision IN ('granted', 'forbidden', 'unavailable')),
    PRIMARY KEY (manifest_id, event_id),
    UNIQUE (event_id)
);
```

Replace the audit-event mutation trigger with an update-rejecting and delete-narrowing trigger. A delete is accepted only when `current_user` is `contextlab_audit_purge_owner`, the transaction-local `contextlab.audit_purge_manifest_id` is set, and the old event ID exists in that manifest's immutable item table. Every other update or delete raises the existing append-only error.

Implement:

```sql
CREATE FUNCTION purge_context_authorization_audit_events(
    p_workspace_id UUID,
    p_policy_revision_id UUID,
    p_cutoff TIMESTAMPTZ,
    p_limit INTEGER DEFAULT 100
) RETURNS TABLE (manifest_id UUID, purged_event_count BIGINT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, public;
```

The function rejects a non-positive or over-bounded limit, verifies that the supplied policy revision belongs to the workspace, selects only `purge_eligible` rows from that workspace and policy where `purge_eligible_at < p_cutoff`, locks them with `FOR UPDATE SKIP LOCKED`, creates one append-only manifest and redacted item rows, sets the transaction-local manifest marker, deletes exactly the selected IDs, and returns zero without a manifest when no row is eligible. It must revoke `PUBLIC` execution, grant execution only to `contextlab_audit_purge_executor`, and transfer function ownership to `contextlab_audit_purge_owner` after the operator bootstrap has run.

- [ ] **Step 5: Expose artifacts without widening base migration**

Add:

```rust
pub const CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP: &str =
    include_str!("../privileged/contextlab_audit_purge_roles.sql");
pub const CONTEXTLAB_AUDIT_PURGE_EXECUTOR: &str =
    include_str!("../privileged/contextlab_audit_purge_executor.sql");
```

Do not append either artifact to `CONTEXT_PLATFORM_MIGRATION`; their prerequisite is explicit operator role provisioning.

- [ ] **Step 6: Verify static contracts pass**

Run: `cargo test -p contextlab-storage privileged_audit_purge --lib`

Expected: PASS, with base migration still free of `SECURITY DEFINER`.

### Task 2: Add the private executor port and prove role isolation / 添加私有执行器端口并证明角色隔离

**Files:**
- Create: `crates/storage/src/authorization_audit_purge.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/postgres.rs`
- Test: `crates/storage/src/postgres.rs`

- [ ] **Step 1: Write the private storage port and its failing adapter test**

Define a framework-independent private storage port:

```rust
#[async_trait]
pub trait AuthorizationAuditPurgeExecutor: Send + Sync {
    async fn purge_authorization_audit_events(
        &self,
        request: AuthorizationAuditPurgeRequest,
    ) -> Result<AuthorizationAuditPurgeResult, StorageRepositoryError>;
}
```

`AuthorizationAuditPurgeRequest` contains a typed `WorkspaceId`, a policy `Uuid`, a UTC cutoff, and a bounded batch limit; `AuthorizationAuditPurgeResult` contains an optional manifest UUID and count. Add a failing test requiring `PostgresAuthorizationAuditPurgeExecutor` to call only `purge_context_authorization_audit_events`, with a separately supplied `PgPool`. Do not add the executor to `PostgresContextGraphRepository`, `WorkspaceDataRepository`, `AppState`, or any API state.

- [ ] **Step 2: Implement the dedicated-pool adapter**

Add `PostgresAuthorizationAuditPurgeExecutor::from_pool(PgPool)` in `postgres.rs`; it calls the private SQL function with the four typed request values and maps its one-row result. The adapter never issues `SET ROLE`, never assembles SQL dynamically, and never falls back to `CONTEXTLAB_DATABASE_URL`; the caller must supply a pool authenticated as the designated executor role.

- [ ] **Step 3: Write the ignored PostgreSQL role-isolation test before connecting the adapter**

Add `postgres_restricted_audit_purge_requires_executor_role_and_preserves_versioning_rows`. It applies the base migration, role bootstrap, and privileged procedure artifact to an empty disposable test database; inserts an eligible old event, an event exactly at the cutoff, an unexpired event, and a versioning fixture containing a commit, graph snapshot, and idempotency row.

The assertions must cover:

```rust
assert!(direct_delete_error.to_string().contains("permission denied"));
assert_eq!(purge_result.purged_event_count, 1);
assert_eq!(second_purge.purged_event_count, 0);
assert_eq!(remaining_event_ids, vec![boundary_event_id, unexpired_event_id]);
assert_eq!(versioning_state_before, versioning_state_after);
assert!(!manifest_item_json.contains("principal_id"));
assert!(!manifest_item_json.contains("identity_source"));
```

The test provisions a disposable login role with a test-only password, grants it membership in `contextlab_audit_purge_executor`, and connects a separate SQLx pool with that login. It proves the dedicated runtime may `SET ROLE contextlab_audit_purge_executor` and call the adapter, but cannot delete directly, read audit tables, or assume the non-login function owner. Reset to the migration role only for fixture setup and assertions.

- [ ] **Step 4: Run the test and observe the expected failure**

Run against an empty disposable PostgreSQL database:

```powershell
$env:CONTEXTLAB_TEST_DATABASE_URL = '<loopback disposable URL>'
cargo test -p contextlab-storage postgres::tests::postgres_restricted_audit_purge_requires_executor_role_and_preserves_versioning_rows --lib -- --ignored --exact --test-threads=1
```

Expected: FAIL before the privileged artifacts are applied, because the function, role, adapter, and manifest-item table do not exist.

- [ ] **Step 5: Apply the artifacts and connect the executor pool inside test setup**

Use `sqlx::raw_sql(CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP)` then `sqlx::raw_sql(CONTEXTLAB_AUDIT_PURGE_EXECUTOR)` after applying the base migration. Build a second `PgPool` from the explicitly configured disposable test URL with the ephemeral runtime login; keep it out of `PostgresContextGraphRepository::connect_lazy` and every server/API constructor.

- [ ] **Step 6: Verify the isolated-role test passes**

Run the exact ignored test again. Expected: PASS with one precisely purged event, immutable redacted manifest evidence, strict cutoff behavior, direct deletion rejection, idempotency, and byte-for-byte unchanged versioning state.

### Task 3: Keep disposable CI comprehensive / 保持一次性 CI 完整性

**Files:**
- Modify: `scripts/verify-disposable-postgres-storage.sh`
- Modify: `scripts/verify-disposable-postgres-storage.test.sh`

- [ ] **Step 1: Add the test to the guarded CI list**

Append the exact new ignored test name to `storage_tests` after the existing audit-retention integration test:

```bash
postgres::tests::postgres_restricted_audit_purge_requires_executor_role_and_preserves_versioning_rows
```

- [ ] **Step 2: Update the shell regression expectation**

Change the mocked Cargo invocation assertion from `11` to `12`, retaining the remote-host and mismatched-role rejection checks.

- [ ] **Step 3: Run shell verification**

Run:

```powershell
& 'C:\Program Files\Git\bin\bash.exe' scripts/verify-disposable-postgres-storage.test.sh
& 'C:\Program Files\Git\bin\bash.exe' -n scripts/verify-disposable-postgres-storage.sh
```

Expected: both commands exit `0`.

### Task 4: Record private boundary and remaining gates / 记录私有边界与剩余门禁

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/goal-governance.md`

- [ ] **Step 1: Add bilingual boundary wording**

Document that the purge procedure is private SQL callable only by the dedicated executor role, the function owner cannot log in, the normal base migration stays non-privileged, and manifest items preserve only redacted audit facts. State that public routes, OpenAPI, SDK, Web, operator transport, automatic scheduling, shared rate limiting, remote CI success, and production migration rehearsal remain out of scope.

- [ ] **Step 2: Update the governance decision log**

Record the completed criterion, runtime role model, discovered blockers, minimal fixes, exact fresh test outputs, and the still-open production migration evidence. Do not change the active long-term goal to complete.

### Task 5: Run fresh evidence gates / 运行新鲜证据门禁

**Files:**
- Verify only

- [ ] **Step 1: Start a new disposable PostgreSQL 16.14 container**

Use a new loopback-bound test database with `contextlab_test_runner`; never read `.env`, inspect an existing database, or reuse a non-disposable database.

- [ ] **Step 2: Run the complete named storage list**

Run `scripts/verify-disposable-postgres-storage.sh` when a local `psql` client is available, or run the same per-test schema-reset sequence through the official container client and document the environment limitation. Expected: all 12 named ignored tests pass.

- [ ] **Step 3: Run cross-stack checks**

Run:

```powershell
cargo fmt --all -- --check
cargo test --workspace
pnpm check:web
```

Expected: all commands exit `0`; the default Rust suite may still report the named database tests as explicitly ignored.

- [ ] **Step 4: Obtain remote CI and preserve the production boundary**

Verify a GitHub Actions run of `.github/workflows/verify.yml` after the change. Record its run identifier or failure evidence in `docs/roadmap/goal-governance.md`. Do not claim production migration readiness without an independently provisioned production-like migration rehearsal.
