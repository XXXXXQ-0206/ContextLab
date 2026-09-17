# PostgreSQL Component-Content Revision Evidence Plan / PostgreSQL Component 正文修订证据计划

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Completion criterion 1 requires persistent Context-first coverage, and criterion 2 requires replayable versioning backed by tested reusable contracts.

**Unmet gap / 未满足缺口：** The private component-content revision contract has focused in-memory coverage, but its PostgreSQL transaction path has not yet been exercised against migration `0012`. The existing disposable storage gate therefore cannot prove that commit insertion, immutable revision insertion, component hash projection, and idempotent replay remain atomic in PostgreSQL.

**Why now / 为什么现在做：** The domain model, guarded writer, migration, seed component, disposable PostgreSQL harness, and per-test schema reset are already present. This is the closest dependency-ready evidence gap after private revision implementation, and is narrower than public Context editing, benchmark workflows, or release promotion.

**Non-goals / 非目标：** No public component body read or write route, OpenAPI operation, SDK method, Web mutation control, operator transport, new migration, GraphDiff behavior, or public protected-write promotion.

**Affected boundary and documentation / 受影响边界与文档：** Add one ignored PostgreSQL storage integration test and include it in the existing disposable CI selection. Update the active-goal, roadmap, completion audit, REST, persistence, and SDK documentation to say body storage is private and commit-bound rather than absent.

**Fresh verification / 新鲜验证：** Run the exact ignored PostgreSQL test through `scripts/verify-disposable-postgres-storage.sh` with its loopback database guard; then run formatting, workspace Rust tests, Web checks, artifact-shape checks, and public-surface searches. Remote CI and operator-approved rehearsal receipts remain independent `unobserved` release evidence.

## Tasks / 任务

- [x] Add a PostgreSQL create/replay/read/projection regression for one existing seeded component.
- [x] Register the exact test in the disposable PostgreSQL script.
- [x] Correct bilingual documentation that still says component body storage is future work.
- [x] Run the exact guarded disposable database test; broad local regressions are recorded with this increment.
