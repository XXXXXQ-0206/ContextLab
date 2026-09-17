# Rehearsal Evidence Manifest Implementation Plan / 演练证据清单实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标:** Retain a deterministic, secret-free manifest of every regular migration and privileged SQL asset after a successful CI production-like rehearsal, so a future operator-approved receipt can review real asset digests.

**Architecture / 架构:** A standalone Bash writer reads only checked-in SQL and the rehearsal runner, emits a line-oriented manifest plus a SHA-256 in a success-only CI artifact, and has no database or runtime dependencies. The artifact is technical input only: it cannot represent operator approval, isolated environment identity, actual executor identity, cleanup proof, a production migration, or `observed_pass`.

**Tech Stack / 技术栈:** Bash, `sha256sum`, GitHub Actions, Markdown.

---

## Necessity Record / 必要性记录

**Completion condition and charter principle / 服务的完成条件与宪章原则:** This increment directly supplies the migration-asset integrity input named in the `Production security and collaboration` and `Reliable release gates` criteria. It supports reproducible, replayable ContextLab storage evolution without widening the public product surface.

**Unmet dependency, risk, and evidence gap / 未满足的依赖、风险与证据缺口:** The protocol requires ID/SHA-256 entries for every regular migration and separately ordered privileged SQL artifact. The current immutable ledger only proves append-only behavior and the remote CI artifact only proves checkout/workflow identity; neither retains a reviewable manifest of the assets exercised by the production-like rehearsal.

**Why this is next / 为何此时优先:** This is the remaining dependency-ready technical input for the operator-rehearsal receipt. Operator approval, non-production isolation, and environment-level execution evidence remain externally controlled; graph work, public writes, editing, benchmarks, plugins, and UI expansion are downstream.

**Explicit non-goals / 明确非目标:** No database connection, `psql`, `cargo test`, production/replica/tunnel access, change approval, environment identity capture, public REST/OpenAPI/SDK/Web change, operator transport, migration rewrite, runtime behavior change, or graph-diff calculator. `GraphDiff` remains the sole graph-diff calculator.

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Create a manifest writer and dynamic Bash test, add a CI static layout guard plus success-only artifact upload, and update the evidence protocol, governance ledger, completion criteria, README, and this plan in English and Chinese.

**Fresh verification required before another increment / 下一增量前必须取得的新鲜验证:** Watch the dynamic writer test fail before the writer exists, then pass after implementation. Run shell syntax and static CI tests, both existing database guard tests, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, and scope searches. A remote artifact and an independently reviewed operator receipt remain open.

### Task 1: Define the failing manifest contract / 定义失败的清单契约

**Files:**
- Create: `scripts/write-production-rehearsal-evidence.test.sh`
- Test: `scripts/write-production-rehearsal-evidence.test.sh`

- [x] **Step 1: Write the dynamic failing test**

Create a Bash test that invokes `scripts/write-production-rehearsal-evidence.sh` with a temporary output file and requires these records:

```bash
schema_version=production-rehearsal-evidence-capture.v1
capture_type=technical-input-only
regular_migration=crates/storage/migrations/<file> sha256=<digest>
privileged_artifact=crates/storage/privileged/<file> sha256=<digest>
rehearsal_script=scripts/verify-production-migration-rehearsal.sh sha256=<digest>
capture_scope=asset-integrity-only
```

The test derives every expected SQL path and digest from the checked-in directories, requires lexical ordering and exact counts, and rejects `postgres://`, `password`, `DATABASE_URL`, `CONTEXTLAB_`, `psql`, and `cargo test` in the output or writer implementation.

- [x] **Step 2: Run the test and verify red**

Run:

```bash
bash scripts/write-production-rehearsal-evidence.test.sh
```

Expected: FAIL because the manifest writer does not exist.

### Task 2: Write the deterministic technical manifest / 写入确定性的技术清单

