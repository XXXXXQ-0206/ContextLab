# Private Exact-Commit Context Graph Relationship Inspector / 私有精确提交 Context Graph 关系检查器

## Scope / 范围

This receipt covers the private local SDK and Web read projection for an exact Context lifecycle
commit. It repairs the Rust-shaped `graph_snapshot.project_id` contract and exposes all supported
graph edge kinds through the existing `data -> presenter -> screen` boundary. It does not add a
route, public SDK/OpenAPI method, write, migration, or second graph-diff calculator.

本回执覆盖精确 Context lifecycle commit 的私有 local SDK 与 Web read projection。它修复 Rust-shaped
`graph_snapshot.project_id` contract，并通过既有 `data -> presenter -> screen` 边界呈现全部支持的 graph edge kind。不新增 route、public SDK/OpenAPI method、
write、migration 或第二个 graph-diff calculator。

## Implementation / 实现

- `packages/local-sdk/src/types.ts` validates and preserves the snapshot project UUID.
- `apps/web/src/app/context-lifecycle-presenter.ts` owns strict relationship adaptation, labels,
  deterministic ordering, exact commit identity, and redacted node/edge facts.
- `apps/web/src/app/context-lifecycle-editor.tsx` renders only the presenter model with shared
  `DefinitionGrid` and `StackTable` primitives, including bilingual empty/accessibility states.

- `packages/local-sdk/src/types.ts` 校验并保留 snapshot project UUID。
- `apps/web/src/app/context-lifecycle-presenter.ts` 负责严格 relationship adaptation、双语 label、确定性排序、exact commit identity 与脱敏 node/edge fact。
- `apps/web/src/app/context-lifecycle-editor.tsx` 仅使用 presenter model 与 shared `DefinitionGrid`、`StackTable` primitive 渲染，并包含双语 empty/accessibility state。

## Fresh local evidence / 新鲜本地证据

| Check / 检查 | Result / 结果 |
| --- | --- |
| Focused Web presenter/editor | `29 passed` |
| Local SDK | `135 passed`; TypeScript check passed |
| Rust format and workspace | `cargo fmt --all -- --check` passed; workspace tests passed, storage `212 passed, 39 ignored` |
| Quality gates | strict offline Clippy and locked Rust `1.85.0` workspace check passed |
| Web workspace | `pnpm check:web` passed; public SDK `15`, local SDK `135`, Web `276`, TypeScript/lint and production build |
| Contract boundary | scoped verifier checks passed; `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` without unified diff input |

## Evidence boundary / 证据边界

No raw component content, metadata, credentials, or private payload enters the relationship model.
Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git change-set, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`. The long-term goal remains
active; the next implementation requires a new bilingual Necessity Record and a fresh dependency
audit.

relationship model 不包含 raw component content、metadata、credential 或 private payload。Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git change-set、
remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。长期目标保持 active；下一项实现必须先新增双语 Necessity Record 并进行新鲜依赖审计。
