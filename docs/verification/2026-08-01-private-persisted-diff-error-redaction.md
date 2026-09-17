# Private Persisted Context Diff Error Redaction / 私有持久化 Context Diff 错误脱敏

## Scope / 范围

This receipt covers the Web adapter for the private persisted Context semantic/behavior/evaluation
diff review. It keeps exact project/Context/source/target scope and retry behavior while redacting
structured upstream messages before they reach the client-visible error object. No route, SDK,
OpenAPI, write, or diff calculator changed.

本回执覆盖私有 persisted Context semantic/behavior/evaluation diff review 的 Web adapter。它保持 exact project/Context/source/target scope 与 retry behavior，
并在 structured upstream message 进入 client-visible error object 前完成脱敏。不修改 route、SDK、OpenAPI、write 或 diff calculator。

## Fresh local evidence / 新鲜本地证据

| Check / 检查 | Result / 结果 |
| --- | --- |
| Red regression | `4 passed, 2 failed`; old behavior exposed `sql://internal-db?token=secret diagnostic payload` |
| Green adapter suite | `6 passed` |
| Web workspace | `pnpm check:web` passed; public SDK `15`, local SDK `135`, Web `278`, TypeScript/lint and production build |
| Rust gates | format passed; workspace tests passed with storage `212 passed, 39 ignored`; strict offline Clippy and locked Rust `1.85.0` check passed |
| Boundary checks | scoped local verifier passed; `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` without unified diff input |

## Boundary / 边界

The adapter exposes only stable local error text, status, and upstream error code; raw diagnostics,
credentials, and private payloads are excluded. Docker/PostgreSQL runtime, authenticated
browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`. The long-term goal remains active and the next implementation requires
a new bilingual Necessity Record.

adapter 只暴露稳定 local error text、status 与 upstream error code；raw diagnostic、credential 与 private payload 均被排除。Docker/PostgreSQL runtime、authenticated browser/visual smoke、
Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。长期目标保持 active，下一项实现必须先新增双语 Necessity Record。
