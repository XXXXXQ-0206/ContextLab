# Private Context Metadata Semantic Diff / 私有 Context Metadata Semantic Diff

**Status / 状态:** admitted for the next local increment / 已准入下一项本地增量

## Necessity Record / 必要性记录

### Completion criterion and charter principle / 完成条件与章程原则

This increment directly serves Criteria 1 and 2: Context is the primary abstraction, and a
versioned Context must be semantically comparable at exact commits. The existing lifecycle now
replays Context metadata at an exact commit, but the reusable diff snapshot carries only graph and
semantic documents. A metadata-only commit can therefore produce an empty semantic diff.

本增量直接服务条件 1 与 2：Context 是首要抽象，versioned Context 必须能在精确 commit 上进行 semantic comparison。现有 lifecycle
已可在精确 commit 回放 Context metadata，但可复用 diff snapshot 目前只有 graph 与 semantic documents，因此 metadata-only commit 仍可能
产生空的 semantic diff。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

`ContextMetadata` and exact replay are available, and private persisted diff review already owns
the version-bound snapshot boundary. The missing contract is a deterministic, schema-validated
metadata value in `SemanticSnapshotV1`, plus a storage projection that binds it to the exact
replayed commit. Without this, Context-first changes are invisible to semantic review. The risk is
schema drift or accidental policy calculation in Web; the reusable Rust diff domain must remain the
sole semantic calculator.

`ContextMetadata` 与 exact replay 已可用，private persisted diff review 也已有 version-bound snapshot boundary。缺失 contract 是在
`SemanticSnapshotV1` 中加入确定性、schema-validated 的 metadata value，并由 storage 将其绑定到 exact replayed commit。否则 Context-first
变化会在 semantic review 中不可见。风险是 schema drift 或 Web 意外计算 policy；可复用 Rust diff domain 必须继续是唯一 semantic calculator。

### Why now / 为什么现在优先

The lifecycle closure and its Web fixture regression are freshly green, so the exact metadata
source of truth is available without adding transport or UI. This is the smallest remaining local
gap that directly connects the newly replayable Context metadata to the already-tested version-backed
diff workflow. It is more necessary than another presentation surface or external-release audit.

lifecycle closure 及其 Web fixture regression 已新鲜变绿，因此无需新增 transport 或 UI 即可使用 exact metadata source of truth。这是当前最小
且直接把新可回放 Context metadata 接入已有 version-backed diff workflow 的本地缺口；其必要性高于再加 presentation surface 或重复外部
release audit。

### Minimal affected boundary / 最小受影响边界

- `crates/diff-engine/src/contract.rs` and focused contract/application tests: metadata field,
  deterministic diff transition, schema validation, serialization, and red/green proof.
- Existing private storage diff snapshot construction and focused tests only if the current
  adapter must pass replayed metadata into the diff contract.
- This plan and the bilingual roadmap receipt after fresh verification.

- `crates/diff-engine/src/contract.rs` 与 focused contract/application tests：metadata field、确定性 diff transition、schema validation、
  serialization 与 red/green proof。
- 仅在当前 adapter 必须将 replayed metadata 传入 diff contract 时，修改既有 private storage diff snapshot construction 与 focused tests。
- 本计划与新鲜验证后的双语 roadmap receipt。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK change, public write promotion, operator transport, or Web,
  CLI, or Desktop mutation.
- No new metadata table, migration, provider/evaluator call, benchmark execution, merge/rollback,
  branch mutation, or external evidence work.
- No policy/threshold calculation in Web or SDK, no raw secret/private payload exposure, and no
  second `GraphDiff` implementation. `GraphDiff::between` remains the sole graph-diff calculator.

- 不新增 public REST/OpenAPI/public SDK change、public write promotion、operator transport 或 Web、CLI、Desktop mutation。
- 不新增 metadata table、migration、provider/evaluator call、benchmark execution、merge/rollback、branch mutation 或 external evidence work。
- Web 或 SDK 不计算 policy/threshold，不暴露 raw secret/private payload，不新增第二个 `GraphDiff` implementation；`GraphDiff::between` 仍是唯一
  graph-diff calculator。

### Fresh verification required before the next increment / 下一增量前的新鲜验证

First observe a red focused test proving that a metadata-only semantic change is absent from the
old contract. Then require green focused diff-engine and storage tests, `cargo fmt --all --
--check`, `cargo test --workspace --quiet --no-fail-fast`, strict offline workspace Clippy,
locked Rust `1.85.0`, `pnpm check:web`, and static `impl GraphDiff` plus public-surface checks.
PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git change-set, remote CI, operator
rehearsal, release, production, and public promotion remain `unobserved` or `deferred`.

