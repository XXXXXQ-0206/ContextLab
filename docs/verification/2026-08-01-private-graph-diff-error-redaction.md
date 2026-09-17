# Private Graph Diff Error Redaction / 私有 Graph Diff 错误脱敏

## Scope / 范围

This receipt covers only the Web local graph-diff data adapter. Unknown-status structured proxy
messages are now redacted before entering the local error object, while HTTP status and structured
error code remain available for stable presentation. No route, SDK, OpenAPI, write, or diff
calculation changed.

本回执仅覆盖 Web local graph-diff data adapter。unknown-status structured proxy message 现会在进入 local error object 前脱敏，同时保留 HTTP status 与 structured
error code 供稳定 presentation 使用。不修改 route、SDK、OpenAPI、write 或 diff calculation。

## Fresh local evidence / 新鲜本地证据

| Check / 检查 | Result / 结果 |
| --- | --- |
| Red regression | `2 passed, 1 failed`; failure exposed the synthetic `sql://internal-db?token=secret` message under the old behavior |
| Green adapter suite | `3 passed` |
| Web workspace | `pnpm check:web` passed; public SDK `15`, local SDK `135`, Web `277`, TypeScript/lint and production build |
| Rust gates | format passed; workspace tests passed with storage `212 passed, 39 ignored`; strict offline Clippy and locked Rust `1.85.0` check passed |
| Boundary checks | scoped local verifier passed; `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` without unified diff input |

## Boundary / 边界

No raw upstream diagnostic, credential, or private payload is returned by the adapter's structured
message field. Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`. The long-term goal
remains active; the next implementation requires a new bilingual Necessity Record.

adapter 的 structured message field 不再返回 raw upstream diagnostic、credential 或 private payload。Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、
operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。长期目标保持 active；下一项实现必须先新增双语 Necessity Record。
