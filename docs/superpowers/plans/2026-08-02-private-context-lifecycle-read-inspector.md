# Private Context Lifecycle Read Inspector / 私有 Context 生命周期只读检查器

## Necessity Record / 必要性记录

### Criterion and charter principle / 对应条件与宪章原则

This increment directly advances Criterion 1, Context-first platform coverage, and the charter's
Context-first rule: a Context is the primary versioned unit, and its exact commit state must be
inspectable through shared domain and presentation boundaries.

本增量直接推进条件 1“以 Context 为核心的平台覆盖”，并落实项目宪章的 Context-first 原则：Context
是首要的版本化单元，且必须通过共享领域与呈现边界检查指定 commit 的精确状态。

### Gap and dependencies / 缺口与依赖

The protected lifecycle-state read path, local SDK parser, same-origin BFF, selected commit identity,
and lifecycle aggregate already exist. The Web workspace currently exposes lifecycle editing and
adjacent graph review, but it does not provide a dedicated read-only inspector that presents the
server-owned aggregate through `data -> presenter -> screen`.

受保护的 lifecycle-state 读取路径、local SDK parser、同源 BFF、selected commit identity 与 lifecycle aggregate
均已存在。当前 Web 工作台已有 lifecycle 编辑与相邻图谱审查，但缺少一个通过
`data -> presenter -> screen` 呈现 server-owned aggregate 的独立只读检查器。

### Why now / 为什么现在

The persistence and exact-commit read contracts are already locally verified, so this is the smallest
dependency-ready product increment that turns those facts into an inspectable Context workflow. It
has higher convergence value than adding another private storage field or starting a new subsystem.

持久化与 exact-commit 读取契约已有本地验证，因此这是当前依赖已满足、能把既有事实转化为可检查 Context
工作流的最小产品增量。相比继续增加私有存储字段或启动新子系统，它更直接服务收束条件。

### Minimal boundary / 最小受影响边界

- Web-only composition in the existing workspace.
- Exact Context and commit scope; no current-head inference.
- Reuse `context-lifecycle-data.ts`, the protected route/BFF, the local SDK parser, and shared UI primitives.
- Keep all business validation and lifecycle state semantics outside React components.

- 仅修改现有 workspace 的 Web 组合层。
- 保持精确 Context 与 commit scope；不推断 current head。
- 复用 `context-lifecycle-data.ts`、受保护 route/BFF、local SDK parser 与共享 UI primitive。
- 所有业务校验与 lifecycle state 语义继续留在 React 组件之外。

### Explicit non-goals / 明确非目标

No Rust/storage/migration changes, public REST/OpenAPI/public SDK write methods, Web mutation controls,
operator transport, Docker/PostgreSQL runtime, provider access, secrets, release, production, remote
CI, operator rehearsal, browser E2E, or visual-regression claim is part of this increment. `GraphDiff::between`
remains the sole graph-diff calculator.

本增量不包含 Rust/storage/migration 变更、public REST/OpenAPI/public SDK 写方法、Web mutation 控件、operator
transport、Docker/PostgreSQL runtime、provider 访问、secret、release、production、remote CI、operator rehearsal、
browser E2E 或 visual-regression 声明。`GraphDiff::between` 仍是唯一图差异计算器。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

Focused presenter and inspector tests must prove exact-scope rendering, loading/error/empty/available
states, state reset when the selected commit changes, bilingual labels, and accessible status semantics.
Then run `pnpm check:web` and the existing local contract checks. Results must preserve
`passed`/`ignored`/`unobserved`/`deferred` evidence boundaries; this increment advances Criterion 1 but
does not close it or the active long-term goal.

必须通过聚焦 presenter 与 inspector 测试，证明 exact-scope 呈现、loading/error/empty/available 状态、selected
commit 改变时的状态重置、双语标签与可访问性 status 语义。随后运行 `pnpm check:web` 与既有 local contract checks。
结果必须保留 `passed`/`ignored`/`unobserved`/`deferred` 证据边界；本增量仅推进条件 1，不关闭条件或 active 长期目标。

## Ownership / 文件归属

- Web worker: `apps/web/src/app/context-lifecycle-read-inspector.tsx`, its focused test, and the
  explicitly listed existing presenter/screen integration files.
- Integration lead: this record and final verification.
- PostgreSQL evidence worker: only the four benchmark breadth receipt documents listed in its task.

## Exit record / 退出记录

Completed locally. Focused presenter/inspector tests passed (`23 passed`); `pnpm --filter
@contextlab/web lint` passed; the full Web suite passed (`290 passed`); and `pnpm check:web` passed
with public SDK `15`, local SDK `135`, Web `290`, and the production build. Rust format, workspace
tests (`220 passed`, `41 ignored`), strict offline Clippy, and locked Rust `1.85.0` check also passed
after a test-only helper arity correction. No local read receipt is described as authenticated
browser, visual, remote, operator, release, or production evidence.

已在本地完成。聚焦 presenter/inspector 测试通过（`23 passed`）；`pnpm --filter @contextlab/web lint` 通过；
完整 Web suite 通过（`290 passed`）；`pnpm check:web` 通过，其中 public SDK `15`、local SDK `135`、Web `290`
与 production build 均通过。Rust format、workspace tests（`220 passed`、`41 ignored`）、strict offline Clippy 与
锁定 Rust `1.85.0` check 也在 test-only helper 参数修正后通过。没有把任何本地读取回执描述为 authenticated
browser、visual、remote、operator、release 或 production 证据。

上述退出记录基于已实际运行的新鲜命令。任何本地读取回执都不得描述为 authenticated browser、visual、remote、
operator、release 或 production 证据。
