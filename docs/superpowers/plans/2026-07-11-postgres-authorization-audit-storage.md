# PostgreSQL Authorization Audit Storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist safe authorization decisions through the existing `AuthorizationAuditSink` port and automatically compose the durable PostgreSQL adapter for PostgreSQL-backed API state without publishing a commit mutation contract.

**Architecture:** `contextlab-auth` retains framework-independent decisions and adds stable storage values. `contextlab-storage` owns migration `0005`, implements the existing sink on `PostgresContextGraphRepository`, and maps every SQLx failure to the already-safe `AuthorizationAuditError::Unavailable`. `server/api` chooses the adapter only for PostgreSQL `WorkspaceDataRepository`; the guarded route continues to record before its single guarded-write operation.

**Tech Stack:** Rust stable, Tokio, SQLx/PostgreSQL, Axum, Serde, existing ContextLab auth and storage crates.

---

## File Responsibilities

- `crates/auth/src/authorization.rs`: stable persistence-safe values for `ContextPermission` and `AuthorizationDecision`.
- `crates/auth/src/lib.rs`: unit tests for those values and the event contract.
- `crates/storage/migrations/0005_context_authorization_audit_events.sql`: append-only event table, checks, foreign key, and indexes.
- `crates/storage/src/lib.rs`: includes migration `0005` in the composed migration asset and asserts its schema invariants.
- `crates/storage/src/postgres.rs`: SQL insert adapter plus disposable PostgreSQL persistence evidence.
- `server/api/src/lib.rs`: PostgreSQL-only runtime composition for the durable sink and its pure configuration test.
- `docs/adr/0002-guarded-context-commit-writes.md`, `docs/roadmap/completion-criteria.md`, `docs/roadmap/long-term-roadmap.md`: bilingual boundary update; no claim that the public mutation is promoted.

## Task 1: Stabilize Domain Storage Values

**Files:**
- Modify: `crates/auth/src/authorization.rs`
- Modify: `crates/auth/src/lib.rs`

- [ ] **Step 1: Write the failing value-contract test**

```rust
#[test]
fn authorization_values_have_stable_storage_representations() {
    assert_eq!(ContextPermission::Read.as_str(), "read");
    assert_eq!(ContextPermission::Write.as_str(), "write");
    assert_eq!(AuthorizationDecision::Granted.as_str(), "granted");
    assert_eq!(AuthorizationDecision::Forbidden.as_str(), "forbidden");
    assert_eq!(AuthorizationDecision::Unavailable.as_str(), "unavailable");
}
```

- [ ] **Step 2: Run the focused test and observe the missing methods**

Run: `cargo test -p contextlab-auth authorization_values_have_stable_storage_representations`

Expected: failure because `as_str` is not defined on the two domain enums.

- [ ] **Step 3: Add minimal framework-independent values**

```rust
impl ContextPermission {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

impl AuthorizationDecision {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Granted => "granted",
            Self::Forbidden => "forbidden",
            Self::Unavailable => "unavailable",
        }
    }
}
```

- [ ] **Step 4: Run focused auth tests**

Run: `cargo test -p contextlab-auth`

Expected: all auth tests pass.

## Task 2: Add the Append-Only PostgreSQL Asset

**Files:**
- Create: `crates/storage/migrations/0005_context_authorization_audit_events.sql`
- Modify: `crates/storage/src/lib.rs`

- [ ] **Step 1: Extend the migration assertion first**

Add these exact expected fragments to `migration_declares_core_tables_and_indexes`:

```rust
"CREATE TABLE context_authorization_audit_events",
"CHECK (permission IN ('read', 'write'))",
"CHECK (decision IN ('granted', 'forbidden', 'unavailable'))",
"REFERENCES contexts(id) ON DELETE RESTRICT",
"CREATE INDEX idx_context_authorization_audit_events_context_recorded_at",
"CREATE INDEX idx_context_authorization_audit_events_principal_recorded_at",
```

- [ ] **Step 2: Run the migration assertion and observe its failure**