**Files:**
- Create: `scripts/write-production-rehearsal-evidence.sh`

- [x] **Step 1: Implement the writer without runtime side effects**

Accept one output path, locate the repository root from the script path, and write atomically through a sibling temporary file. Sort `crates/storage/migrations/*.sql` and `crates/storage/privileged/*.sql` lexically; write each relative path with its SHA-256; write the rehearsal runner SHA-256 and safe fixed metadata. Fail before writing a partial manifest when either asset group is empty.

- [x] **Step 2: Run the dynamic test and verify green**

Run:

```bash
bash scripts/write-production-rehearsal-evidence.test.sh
bash -n scripts/write-production-rehearsal-evidence.sh
```

Expected: PASS with every checked-in regular migration and privileged SQL asset recorded once, in lexical order, with matching digests.

### Task 3: Retain the technical input in successful CI / 在成功 CI 中保留技术输入

**Files:**
- Create: `scripts/verify-rehearsal-evidence-artifact.test.sh`
- Modify: `.github/workflows/verify.yml`

- [x] **Step 1: Write a failing static workflow test**

Require the writer test before database verification, a `Capture rehearsal technical evidence` step after `pnpm check:web`, and a success-only `Upload rehearsal technical evidence` step using `actions/upload-artifact@v4`, artifact name `contextlab-rehearsal-evidence`, `.rehearsal-evidence/` path, 90-day retention, and `if-no-files-found: error`. The capture block must invoke only the writer and `sha256sum`, and must reject database/credential markers.

- [x] **Step 2: Add the success-only capture and upload steps**

Run the writer only after all verify checks pass, create a SHA-256 for `production-rehearsal-evidence.txt`, and upload only `.rehearsal-evidence/`. Do not add environment variables to either capture or upload step.

### Task 4: Document the non-claim boundary / 文档化非声明边界

**Files:**
- Modify: `docs/roadmap/external-release-evidence-protocol.md`
- Modify: `docs/roadmap/goal-governance.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `README.md`

- [x] **Step 1: State the artifact's limited role in both languages**

Describe it as a migration-asset integrity input only. Explicitly exclude operator approval, environment isolation, actual privileged execution, cleanup proof, final state, production migration, rollback/DR, and public-write approval.

- [x] **Step 2: Record the root cause and regression proof**

Record that the preceding ledger and CI capture lacked real migration/privileged-asset digests. State that dynamic asset-manifest assertions, static workflow layout checks, and fresh regression commands prove the local repair only.

### Task 5: Verify the complete boundary / 验证完整边界

**Files:**
- Test: `scripts/write-production-rehearsal-evidence.test.sh`
- Test: `scripts/verify-rehearsal-evidence-artifact.test.sh`

- [x] **Step 1: Run focused script checks**

```bash
bash scripts/write-production-rehearsal-evidence.test.sh
bash scripts/verify-rehearsal-evidence-artifact.test.sh
bash -n scripts/write-production-rehearsal-evidence.sh
bash -n scripts/write-production-rehearsal-evidence.test.sh
bash -n scripts/verify-rehearsal-evidence-artifact.test.sh
```

Expected: PASS without a database, network, secrets, or production target.

- [x] **Step 2: Run existing release gates**

```bash
bash scripts/verify-disposable-postgres-storage.test.sh
bash scripts/verify-production-migration-rehearsal.test.sh
cargo fmt --all -- --check
cargo test --workspace
pnpm check:web
```

Expected: PASS; no local command creates an operator-approved receipt.

- [x] **Step 3: Verify scope and plan completeness**

```bash
rg -n "contextlab-rehearsal-evidence|production-rehearsal-evidence" .github scripts docs/roadmap README.md
rg -n "contextlab-rehearsal-evidence|production-rehearsal-evidence" server apps packages
```

Expected: capture references occur only in CI, scripts, and bilingual evidence documentation; no runtime/public-surface references occur.
