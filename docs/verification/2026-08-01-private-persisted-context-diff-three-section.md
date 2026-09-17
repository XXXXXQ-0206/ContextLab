# Private Persisted Context Diff Receipt / 私有持久化 Context Diff 回执

## Scope / 范围

This receipt covers only the local, private persisted Context diff path. The commit writer now
derives a validated V1 diff input from the same graph and persists it with the commit and graph
snapshot in the existing Memory/PostgreSQL transaction boundary. The local API, SDK, BFF, and Web
tests verify exact scope, redaction, and non-empty semantic, behavior, and evaluation sections
where producer evidence is explicitly supplied.

本回执仅覆盖本地 private persisted Context diff path。commit writer 现从同一 graph 派生经过校验的 V1 diff input，并在既有 Memory/PostgreSQL
transaction boundary 中与 commit、graph snapshot 一起持久化。local API、SDK、BFF 与 Web tests 验证 exact scope、脱敏，以及在明确提供
producer evidence 时 semantic、behavior、evaluation 三段均非空。

## Fresh Evidence / 新鲜证据

| Check / 检查 | Result / 结果 |
| --- | --- |
| Rust format / Rust 格式 | `cargo fmt --all -- --check` passed |
| Rust workspace / Rust workspace | `cargo test --workspace --quiet --no-fail-fast --offline` passed; API `214 passed`; storage `212 passed, 39 ignored` |
| Focused storage/API / storage/API 聚焦 | guarded writer replay `1 passed`; storage review `8 passed`; direct API review `1 passed`; protected API review `3 passed` |
| Strict lint / 严格 lint | `cargo clippy --workspace --all-targets --offline -- -D warnings` passed |
| Locked toolchain / 锁定工具链 | `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed |
| Web and SDK / Web 与 SDK | `pnpm check:web` passed; public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production build |
| Contract verifier / Contract verifier | scoped checks passed; `graph_diff_application=passed count=1`; `overall=unobserved` because no unified diff input was supplied |
| Sole calculator / 唯一 calculator | `GRAPH_DIFF_IMPL_COUNT=1` passed |

## Boundaries / 边界

The default derived commit snapshot keeps behavior/evaluation collections empty when no benchmark
producer has supplied evidence. The non-empty API/BFF fixtures are redacted test producer facts,
not runtime evaluation claims. `GraphDiff::between` remains the sole graph-diff calculator.

默认 derived commit snapshot 在没有 benchmark producer evidence 时保持 behavior/evaluation collection 为空。非空 API/BFF fixture 是脱敏测试 producer
facts，不是运行时评测声明。`GraphDiff::between` 仍是唯一 graph-diff calculator。

No public REST/OpenAPI/public SDK write, operator transport, Web mutation, provider, secret access,
or production migration was added. PostgreSQL/Docker runtime, authenticated browser, visual smoke,
Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`. The long-term goal remains active.

没有新增 public REST/OpenAPI/public SDK write、operator transport、Web mutation、provider、secret access 或 production migration。PostgreSQL/Docker runtime、
authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active。
