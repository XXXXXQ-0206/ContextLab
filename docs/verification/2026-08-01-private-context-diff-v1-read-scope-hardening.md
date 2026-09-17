# Private Context Diff V1 Read-Scope Hardening Receipt / 私有 Context Diff V1 读取范围硬化回执

## Result / 结果

This bounded storage contract increment is `completed / verified locally`. The PostgreSQL V1 read
now binds `schema_version = context-diff-snapshot-v1` instead of selecting an arbitrary row by
schema ordering. The Memory contract test independently proves that project, Context, and commit
identity drift each returns `NotFound`.

本有界 storage contract 增量标记为 `completed / verified locally`。PostgreSQL V1 read 现显式绑定
`schema_version = context-diff-snapshot-v1`，不再按 schema ordering 选择任意 row。Memory contract test 分别证明 project、Context 与 commit identity 漂移都会返回 `NotFound`。

## Fresh Evidence / 新鲜证据

- Focused Memory scope repository: `4 passed`.
- Focused PostgreSQL SQL contract: `1 passed`.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; API `214 passed`, storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production build.
- `scripts/verify-local-contracts.ps1`: scoped checks passed, `graph_diff_application=passed count=1`; `overall=unobserved` because no unified diff input was supplied.
- `GRAPH_DIFF_IMPL_COUNT=1`: passed.

- focused Memory scope repository：`4 passed`。
- focused PostgreSQL SQL contract：`1 passed`。
- `cargo fmt --all -- --check`：通过。
- `cargo test --workspace --quiet --no-fail-fast --offline`：通过；API `214 passed`、storage `212 passed, 39 ignored`。
- `cargo clippy --workspace --all-targets --offline -- -D warnings`：通过。
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`：通过。
- `pnpm check:web`：通过；public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build。
- `scripts/verify-local-contracts.ps1`：scoped checks 通过，`graph_diff_application=passed count=1`；因未提供 unified diff input，`overall=unobserved`。
- `GRAPH_DIFF_IMPL_COUNT=1`：通过。

## Boundary / 边界

No migration, runtime PostgreSQL execution, public route, OpenAPI/public SDK method, Web mutation,
operator transport, provider, secret access, or second diff calculator changed. Docker/PostgreSQL
runtime, authenticated browser, visual smoke, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. The long-term goal remains active; the
next increment needs a new bilingual Necessity Record.

没有新增 migration、PostgreSQL runtime execution、public route、OpenAPI/public SDK method、Web mutation、operator transport、provider、secret access 或第二个 diff calculator。
Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。
长期目标保持 active；下一增量必须新增双语 Necessity Record。
