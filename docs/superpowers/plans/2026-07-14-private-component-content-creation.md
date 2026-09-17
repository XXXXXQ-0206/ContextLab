# Private Component Content Creation Plan / 私有 Component 正文创建计划

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment advances Context-first persistent coverage and replayable versioning by completing the first lifecycle operation that the existing content-revision boundary deliberately excluded: creating a new component with its initial immutable body revision.

**Unmet gap / 未满足缺口：** The guarded writer can revise only an already seeded component. `AddedComponent` now binds an initial body hash, but a new component still needs replayable name/metadata, guarded graph validation, and atomic persistence with its commit, graph snapshot, branch-head advance, idempotency record, and immutable first revision.

**Why now / 为什么现在做：** The authenticated guarded transaction, `AddedComponent` semantic change, component projection table, immutable revision store, disposable PostgreSQL harness, and private revision read port are already established. This is the closest dependency-ready Context lifecycle gap; a public editor, REST/SDK write surface, or Web mutation control would bypass or duplicate this contract.

**Boundary / 边界：** Add one private creation command to the existing guarded writer. It validates a replayable `AddedComponent` descriptor with deterministic body hash, name, metadata, snapshot node, and taxonomy relationship; it then inserts the component and an initial revision with no previous hash atomically and replays idempotently. No public body read/write route, OpenAPI operation, SDK method, Web mutation control, operator transport, GraphDiff behavior, or release promotion is included.

**Fresh verification / 新鲜验证：** Add focused memory and disposable PostgreSQL create/replay/read/projection tests; run formatting, workspace Rust tests, Web checks, static disposable and evidence scripts, and searches proving no public write surface changed. Remote CI and operator rehearsal receipts remain independent unobserved release evidence.

**Verification root-cause addendum / 验证根因补充：** Independent review found three consistency gaps in the first implementation: an initial-content change without the replayable descriptor could bypass attachment validation, a commit could contain multiple component transitions while one attachment persisted only one state, and in-memory graph projections could retain the initial hash after a later body revision. It also found that `0013` needed database-level enforcement that a `NULL` prior is an initial revision. The minimal repair removes the incomplete initial-content constructor, validates exactly one component transition against exactly one private attachment after idempotency replay, overlays current hashes and accepted revision timestamps for seeded and dynamic components, and adds forward migration `0014` with a history check, partial unique index, trigger, and negative PostgreSQL coverage. These fixes remain within the private storage boundary and require fresh focused, disposable PostgreSQL, workspace, and Web regressions before this increment can close.

**验证根因补充：** 独立审阅发现首个实现存在三处一致性缺口：没有可回放 descriptor 的 initial-content change 可以绕过 attachment validation；一个 commit 可能包含多个 component transition，而单个 attachment 只持久化一个状态；in-memory graph projection 在后续 body revision 后仍可能保留初始 hash。审阅还发现 `0013` 需要数据库级约束，确保 `NULL` prior 只用于 initial revision。最小修复是移除不完整的 initial-content constructor，在 idempotency replay 后校验“恰好一个 component transition 对应恰好一个私有 attachment”，在 memory projection 中为 seeded 与 dynamic component 覆盖当前 hash 和已接受 revision timestamp，并增加带 history check、partial unique index、trigger 及 PostgreSQL negative coverage 的前向迁移 `0014`。这些修复保持在私有 storage boundary 内，并要求在当前增量关闭前重新取得 focused、disposable PostgreSQL、workspace 与 Web regression 证据。

## Tasks / 任务

- [x] Bind `AddedComponent` to an initial resulting content hash and replayable descriptor.
- [x] Add private creation command and nullable-prior immutable revision semantics.
- [x] Persist creation atomically through memory and PostgreSQL guarded writers.
- [x] Add exact disposable PostgreSQL evidence and script registration.
- [x] Update bilingual roadmap, persistence, REST, and SDK boundary documentation.
- [x] Run focused and broader verification.

## Fresh Verification Record / 新鲜验证记录

- `cargo fmt --all -- --check` and `cargo test --workspace` passed; the storage crate reported `140 passed, 21 ignored`, with the ignored database cases exercised separately below.
- The 21 named disposable PostgreSQL cases registered by `scripts/verify-disposable-postgres-storage.sh` each passed after an explicit reset of the verified loopback-only `contextlab_test` schema in the local `contextlab-postgres-disposable` container. This includes the creation/replay transaction, latest in-memory projection, `0012 -> 0014` upgrade, and malformed nullable-prior rejection cases.
- The current Windows session cannot execute the shell runner itself because its available native Bash is WSL (which does not inherit the Windows test environment) and Git Bash has no `psql`. `bash -n scripts/verify-disposable-postgres-storage.sh` and `scripts/verify-disposable-postgres-storage.test.sh` passed under Git Bash; the actual database cases were run with the same registered names, isolated reset semantics, and loopback container through PowerShell.
- `pnpm check:web` passed, including SDK and Web type checks/tests and the production Web build. A public-surface search found no component-body creation transport in `server/api`, checked-in OpenAPI, TypeScript SDK, Web, or `diff-engine`.

- `cargo fmt --all -- --check` 与 `cargo test --workspace` 均已通过；storage crate 报告 `140 passed, 21 ignored`，其中被忽略的数据库 case 已在下文单独执行。
- `scripts/verify-disposable-postgres-storage.sh` 登记的 21 个 disposable PostgreSQL case 均在本地 `contextlab-postgres-disposable` 容器中、已核验为 loopback-only 的 `contextlab_test` schema 每例显式 reset 后通过。其中包括 creation/replay transaction、最新 in-memory projection、`0012 -> 0014` 升级以及 malformed nullable-prior rejection。
- 当前 Windows 会话无法直接执行 shell runner：可用的 native Bash 是 WSL，不能继承 Windows test environment；Git Bash 则没有 `psql`。`bash -n scripts/verify-disposable-postgres-storage.sh` 与 `scripts/verify-disposable-postgres-storage.test.sh` 已在 Git Bash 下通过；实际数据库 case 则通过 PowerShell 按同一登记名称、隔离 reset 语义和 loopback 容器执行。
- `pnpm check:web` 已通过，其中包含 SDK/Web type check、test 与 production Web build。public-surface search 未在 `server/api`、checked-in OpenAPI、TypeScript SDK、Web 或 `diff-engine` 中发现 component-body creation transport。
