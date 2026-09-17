# Private Benchmark Evidence Error Redaction / 私有 Benchmark Evidence 错误脱敏

## Necessity Record / 必要性记录

### Named criterion and charter principle / 命名条件与宪章原则

This bounded read-surface increment directly advances Criterion 6 (production security and
collaboration) and the charter's secure-by-default, secret-redaction, and fail-closed principles.
The private Benchmark evidence inspector must not render diagnostic text supplied by an upstream
BFF or protected API error response.

本有界 read-surface 增量直接推进条件 6（生产级安全与协作）以及宪章的 secure-by-default、secret-redaction 与 fail-closed 原则。
私有 Benchmark evidence inspector 不得渲染 upstream BFF 或 protected API error response 提供的诊断文本。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

`context-benchmark-evidence-data.ts` preserves a structured `body.message`, and the inspector
concatenates it into the user notice. A response carrying internal paths, database details, or
other private diagnostics can therefore cross the data-to-screen boundary. Existing tests cover
scope and safe payload shape but do not prove error-message redaction.

`context-benchmark-evidence-data.ts` 会保留结构化 `body.message`，inspector 又将其拼接到用户 notice。含有内部路径、数据库细节或其他
private diagnostics 的 response 因此可能穿过 data-to-screen 边界。现有测试覆盖 scope 与 safe payload shape，但没有证明 error-message 脱敏。

### Why now / 为何现在优先

The lifecycle redaction receipt is complete, while this adjacent Benchmark read boundary was
identified by fresh source inspection as an independent concrete leak. The change is dependency
ready, small, and security-relevant; it is more necessary than repeating already completed
multi-dataset breadth work or adding another read surface.

lifecycle 脱敏回执已完成，而本次新鲜源码检查确认相邻 Benchmark read boundary 仍存在具体泄漏。该修正依赖已满足、边界小且直接服务安全收束；
相比重复已完成的 multi-dataset breadth 或新增另一个 read surface，更有必要优先处理。

### Explicit non-goals / 明确非目标

- No benchmark dataset, suite, scorecard, threshold, evaluation diff, or GraphDiff algorithm change.
- No Rust, storage, migration, API, REST, OpenAPI, SDK, public write, Web mutation, provider,
  secret access, Docker/PostgreSQL runtime, browser, Git, remote CI, operator, release, or production claim.
- Preserve stable error codes, HTTP status, retry-after handling, exact scope validation,
  `credentials: "omit"`, `cache: "no-store"`, and the existing bilingual presenter messages.

- 不修改 benchmark dataset、suite、scorecard、threshold、evaluation diff 或 GraphDiff algorithm。
- 不修改 Rust、storage、migration、API、REST、OpenAPI、SDK、public write、Web mutation、provider、secret access、Docker/PostgreSQL runtime、
  browser、Git、remote CI、operator、release 或 production 声明。
- 保持 stable error code、HTTP status、retry-after、exact scope validation、`credentials: "omit"`、`cache: "no-store"` 与既有双语 presenter 文案。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

Only `apps/web/src/app/context-benchmark-evidence-data.ts`,
`context-benchmark-evidence-data.test.ts`, `context-benchmark-evidence-inspector.tsx`, and its
focused test are in scope. The data adapter retains only a safe local message; the inspector
continues to use the existing status/retry presenter and does not implement policy or algorithms.
This plan and the corresponding bilingual completion and parallel-ledger receipts are the
documentation boundary.

仅修改 `apps/web/src/app/context-benchmark-evidence-data.ts`、`context-benchmark-evidence-data.test.ts`、
`context-benchmark-evidence-inspector.tsx` 与其 focused test。data adapter 只保留安全本地 message；inspector 继续使用既有 status/retry presenter，
不承载 policy 或 algorithm。本计划及对应的双语 completion 与 parallel-ledger 回执构成文档边界。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe red tests with a private upstream diagnostic in both decision and run-details error
responses and in the inspector notice path. Then require focused data/inspector tests, `pnpm
check:web`, `cargo fmt --all -- --check`, offline workspace Rust tests, strict offline Clippy,
locked Rust 1.85 check, local contract verification, and the exact-one GraphDiff source check.