首先观察 focused red test，证明旧 contract 看不见 metadata-only semantic change；随后必须取得 diff-engine 与 storage focused green tests、
`cargo fmt --all -- --check`、`cargo test --workspace --quiet --no-fail-fast`、strict offline workspace Clippy、锁定 Rust `1.85.0`、
`pnpm check:web`，以及 `impl GraphDiff` 唯一性和 public-surface checks。PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git
change-set、remote CI、operator rehearsal、release、production 与 public promotion 继续为 `unobserved` 或 `deferred`。

## Ownership / 所有权

- Luna diff worker: only `crates/diff-engine/src/contract.rs`, its application adapter if strictly
  required, and focused diff-engine tests.
- Integration Lead: existing storage snapshot adapter, cross-crate integration tests, roadmap
  receipts, and final verification.

- Luna diff worker：仅负责 `crates/diff-engine/src/contract.rs`、严格必要的 application adapter 与 diff-engine focused tests。
- Integration Lead：负责既有 storage snapshot adapter、跨 crate integration tests、roadmap receipt 与最终验证。

All workers use `gpt-5.6-luna`, do not read secrets or external systems, do not revert unrelated
work, and must report ownership conflicts before editing.

所有 worker 使用 `gpt-5.6-luna`，不得读取 secret 或访问 external system，不得回退无关改动，发现 ownership conflict 必须先报告。

## Current Documentation/QA Receipt / 当前文档与 QA 回执

Historical 2026-07-30 snapshot / 2026-07-30 历史快照：`in_progress / blocked by a source-contract mismatch` / `in_progress / 被源码契约形状不匹配阻塞`。Workspace inspection recorded that
`crates/diff-engine/tests/metadata_semantic_diff.rs` requires the exported enum
`ContextMetadataChangeV1::Modified { original, revised }`, while the current implementation
still exposed `SemanticMetadataChangeV1` as a struct and constructed that shape from the
application layer. The metadata semantic-diff focused test was therefore not green at that time;
this historical snapshot does not describe the current focused receipt.

`in_progress / blocked by a source-contract mismatch` / `in_progress / 被源码契约形状不匹配阻塞`。工作区检查显示，
`crates/diff-engine/tests/metadata_semantic_diff.rs` 要求导出的枚举为
`ContextMetadataChangeV1::Modified { original, revised }`，而当前实现仍暴露结构体
`SemanticMetadataChangeV1`，application layer 也仍按该形状构造。因此 metadata semantic-diff focused test 尚未变绿；本计划不声称
Rust、storage、SDK、Web 或全 workspace 增量已完成。

This docs/QA pass changes only this plan and the bounded roadmap receipts. It does not edit the
diff engine, storage, SDK, Web, or tests; it does not read secrets or external systems. The next
implementation owner must first reconcile the enum contract, then observe the required red/green
focused proof before any broader verification is recorded. `GraphDiff::between` remains the sole
graph-diff calculator, and the local/private boundary remains unchanged.

本次 docs/QA pass 只修改本计划与有界路线图回执，不修改 diff engine、storage、SDK、Web 或 tests；不读取 secret 或访问 external system。下一次
implementation 必须先统一 enum contract，再观察所需的 focused red/green proof，之后才能记录更广泛验证。`GraphDiff::between` 仍是唯一
graph-diff calculator，local/private 边界不变。

## 2026-07-31 Current Receipt and Next Necessity Record / 2026-07-31 当前回执与下一项必要性记录

### Current status / 当前状态

`completed / verified locally` for this bounded private metadata semantic-diff implementation slice;
the historical 2026-07-30 source-contract mismatch above is superseded by the current code and is
retained only as an audit snapshot. The long-term goal and every broader completion criterion remain
open.

本次有界 private metadata semantic-diff implementation slice 标记为 `completed / verified locally`；上方 2026-07-30
source-contract mismatch 仅作为审计历史保留，已由当前源码状态取代。长期目标与所有更广泛收束条件仍开放。

### Fresh local verification / 新鲜本地验证

- `cargo test -p contextlab-diff-engine --test metadata_semantic_diff --quiet`: `4 passed`.
- `cargo test -p contextlab-storage --test context_diff_review --quiet`: `7 passed`.
- `cargo test --workspace --quiet --no-fail-fast`: passed; `40 ignored`.
- `cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --locked --offline`: passed.
- `cargo fmt --all -- --check`: passed.
- `pnpm check:web`: public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build passed.
- Static audit: production `impl GraphDiff` count is `1`; no public write surface was added.

- `cargo test -p contextlab-diff-engine --test metadata_semantic_diff --quiet`：`4 passed`。
- `cargo test -p contextlab-storage --test context_diff_review --quiet`：`7 passed`。
- `cargo test --workspace --quiet --no-fail-fast`：通过；观察到 `40 ignored`。
- strict offline workspace Clippy、锁定 Rust `1.85.0` check 与 `cargo fmt --all -- --check`：通过。
- `pnpm check:web`：public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build 通过。
- 静态审计：production `impl GraphDiff` 数量为 `1`；未新增 public write surface。

### Evidence boundary / 证据边界

