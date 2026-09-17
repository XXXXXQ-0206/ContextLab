# External Release Evidence Implementation Plan / 外部发布证据实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标:** Define one bilingual, redacted receipt protocol for the two external release gates that remain before public-write readiness can be decided: remote disposable CI and an operator-approved production-change rehearsal.

**Architecture / 架构:** Keep the evidence model in roadmap documentation, with `docs/roadmap/goal-governance.md` as the authoritative status ledger and a dedicated protocol defining receipt fields, binding rules, and decision semantics. The protocol accepts external evidence but never fabricates it; it is intentionally separate from runtime code, the public REST/SDK contract, and the private `GraphDiff` boundary.

**Tech Stack / 技术栈:** Markdown, GitHub Actions metadata, PostgreSQL migration asset digests, operator-controlled change records, PowerShell/Ripgrep documentation checks.

---

## Necessity Record / 必要性记录

**Completion condition and charter principle / 服务的完成条件与宪章原则:** This increment serves the named `Reliable release gates` and `Production security and collaboration` conditions in `docs/roadmap/completion-criteria.md`. It protects reproducibility, secure-by-default operation, stable contracts, and bilingual documentation without treating a local test or an external observation as project completion.

**Unmet dependency, risk, and evidence gap / 未满足的依赖、风险与证据缺口:** Local PostgreSQL 16.14 disposable tests, the isolated purge-role contract, and a production-like forward rehearsal have fresh local evidence, but no auditable remote GitHub Actions receipt or operator-approved production-change rehearsal receipt exists. The local workspace cannot establish an exact remote binding because Git metadata is unavailable. The current rehearsal ledger proves append-only behavior only; it does not prove that real migration assets were executed or hashed.

**Why this is next / 为何此时优先:** Under convergence-first governance, external evidence is the remaining dependency-ready release-gate work. Public writes, editing, benchmarks, plugins, and UI expansion are downstream and therefore inadmissible now.

