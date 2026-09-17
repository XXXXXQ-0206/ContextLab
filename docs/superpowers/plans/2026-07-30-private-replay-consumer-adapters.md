# Private Replay Consumer Adapters / 私有回放消费者适配器

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Criteria 1 and 2: exact Context commit history must be
replayable through shared Rust contracts and inspectable by local developer tools without
duplicating domain logic.

**Unmet dependency or evidence gap / 未满足依赖或证据缺口：** `ReplayStateSnapshotV1` and the
private exact-commit storage replay port are implemented and tested, but the CLI and Desktop
staging adapters stop at capability availability. There is no typed read-only projection that
lets either local shell consume an exact Context/commit replay state.

**Why now / 为什么现在优先：** The reusable versioning envelope and storage read boundary are
already available, so this is a dependency-ready consumer that directly increases replayability
evidence. It is independent of the protected graph-diff adapter and can proceed in parallel with
disjoint ownership.

**Non-goals / 明确非目标：** No replay implementation in CLI/Desktop, no storage transport, no
Tauri runtime/UI, no mutation, no public REST/OpenAPI/SDK route, no provider call, no secret, and
no production or release claim.

**Smallest boundary and bilingual documentation / 最小边界与双语文档：** Extend the shared
CLI/Desktop adapter contract with a typed immutable ReplayState projection, add one CLI inspect
path and one Desktop staging-shell presenter method, and add parser/order/schema-drift tests.
Business logic remains in the existing Rust core.

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Focused adapter,
CLI, and Desktop tests; format; strict package Clippy; and workspace/Web checks as integration
allows. Runtime Tauri, authenticated browser, PostgreSQL/Docker, Git, remote CI, operator,
release, and production evidence remain `unobserved` or `deferred`.

## Tasks / 任务

- [x] Add the typed ReplayState projection and fail-closed parser to the shared adapter contract.
- [x] Add CLI read-only inspection and Desktop staging presenter adaptation.
- [x] Add focused deterministic ordering and schema-drift tests.
- [x] Record fresh local evidence and bilingual roadmap status.

## Fresh Verification Record / 新鲜验证记录

- Shared adapter tests passed `8`, CLI command-path tests passed `5`, Desktop staging-shell tests
  passed `5`; owned-package strict offline Clippy and formatting passed.
- Workspace Rust, locked Rust `1.85.0`, full strict Clippy, and `pnpm check:web` also passed in the
  parallel integration wave. The consumer remains read-only and local.
- PostgreSQL/Docker runtime, Tauri runtime, authenticated browser, Git, remote CI, operator,
  release, and production remain `unobserved` or `deferred`; no secret or transport was added.

- shared adapter test `8` 项通过、CLI command-path test `5` 项通过、Desktop staging-shell test `5` 项通过；
  owned-package strict offline Clippy 与格式检查通过。
- workspace Rust、锁定 Rust `1.85.0`、完整 strict Clippy 与 `pnpm check:web` 也在并行集成波次中通过；
  该 consumer 仍是 local read-only。
- PostgreSQL/Docker runtime、Tauri runtime、authenticated browser、Git、remote CI、operator、release 与
  production 继续为 `unobserved` 或 `deferred`；未新增 secret 或 transport。