The current contract carries optional `SemanticSnapshotV1` Context metadata and produces typed
`ContextMetadataChangeV1` `added`/`removed`/`modified` transitions. The storage receipt currently
proves the modified exact-commit replay path. Rust enum-variant unknown-field rejection and storage
exact-commit replay receipts for added and removed metadata remain unobserved; they are the next
bounded local gap. PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git change-set,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

当前 contract 已携带可选的 `SemanticSnapshotV1` Context metadata，并生成 typed `ContextMetadataChangeV1`
`added`/`removed`/`modified` transition。storage 回执当前证明 modified exact-commit replay；Rust enum variant unknown-field rejection
以及 added/removed metadata 的 storage exact-commit replay 回执仍未观测，是下一项有界本地缺口。PostgreSQL/Docker runtime、authenticated
browser/visual smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

### Next necessity record / 下一项必要性记录

- **Criterion/principle served / 服务条件与原则：** Criteria 1 and 2, Context-first exact replay, strict DTO boundaries, and the sole `GraphDiff::between` calculator. / 条件 1、2，Context-first 精确回放、严格 DTO 边界与唯一 `GraphDiff::between` calculator。
- **Gap and why now / 缺口与为何现在：** added/removed persistence evidence and Rust variant fail-closed proof are the smallest remaining evidence gaps in the already implemented semantic-diff path; they precede any new surface. / added/removed 持久化证据与 Rust variant fail-closed 证明是当前 semantic-diff path 最小的剩余证据缺口，应先于任何新表面。
- **Non-goals / 非目标：** no public REST/OpenAPI/SDK write, Web mutation, migration, provider, operator transport, secret access, PostgreSQL runtime claim, or second diff calculator. / 不新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、operator transport、secret access、PostgreSQL runtime 声明或第二个 diff calculator。
- **Minimal boundary / 最小边界：** `crates/diff-engine/tests` enum deserialization regression and existing storage review tests only; keep Rust core authoritative and preserve `data -> presenter -> screen`. / 仅修改 `crates/diff-engine/tests` enum 反序列化回归与既有 storage review tests；Rust core 保持权威并保留 `data -> presenter -> screen`。
- **Fresh verification before the next increment / 下一增量前的新鲜验证：** focused red/green tests for all three metadata transitions, focused storage replay, workspace tests, strict offline Clippy, MSRV check, fmt, `pnpm check:web`, and `impl GraphDiff=1` must be observed. / 必须新鲜观察三种 metadata transition 的 focused red/green、storage replay、workspace tests、strict offline Clippy、MSRV、fmt、`pnpm check:web` 与 `impl GraphDiff=1`。

The long-term goal remains active; this receipt is progress, not project completion. / 长期目标保持 active；本回执是进展，不是项目完成。

## 2026-07-31 Evidence Closure for Metadata Transitions / 2026-07-31 Metadata Transition 证据收束

The next bounded test increment is now `completed / verified locally`. The red phase was real: the
new variant test observed that serde accepted an unexpected field. The minimal production repair adds
`deny_unknown_fields` to `ContextMetadataChangeV1`; no transport or domain expansion was needed.

下一项有界测试增量现标记为 `completed / verified locally`。红阶段是真实观察到的：新增 variant test 发现 serde 接受 unexpected field。最小生产修复
是在 `ContextMetadataChangeV1` 上加入 `deny_unknown_fields`；没有扩大 transport 或 domain。

Fresh evidence / 新鲜证据：diff-engine metadata focused `5 passed`; storage exact-commit review `8 passed` including both
`added` and `removed`; workspace passed with storage `212 passed, 39 ignored`; strict offline Clippy, locked Rust `1.85.0` check,
fmt, `pnpm check:web` (`15/134/270 + production build`), and `impl GraphDiff=1` passed.

新鲜证据：diff-engine metadata focused `5 passed`；storage exact-commit review `8 passed`（包含 `added` 与 `removed`）；workspace 通过且 storage
为 `212 passed, 39 ignored`；strict offline Clippy、锁定 Rust `1.85.0` check、fmt、`pnpm check:web`（`15/134/270 + production build`）与唯一
`impl GraphDiff=1` 通过。

No public REST/OpenAPI/SDK write, Web mutation, migration, provider, operator transport, secret access, Docker/PostgreSQL runtime claim,
external receipt, release, production, or second diff calculator was added. The long-term goal remains active. / 未新增 public REST/OpenAPI/SDK write、
Web mutation、migration、provider、operator transport、secret access、Docker/PostgreSQL runtime 声明、external receipt、release、production 或第二个
diff calculator。长期目标保持 active。

The next admitted local increment requires a new bilingual Necessity Record and must be selected from a dependency-ready completion gap;
the completed metadata slice is not a stop condition. / 下一项本地增量必须先有新的双语 Necessity Record，并从依赖就绪的收束缺口中选择；已完成的 metadata slice
不是停止条件。