**Explicit non-goals / 明确非目标:** No production deployment, database connection, SSH tunnel, port-forwarded target, secret access, `.env` read, public write route, operator transport, migration change, OpenAPI/SDK method, Web mutation control, runtime limiter wiring, or new graph-diff implementation. `GraphDiff` remains the sole graph-diff calculator.

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Create this plan and `docs/roadmap/external-release-evidence-protocol.md`; update only `docs/roadmap/goal-governance.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and `docs/roadmap/long-term-roadmap.md` to point to the protocol and current gate state.

**Fresh verification required before another increment / 下一增量前必须取得的新鲜验证:** Validate protocol sections, current `unobserved` status, Git-unavailable binding rule, non-goals, and cross-document links with fresh local searches. A later public-write readiness decision additionally requires two independently reviewable, redacted receipts matching this protocol; an absent, failed, or unbound receipt keeps the gate open.

**Gate root-cause update / 门禁根因更新:** Review found that the disposable rehearsal script can reset a loopback PostgreSQL target but cannot prove the target is not an SSH tunnel or an incorrectly mapped shared instance, and that its ledger test does not hash real migration assets. The minimum repair in this documentation-only increment is to reject that script and ledger alone as operator-rehearsal evidence, require independent environment-isolation proof plus a migration-asset manifest in the external receipt, and prohibit production, replicas, tunnels, and shared clusters. The regression proof is the protocol and governance record containing those prohibitions and acceptance fields; no production-like run is claimed.

## File Structure / 文件结构

- Create: `docs/roadmap/external-release-evidence-protocol.md` — bilingual receipt schema, status semantics, review procedure, and decision boundary.
- Create: `docs/superpowers/plans/2026-07-14-external-release-evidence.md` — this Necessity Record and executable documentation plan.
- Modify: `docs/roadmap/goal-governance.md` — authoritative current receipt ledger and admission decision.
- Modify: `docs/roadmap/active-long-term-goal.md` — current gate and next increment.
- Modify: `docs/roadmap/completion-criteria.md` — dated audit update preserving historical local-test counts.
- Modify: `docs/roadmap/long-term-roadmap.md` — correct priority ordering and link to the protocol.

### Task 1: Define the redacted receipt contract / 定义去敏回执契约

**Files:**
- Create: `docs/roadmap/external-release-evidence-protocol.md`

- [x] **Step 1: Write the external evidence model**

Define `unobserved`, `observed_pass`, `observed_fail`, and `inconclusive`, and require an `evidence_id`, collection timestamp, custodian, and redacted immutable references for every receipt.

- [x] **Step 2: Specify remote CI receipt fields**

Require repository identity, workflow path and reviewed blob digest, run identifier/URL/attempt, event/ref, provider-reported head SHA, required jobs and steps, timestamps, runner/action/container identity, command outcome, immutable log/artifact references, and explicit remote-to-local binding state.

- [x] **Step 3: Specify operator rehearsal receipt fields**

Require an independent approval, a redacted isolated non-production environment proof, synthetic or de-identified baseline declaration, migration and privileged-SQL asset digest manifest, execution window, abort owner, least-privilege roles, invariant results, cleanup proof, and a non-claimable rollback state.

- [x] **Step 4: Record non-claims and rejection rules**

State that a green CI run is not a production deployment or public-write approval; a local Git-unavailable workspace cannot claim an exact remote binding; and the current disposable reset script/ledger cannot alone qualify as operator evidence.

### Task 2: Update the authoritative governance ledger / 更新权威治理账本

**Files:**
- Modify: `docs/roadmap/goal-governance.md`

- [x] **Step 1: Add a dated external-evidence decision entry**

Set both receipts to `unobserved`, set local-worktree binding to `unavailable`, link the protocol, preserve the prior local evidence history, and say that public-write readiness remains undecided.

- [x] **Step 2: Record the review-discovered safety boundary**

Record the loopback/tunnel limitation and ledger-integrity limitation, then tie the minimum documentation repair and its regression proof to the protocol rather than claiming an unperformed operator rehearsal.

### Task 3: Synchronize roadmap entry points / 同步路线图入口

**Files:**
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Replace stale next-gate language**

Describe the locally proven limiter, restricted purge executor, and production-like forward rehearsal as non-production prerequisites, then name receipt collection as the only admitted next increment.

- [x] **Step 2: Preserve historical evidence while adding current status**

Keep earlier 11/12/13/15-test records as dated history. Add a current 17-test note and point readers to the protocol for the two still-unobserved external gates.

- [x] **Step 3: Preserve product boundaries**

State consistently that the update changes no REST route, OpenAPI operation, SDK method, Web control, operator transport, database migration, or `GraphDiff` behavior.

### Task 4: Verify the documentation boundary / 验证文档边界

**Files:**
- Test: `docs/roadmap/external-release-evidence-protocol.md`
- Test: `docs/roadmap/goal-governance.md`
- Test: `docs/roadmap/active-long-term-goal.md`
- Test: `docs/roadmap/completion-criteria.md`
- Test: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Check required receipt and safety terms**

Run:

```powershell
rg -n "unobserved|remote_snapshot_binding|migration asset|SSH tunnel|GraphDiff" docs/roadmap
```

Expected: the protocol and governance ledger contain all current-state, binding, asset-integrity, isolation, and graph-diff boundary terms.

- [x] **Step 2: Check no public-surface work was introduced**

Run:

```powershell
rg -n "PostgresProtectedRouteRateLimiter|external-release-evidence" server apps packages .github docs/roadmap
```

Expected: the new protocol appears only in governance documentation; no new runtime, public API, SDK, Web, or workflow file references it as a product feature.

- [x] **Step 3: Check plan completeness**

Run:

```powershell
rg -n "TODO|TBD|implement later|fill in details" docs/superpowers/plans/2026-07-14-external-release-evidence.md | rg -v "rg -n"
```

Expected: no matches.

- [x] **Step 4: Record the result**

Record only the fresh local documentation-check outputs. Do not claim remote CI, operator approval, production deployment, rollback proof, or public-write readiness without receipts.
