# Production Migration Rehearsal Implementation Plan / 生产迁移演练实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal / 目标：** Produce repeatable, non-production evidence that an existing ContextLab PostgreSQL schema can be upgraded through the audited storage and private purge-role contracts without widening the public product surface.

**Architecture / 架构：** Keep regular platform schema migration and privileged role/procedure provisioning as explicitly ordered, separate operator steps. A rehearsal-only script creates a disposable production-like baseline database, applies historical platform migration boundaries forward, provisions privileged roles, records an immutable local migration ledger, then validates post-upgrade data and role invariants. It never connects to an inferred or production database.

**Tech Stack / 技术栈：** PostgreSQL 16.14, SQLx, Bash, Docker, Rust tests, GitHub Actions.

---

## Necessity Record / 必要性记录

**Criterion / 条件：** `docs/roadmap/completion-criteria.md` requires production migration verification before public protected-write readiness can be decided; the charter requires reproducibility, durable storage, secure-by-default operations, and bilingual documentation.

**Gap / 缺口：** Current `CONTEXT_PLATFORM_MIGRATION` is test composition, not a production execution record. The private purge owner/procedure is deliberately outside it. Fresh disposable integration evidence does not prove ordered upgrade behavior, idempotent operator role provisioning, or a durable migration ledger.

**Why now / 为什么现在：** The restricted purge executor and disposable PostgreSQL suite are freshly verified. Production-like migration evidence is the next fixed reliability dependency; public writes, editing, benchmarks, and other roadmap domains remain downstream.

**Non-goals / 非目标：** No live production connection, rollback claim, public write route, operator HTTP transport, OpenAPI/SDK/Web change, automatic scheduler, data deletion beyond the disposable rehearsal database, or GraphDiff change.

**Smallest boundary / 最小边界：** Add a storage-owned migration ledger/rehearsal contract, a guarded disposable script and CI invocation, focused PostgreSQL tests, and bilingual operational documentation. Keep all API, SDK, Web, and default runtime constructors unchanged.

**Required fresh evidence / 所需新鲜证据：** A new PostgreSQL 16.14 disposable database must upgrade from the pre-`0008` schema through current regular schema and privileged provisioning, preserve legacy rows, enforce the private role boundary, write a nonmutable ledger, and pass `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, script syntax/guard tests, and a remote CI run. A real production change window remains a separate operator decision.

### Task 1: Define a storage migration ledger / 定义存储迁移账本

**Files:**
- Create: `crates/storage/migrations/0009_contextlab_migration_ledger.sql`
- Modify: `crates/storage/src/lib.rs`
- Test: `crates/storage/src/lib.rs`

- [ ] Write a failing static test requiring an append-only `contextlab_schema_migration_ledger`, `migration_id`, SHA-256 digest, applied timestamp, and explicit separation from privileged artifacts.
- [ ] Add the regular ledger migration to `CONTEXT_PLATFORM_MIGRATION`; it records no prior migrations retroactively and contains no `SECURITY DEFINER` or role DDL.
- [ ] Verify the static test passes and existing base migration tests remain green.

### Task 2: Rehearse an ordered upgrade / 演练有序升级

**Files:**
- Create: `scripts/verify-production-migration-rehearsal.sh`
- Modify: `crates/storage/src/postgres.rs`
- Test: `crates/storage/src/postgres.rs`

- [ ] Write an ignored PostgreSQL test that starts from `0001` through `0007`, inserts historical membership/audit data, applies `0008` then `0009`, provisions the separate purge roles/procedure, and asserts legacy rows, retention defaults, role restrictions, and ledger immutability.
- [ ] Implement a loopback-only script requiring a distinct `CONTEXTLAB_REHEARSAL_DATABASE_URL`, name, dedicated role, and explicit reset opt-in. It applies the same ordered rehearsal without reading `.env` or a default URL.
- [ ] Prove repeat runs reject duplicated ledger records rather than silently altering history; the privileged operator artifact may be idempotent but must not be folded into the regular ledger.

### Task 3: CI and bilingual boundaries / CI 与双语边界

**Files:**
- Modify: `.github/workflows/verify.yml`
- Modify: `scripts/verify-disposable-postgres-storage.test.sh`
- Modify: `README.md`, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/long-term-roadmap.md`, `docs/roadmap/completion-criteria.md`, `docs/roadmap/goal-governance.md`

- [ ] Run the rehearsal against a separate PostgreSQL service database in CI after disposable storage tests.
- [ ] Document that this is production-like forward-upgrade evidence only, not a production deployment, rollback proof, or public-write approval.
- [ ] Record fresh local and remote verification evidence; leave public write readiness open until all fixed gates are satisfied.