先观测 decision 与 run-details error response 以及 inspector notice path 中带 private upstream diagnostic 的红测。随后必须通过 focused data/inspector
tests、`pnpm check:web`、`cargo fmt --all -- --check`、offline workspace Rust tests、strict offline Clippy、锁定 Rust 1.85 check、local contract verification
与 exact-one GraphDiff source check。

## Status / 状态

`completed / verified locally`; this is not a project closeout. / `completed / verified locally`；这不是项目完成声明。

## Completion Receipt / 收束回执

The red phase observed three failures: both decision and run-details data errors retained private
upstream messages, and the inspector test had no safe presenter boundary. The green implementation
preserves the stable upstream error code, HTTP status, retry-after, exact scope, bearer-only
transport, `credentials: "omit"`, and `cache: "no-store"`, while replacing the error body message
with a bilingual local status message and making the inspector use only the existing status/retry
presenter. No benchmark policy or diff computation moved into the screen.

红阶段真实观察到三项失败：decision 与 run-details data error 都保留 private upstream message，inspector test 也没有安全 presenter boundary。
绿实现保留 stable upstream error code、HTTP status、retry-after、exact scope、bearer-only transport、`credentials: "omit"` 与 `cache: "no-store"`，
同时将 error body message 替换为双语本地 status 文案，并让 inspector 只使用既有 status/retry presenter。没有将 benchmark policy 或 diff calculation 移入 screen。

Fresh local verification / 新鲜本地验证:

- Red focused suite: `3 passed, 3 failed`; green data/inspector suite: `11 passed`.
- `pnpm check:web` passed: public SDK `15`, Web `310` tests, TypeScript/lint, local SDK checks, and production build.
- `cargo fmt --all -- --check` and offline workspace Rust passed: API `223 passed`; storage `238 passed, 41 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed.
- `pwsh -File tests/contract/verify-local-contracts.test.ps1` passed.
- Source inspection: `GRAPH_DIFF_IMPL_COUNT=1`; `GraphDiff::between` matched `10` Rust call sites.

新鲜本地验证：

- focused 红测 `3 passed, 3 failed`；data/inspector 绿测 `11 passed`。
- `pnpm check:web` 通过：public SDK `15`、Web `310` tests、TypeScript/lint、local SDK checks 与 production build。
- `cargo fmt --all -- --check` 与 offline workspace Rust 通过：API `223 passed`；storage `238 passed, 41 ignored`。
- strict offline Clippy 通过。
- 锁定 Rust `1.85.0` 的 `check --locked --offline` 通过。
- `pwsh -File tests/contract/verify-local-contracts.test.ps1` 通过。
- 源码检查：`GRAPH_DIFF_IMPL_COUNT=1`；Rust 中 `GraphDiff::between` 匹配 `10` 个。

No Rust/API/SDK/OpenAPI/public write, migration, provider, secret access, operator transport,
Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, or production claim was added. Those boundaries remain `ignored`, `unobserved`, or
`deferred`; the long-term goal remains `active`. The next increment is the independently audited
private ContextGraph review path snapshot-completeness record, and it requires its own bilingual
Necessity Record before implementation.

未新增 Rust/API/SDK/OpenAPI/public write、migration、provider、secret access、operator transport、Docker/PostgreSQL runtime、authenticated browser/visual smoke、
Git、remote CI、operator rehearsal、release 或 production 声明。上述边界继续为 `ignored`、`unobserved` 或 `deferred`；长期目标保持 `active`。
下一项是独立审查确认的 private ContextGraph review path snapshot completeness，实施前必须先有其独立双语 Necessity Record。
