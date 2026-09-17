# Private Versioned Context Diff Boundary Witness / 私有版本化 Context Diff 边界见证

## Scope / 范围

This local receipt covers only two read-contract evidence additions: an independent protected
router witness for exact version-backed Context Graph diff scope, and local SDK parser coverage for
semantic, behavior, and evaluation `added`/`removed` variants. It does not claim that mocked SDK or
Web fixtures are a live database-to-browser path.

本地回执仅覆盖两项读取契约证据增量：独立 protected router 对精确版本化 Context Graph diff scope 的见证，以及 local SDK 对 semantic、behavior、evaluation
的 `added`/`removed` variant 覆盖。它不把 mocked SDK 或 Web fixture 描述为真实 database-to-browser path。

## Fresh Evidence / 新鲜证据

| Check / 检查 | Result / 结果 |
| --- | --- |
| Protected API boundary / 受保护 API 边界 | `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture`: `3 passed`, including missing auth, exact scope, non-empty stable graph diff, private cache headers, public-route retirement, and same-commit rejection. |
| Local SDK parser matrix / Local SDK parser 矩阵 | `pnpm exec tsx --test src/persisted-context-diff-review.test.ts`: `10 passed`; semantic document, behavior case, and evaluation metric each cover `added` and `removed`, with extra-field rejection. |
| Rust format / Rust 格式 | `cargo fmt --all -- --check` passed. |
| Rust workspace / Rust workspace | `cargo test --workspace --quiet --no-fail-fast --offline` passed; API `214 passed`; storage `212 passed, 39 ignored`. |
| Strict lint / 严格 lint | `cargo clippy --workspace --all-targets --offline -- -D warnings` passed. |
| Locked toolchain / 锁定工具链 | `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed. |
| Web and SDK / Web 与 SDK | `pnpm check:web` passed; public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production build. |
| Contract verifier / Contract verifier | Source, graph, safe-DTO, protected-route/catalog checks passed; `graph_diff_application=passed count=1`; `overall=unobserved` because no unified diff input was supplied. |
| Sole calculator / 唯一 calculator | `GRAPH_DIFF_IMPL_COUNT=1` passed. |

## Boundaries / 边界

The API witness reads the existing protected graph-diff route with an in-memory snapshot fixture;
the SDK matrix parses Rust-shaped fixtures. Neither proves writer-to-review repository identity in
the Memory application composition, a PostgreSQL-backed service flow, authenticated browser or
visual E2E, Git change-set, remote CI, operator rehearsal, release, or production promotion.

API witness 使用既有 protected graph-diff route 与 in-memory snapshot fixture；SDK matrix 解析 Rust-shaped fixture。两者都不能证明 Memory application composition
中的 writer-to-review repository identity、PostgreSQL-backed service flow、authenticated browser 或 visual E2E、Git change-set、remote CI、operator rehearsal、release 或 production promotion。

No public write, OpenAPI/public SDK write method, Web mutation, migration, provider, secret access,
operator transport, or second `GraphDiff` calculator was added. The long-term goal remains active;
external deployment evidence remains `unobserved` or `deferred`.

没有新增 public write、OpenAPI/public SDK write method、Web mutation、migration、provider、secret access、operator transport 或第二个 `GraphDiff` calculator。长期目标保持 active；外部部署证据继续为 `unobserved` 或 `deferred`。
