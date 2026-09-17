# CI Evidence Artifact Implementation Plan / CI 证据工件实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标:** Make a successful remote `verify` job retain a compact, secret-free evidence capture artifact that proves the checked-out commit equals the provider SHA and fingerprints the executed workflow.

**Architecture / 架构:** Keep capture in GitHub Actions after every required verification step has succeeded. The artifact records only CI-visible, non-secret provenance and its own SHA-256; it is evidence input, not an `observed_pass` release receipt or public-write approval. A pure Bash static contract test prevents reordering, missing upload behavior, and secret-bearing capture fields without needing a database, network, or Git repository locally.

**Tech Stack / 技术栈:** GitHub Actions, Bash, POSIX `grep`/`sed`/`sha256sum`, Git, Markdown.

---

## Necessity Record / 必要性记录

**Completion condition and charter principle / 服务的完成条件与宪章原则:** This closes a concrete evidence-collection gap within `Reliable release gates` and `Production security and collaboration`: the CI workflow must retain the checked-out revision proof required for an independently reviewed `exact_commit` receipt. It advances reproducibility, security by default, and bilingual documentation without changing Context domain behavior.

**Unmet dependency, risk, and evidence gap / 未满足的依赖、风险与证据缺口:** `docs/roadmap/external-release-evidence-protocol.md` requires a provider `head_sha`, checked-out revision proof, their equality, workflow digest, and retained artifact reference. Current `.github/workflows/verify.yml` runs tests but emits none of these. The local `.git` directory is empty, so this worktree cannot substitute a commit identity; a future remote run must capture it at checkout time.

**Why this is next / 为何此时优先:** The remote CI receipt is the active, dependency-ready release gate. Capturing provenance at the source is strictly necessary before an external reviewer can form an `exact_commit` receipt. Public writes, graph work, editing, benchmarks, plugins, and UI expansion remain inadmissible.

**Explicit non-goals / 明确非目标:** No remote CI trigger, production connection, operator rehearsal, secret access, `.env` read, database URL or password capture, public REST/OpenAPI/SDK/Web change, operator transport, migration, runtime behavior, or second graph-diff calculator. `GraphDiff` remains the sole graph-diff calculator.

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Add one static workflow contract test under `scripts/`; modify `.github/workflows/verify.yml` only to invoke that test, capture the successful checkout/workflow evidence, and upload it; update the evidence protocol, governance ledger, completion criteria, and this plan in English and Chinese.

**Fresh verification required before another increment / 下一增量前必须取得的新鲜验证:** The static test must fail before workflow changes and pass after them. Run shell syntax checks, the existing disposable/rehearsal guard tests, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, and a public-surface search. A real remote run and operator-approved rehearsal receipt remain open until independently reviewed.

### Task 1: Specify the failing workflow contract / 定义失败的工作流契约

**Files:**
- Create: `scripts/verify-ci-evidence-artifact.test.sh`
- Test: `scripts/verify-ci-evidence-artifact.test.sh`

- [x] **Step 1: Write the failing static contract test**

Create a Bash test that reads `.github/workflows/verify.yml` and requires all of these fixed strings and ordering constraints:

```bash
require '      - run: bash scripts/verify-ci-evidence-artifact.test.sh'
require '      - name: Capture CI evidence'
require '        if: ${{ success() }}'
require '          checked_out_commit_sha="$(git rev-parse --verify HEAD^{commit})"'
require '          provider_head_sha="${GITHUB_SHA:?GITHUB_SHA is required}"'
require '          workflow_sha256="$(sha256sum "$workflow_path" | awk '\''{print $1}'\'')"'
require '      - name: Upload CI evidence'
require '        uses: actions/upload-artifact@v4'
require '          name: contextlab-ci-evidence'
require '          retention-days: 90'
require '          if-no-files-found: error'
```

The test also requires `pnpm check:web` and the static test to appear before capture, capture before upload, and rejects `postgres://`, `POSTGRES_PASSWORD`, `DATABASE_URL`, `password`, and `CONTEXTLAB_` inside the capture block.

- [x] **Step 2: Run the test and verify red**

Run:

```bash
bash scripts/verify-ci-evidence-artifact.test.sh
```

Expected: FAIL because `verify.yml` has no CI evidence capture step yet.

### Task 2: Capture only successful, non-secret provenance / 仅捕获成功且非敏感的来源信息

**Files:**
- Modify: `.github/workflows/verify.yml`

- [x] **Step 1: Invoke the static contract in CI**

Add `bash scripts/verify-ci-evidence-artifact.test.sh` after dependency setup and before the database verification steps so the workflow rejects an invalid evidence layout before executing the named release checks.

- [x] **Step 2: Add the success-only capture step**

After `pnpm check:web`, add a Bash step guarded by `if: ${{ success() }}`. It creates `.ci-evidence/verification-evidence.txt`, verifies `git rev-parse --verify HEAD^{commit}` equals `GITHUB_SHA`, fingerprints `.github/workflows/verify.yml`, writes schema version, provider, the two commit values, workflow path/digest, UTC capture time, safe scope text, and a redaction statement, then writes `verification-evidence.txt.sha256`.

- [x] **Step 3: Upload the capture artifact**

Add an `actions/upload-artifact@v4` step also guarded by `if: ${{ success() }}`. Upload only `.ci-evidence/` as `contextlab-ci-evidence`, retain it for 90 days, and set `if-no-files-found: error`.

### Task 3: Record the evidence boundary / 记录证据边界

**Files:**
- Modify: `docs/roadmap/external-release-evidence-protocol.md`
- Modify: `docs/roadmap/goal-governance.md`
- Modify: `docs/roadmap/completion-criteria.md`

- [x] **Step 1: Document the artifact as capture input, not release approval**

State in both languages that a successful CI run retains checkout equality, workflow digest, and a file digest for later review. It does not create an `observed_pass` receipt, prove production behavior, or approve public writes.

- [x] **Step 2: Record the root cause and remaining external gate**

Record that the prior workflow ran required checks without preserving checkout provenance. State that the static test and capture artifact are the minimum repair, while a fresh remote run plus independent review remains required.

### Task 4: Verify the complete boundary / 验证完整边界

**Files:**
- Test: `scripts/verify-ci-evidence-artifact.test.sh`
- Test: `.github/workflows/verify.yml`

- [x] **Step 1: Verify green workflow contract**

Run:

```bash
bash scripts/verify-ci-evidence-artifact.test.sh
bash -n scripts/verify-ci-evidence-artifact.test.sh
```

Expected: PASS with capture after all checks, upload after capture, equality proof, artifact digest, and no prohibited strings in the capture block.

- [x] **Step 2: Run existing guard and workspace checks**

Run:

```bash
bash scripts/verify-disposable-postgres-storage.test.sh
bash scripts/verify-production-migration-rehearsal.test.sh
cargo fmt --all -- --check
cargo test --workspace
pnpm check:web
```

Expected: PASS; none of these local results constitute a remote receipt.

- [x] **Step 3: Verify scope and documentation**

Run:

```bash
rg -n "contextlab-ci-evidence|verification-evidence|git rev-parse --verify HEAD" .github scripts docs/roadmap
rg -n "PostgresProtectedRouteRateLimiter|contextlab-ci-evidence" server apps packages
```

Expected: evidence capture appears only in CI, its static test, and bilingual governance documentation; no runtime, public API, SDK, Web, or GraphDiff surface changes.