Run: `cargo test -p contextlab-storage migration_declares_core_tables_and_indexes`

Expected: failure because migration `0005` is absent from `CONTEXT_PLATFORM_MIGRATION`.

- [ ] **Step 3: Create the migration and include it in the composed asset**

```sql
CREATE TABLE context_authorization_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    principal_id TEXT NOT NULL CHECK (length(trim(principal_id)) > 0),
    context_id UUID NOT NULL REFERENCES contexts(id) ON DELETE RESTRICT,
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write')),
    decision TEXT NOT NULL CHECK (decision IN ('granted', 'forbidden', 'unavailable')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_context_authorization_audit_events_context_recorded_at
    ON context_authorization_audit_events(context_id, recorded_at DESC, id DESC);
CREATE INDEX idx_context_authorization_audit_events_principal_recorded_at
    ON context_authorization_audit_events(principal_id, recorded_at DESC, id DESC);

CREATE FUNCTION prevent_context_authorization_audit_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'context authorization audit events are append-only';
END;
$$;

CREATE TRIGGER context_authorization_audit_events_append_only
    BEFORE UPDATE OR DELETE ON context_authorization_audit_events
    FOR EACH ROW
    EXECUTE FUNCTION prevent_context_authorization_audit_event_mutation();
```

Append `include_str!("../migrations/0005_context_authorization_audit_events.sql")` to `CONTEXT_PLATFORM_MIGRATION` after migration `0004`.

- [ ] **Step 4: Run focused storage migration tests**

Run: `cargo test -p contextlab-storage migration_`

Expected: all migration tests pass.

## Task 3: Implement the Durable Sink

**Files:**
- Modify: `crates/storage/src/postgres.rs`

- [ ] **Step 1: Write an ignored disposable PostgreSQL test before implementation**

Use `disposable_test_pool(2)` and `seeded_context_id()` to call `repository.record` for the three decisions. Assert that both `UPDATE` and `DELETE` fail, then query only `principal_id`, `context_id`, `permission`, `decision`, and `recorded_at`:

```rust
let events = sqlx::query_as::<_, (String, Uuid, String, String, DateTime<Utc>)>(
    "SELECT principal_id, context_id, permission, decision, recorded_at
     FROM context_authorization_audit_events
     ORDER BY recorded_at ASC, id ASC",
)
.fetch_all(&pool)
.await
.expect("read audit events");

assert_eq!(events.len(), 3);
assert_eq!(events[0].0, "user:alex");
assert_eq!(events[0].1, context_id.as_uuid());
assert_eq!(events[0].2, "write");
assert_eq!(events[0].3, "granted");
```

- [ ] **Step 2: Run the test with the ignored suite disabled and observe the missing trait implementation**

Run: `cargo test -p contextlab-storage postgres_authorization_audit_sink_persists_safe_decisions -- --ignored`

Expected: compile failure because `PostgresContextGraphRepository` does not implement `AuthorizationAuditSink`.

- [ ] **Step 3: Add the adapter with no SQL error disclosure**

Add one constant and one trait implementation near the existing PostgreSQL authorizer:

```rust
const INSERT_CONTEXT_AUTHORIZATION_AUDIT_EVENT_SQL: &str = r#"
INSERT INTO context_authorization_audit_events (
    principal_id, context_id, permission, decision
) VALUES ($1, $2, $3, $4)
"#;

#[async_trait]
impl AuthorizationAuditSink for PostgresContextGraphRepository {
    async fn record(
        &self,
        event: AuthorizationAuditEvent,
    ) -> Result<(), AuthorizationAuditError> {
        sqlx::query(INSERT_CONTEXT_AUTHORIZATION_AUDIT_EVENT_SQL)
            .bind(event.principal_id().as_str())
            .bind(event.context_id().as_uuid())
            .bind(event.permission().as_str())
            .bind(event.decision().as_str())
            .execute(&self.pool)
            .await
            .map_err(|_| AuthorizationAuditError::Unavailable)?;

        Ok(())
    }
}
```

