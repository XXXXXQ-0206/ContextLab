# Private Versioned ContextGraph Diff Adapter / 私有版本化 ContextGraph Diff 适配器

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Criteria 1, 2, and 4: a Context commit history must remain
replayable and reviewable, version-backed graph comparison must be owned by reusable Rust
application contracts, and `GraphDiff::between` must remain the sole graph-diff calculator.

**Unmet dependency or evidence gap / 未满足依赖或证据缺口：** The protected-local Axum handler
currently resolves two exact `CommitGraphSnapshot` records and calls `GraphDiff::between`
directly. The repository already owns immutable snapshot storage and `diff-engine` already owns
three-way graph review, but no two-way versioned graph application contract or storage adapter
connects those boundaries. This leaves scope validation and comparison orchestration in the
transport layer and makes the exact-commit read path harder to reuse and test independently.

**Why now / 为什么现在优先：** The immutable commit snapshot repository, replay-state contract,
graph consistency validation, versioned diff contracts, protected read gate, and Web read adapter
already exist. A small internal adapter is the nearest dependency-ready convergence step before
any future Context editing or merge writer; it closes an architecture gap without expanding the
public surface.

**Non-goals / 明确非目标：** No new REST/OpenAPI/SDK/Web route or mutation, no public write,
branch mutation, merge/rollback writer, provider call, PostgreSQL runtime claim, operator
transport, release, production promotion, or second diff implementation. The adapter must call
the existing `GraphDiff::between` exactly once for the two supplied graphs.

**Smallest boundary and bilingual documentation / 最小边界与双语文档：** Add one pure
`contextlab-diff-engine` V1 request/projection/service, one private
`contextlab-storage` exact-snapshot read adapter with Memory/PostgreSQL repository parity, and
replace only the handler's comparison orchestration with that adapter. Update this plan,
`active-long-term-goal.md`, `completion-criteria.md`, and the architecture boundary note in both
English and Chinese. Keep the existing response DTO and route path byte-compatible.

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Focused diff-engine
and storage tests; focused protected API graph-diff tests; `cargo fmt --all -- --check`,
`cargo test --workspace --quiet --no-fail-fast`, strict offline workspace Clippy, locked Rust
1.85 check, `pnpm check:web`, and a static singularity check proving one `GraphDiff`
implementation/call path. PostgreSQL/Docker runtime, authenticated browser, Git, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`.

## Tasks / 任务

- [x] Add the versioned two-way ContextGraph diff contract and focused fail-closed tests.
- [x] Add the private storage adapter over exact immutable commit snapshots and focused tests.
- [x] Adapt the protected-local API handler without changing its route or response contract.
- [x] Update bilingual roadmap, completion audit, and architecture boundary documentation.
- [x] Run fresh local verification and record passed/ignored/unobserved/deferred evidence.

## Fresh Verification Record / 新鲜验证记录

- Focused diff-engine `3 passed`, storage `3 passed`, protected API graph-diff `6 passed`, shared
  adapter `8 passed`, CLI `5 passed`, and Desktop staging `5 passed`.
- `cargo fmt --all -- --check`, `cargo test --workspace --quiet --no-fail-fast` (storage `202
  passed, 39 ignored`, API `189`, versioning `45`), full offline strict Clippy, and locked Rust
  `1.85.0` check passed.
- `pnpm check:web` passed public SDK `15`, local SDK `99`, Web `207`, TypeScript/lint, and the
  local production build. Static `GRAPH_DIFF_IMPL_COUNT=1`.
- PostgreSQL/Docker runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
  release, and production remain `unobserved` or `deferred`; no secret was read and no public
  write surface changed.

- focused diff-engine `3 passed`、storage `3 passed`、protected API graph-diff `6 passed`、shared
  adapter `8 passed`、CLI `5 passed` 与 Desktop staging `5 passed`。
- `cargo fmt --all -- --check`、`cargo test --workspace --quiet --no-fail-fast`（storage `202
  passed, 39 ignored`、API `189`、versioning `45`）、完整 offline strict Clippy 与锁定 Rust
  `1.85.0` check 均通过。
- `pnpm check:web` 通过 public SDK `15`、local SDK `99`、Web `207`、TypeScript/lint 与本地
  production build。静态 `GRAPH_DIFF_IMPL_COUNT=1`。
- PostgreSQL/Docker runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、
  release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secret，也未改变 public write surface。