- [ ] **Step 4: Run focused storage tests**

Run: `cargo test -p contextlab-storage postgres_authorization_audit`

Expected: unit tests pass and the disposable test remains explicitly ignored without a configured test database.

## Task 4: Compose the Adapter Only for PostgreSQL Runtime

**Files:**
- Modify: `server/api/src/lib.rs`

- [ ] **Step 1: Write the failing pure composition test**

Add a private `WorkspaceDataRepository::postgres_authorization_audit_sink` selector and test it directly inside `server/api/src/lib.rs`:

```rust
#[test]
fn postgres_workspace_data_exposes_a_durable_authorization_audit_sink() {
    let repository = WorkspaceDataRepository::Postgres(
        PostgresContextGraphRepository::connect_lazy("postgres://localhost/contextlab")
            .expect("valid lazy repository"),
    );

    assert!(repository.postgres_authorization_audit_sink().is_some());
    assert!(WorkspaceDataRepository::Memory(
        InMemoryContextGraphRepository::context_engineering_preview()
    )
    .postgres_authorization_audit_sink()
    .is_none());
}
```

- [ ] **Step 2: Run the focused test and observe the missing selector**

Run: `cargo test -p contextlab-api postgres_workspace_data_exposes_a_durable_authorization_audit_sink`

Expected: failure because `postgres_authorization_audit_sink` is not defined.

- [ ] **Step 3: Select the sink before moving repository clones into AppState**

```rust
let workspace_data_repository = workspace_data_repository_from_env(&env)?;
let authorization_audit_sink = workspace_data_repository.postgres_authorization_audit_sink();
let state = Self::with_workspace_repositories(/* existing clone composition */);

Ok(match authorization_audit_sink {
    Some(audit_sink) => state.with_authorization_audit_sink(audit_sink),
    None => state,
})
```

Implement the selector as:

```rust
fn postgres_authorization_audit_sink(&self) -> Option<PostgresContextGraphRepository> {
    match self {
        Self::Memory(_) => None,
        Self::Postgres(repository) => Some(repository.clone()),
    }
}
```

- [ ] **Step 4: Run focused API tests**

Run: `cargo test -p contextlab-api authorization --lib`

Expected: the existing granted-event and audit-unavailable fail-closed tests pass, and no public mutation route is added.

## Task 5: Document the Persisted Boundary and Verify

**Files:**
- Modify: `docs/adr/0002-guarded-context-commit-writes.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-11-guarded-context-commit-write.md`

- [ ] **Step 1: Update bilingual documentation**

State precisely that PostgreSQL composition now persists minimal authorization decisions, that the public commit write route/OpenAPI/SDK remain closed, and that production JWT/OIDC, key rotation, broader RBAC, rate limiting, audit retention, database CI, and release automation remain unfinished.

- [ ] **Step 2: Run focused contract checks**

Run: `cargo test -p contextlab-api public_post_routes_match_openapi_contract`

Expected: passes, proving the public mutation catalog remains unchanged.

- [ ] **Step 3: Run release gates**

Run: `cargo fmt --all -- --check`

Expected: exit `0`.

Run: `cargo test --workspace`

Expected: all non-ignored Rust tests pass.

Run: `pnpm check:web`

Expected: SDK and Web lint, tests, and production build pass.

Run: `python apps/web/verify-context-workspace.py`

Expected: the Context workspace visual/contract verifier exits `0`.

## Completion Notes / 收束说明

Do not initialize, change, or rely on Git metadata: this checkout does not expose a usable repository history. Do not inspect `.env`, print database URLs, or add a public endpoint. The long-term goal remains open after this slice; the next priority is production authorization configuration and an explicit release-promotion decision, not automatic publication of the guarded mutation.

**Execution:** Completed on 2026-07-11 with test-first domain, migration, adapter, runtime-composition, and disposable PostgreSQL evidence. The release gates remain recorded as commands because this checkout does not expose usable Git history for commits.
