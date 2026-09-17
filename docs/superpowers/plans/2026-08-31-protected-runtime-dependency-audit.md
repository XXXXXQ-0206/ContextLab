# Protected Browser Runtime Dependency Audit / 受保护浏览器运行时依赖审计

**Status / 状态:** completed / verified locally for the dependency audit; authenticated runtime
evidence remains `unobserved`. / 依赖审计已完成并在本地验证；authenticated runtime 证据继续为
`unobserved`。

## Necessity Record / 必要性记录

### Named criteria and gap / 对应条件与缺口

This bounded audit advances Criteria 1, 5, and 8 by checking whether the next named boundary,
authenticated browser -> same-origin BFF -> protected Axum runtime, has a runnable local
PostgreSQL-backed environment. The existing preview browser smoke does not exercise protected
authentication or the PostgreSQL repository, so its receipt remains separate.

本有界审计通过检查 authenticated browser -> same-origin BFF -> protected Axum runtime 是否具备可运行的本地
PostgreSQL-backed environment，推进条件 1、5、8。既有 preview browser smoke 未覆盖 protected authentication 或
PostgreSQL repository，因此两类回执保持独立。

### Smallest boundary / 最小边界

Read only `.env.example`, repository startup configuration, Docker/WSL/PostgreSQL command
availability, service state, and loopback listeners. Do not read `.env`, use an existing database,
create a secret, add a route, add a producer, or change application behavior.

只读取 `.env.example`、仓库启动配置、Docker/WSL/PostgreSQL 命令可用性、service state 与 loopback listener。
不读取 `.env`，不使用现有数据库，不创建 secret，不新增 route、producer 或 application behavior。

### Fresh verification required / 新鲜验证要求

Record the exact dependency state, attempt the local Docker start command once, and classify the
protected runtime as `observed_pass`, `observed_fail`, or `unobserved` from direct evidence. A
missing disposable PostgreSQL service keeps live persistence and successful authenticated browser
evidence out of the receipt.

记录精确依赖状态，执行一次本地 Docker start command，并根据直接证据将 protected runtime 分类为
`observed_pass`、`observed_fail` 或 `unobserved`。缺少 disposable PostgreSQL service 时，live persistence 与成功的
authenticated browser evidence 不进入本回执。

## Execution Receipt / 执行回执

Observed on `2026-08-31` in the current worktree:

- Docker CLI is installed (`29.6.2`), but `docker info` and `docker version` did not obtain a
  server response. `docker desktop start` was attempted once; `com.docker.service` remained
  `Stopped` with `Manual` start type.
- `postgres`, `initdb`, `pg_ctl`, `createdb`, and `psql` were absent from the Windows command path.
- WSL is installed, but `wsl.exe --list --verbose` reported no usable distribution.
- `.env.example` requires protected mode to use `CONTEXTLAB_GRAPH_REPOSITORY=postgres`, a database
  URL, HMAC/OIDC authentication settings, and bounded protected rate-limit settings.

以上事实证明依赖审计本身完成，但没有形成 protected Axum success response、PostgreSQL migration/read/write
evidence 或 authenticated browser-to-BFF-to-protected-Axum success receipt。对应边界保持 `unobserved`；不以
preview 503 或 lazy PostgreSQL URL 替代 live runtime evidence。

Independent repository gates also passed after the audit: `pnpm.cmd check:web` (public SDK `15`,
local SDK `148`, Web `310`, TypeScript, and production build), `cargo +1.85.0 test --workspace
--quiet --offline -j 1` (API `223 passed`, storage `239 passed, 41 ignored`), Rust format,
strict offline Clippy, locked Rust check, `tests/contract/verify-local-contracts.test.ps1`, and
the direct local verifier. The direct verifier reported `overall=unobserved` only because no
unified diff input was supplied; `git diff --check` passed.

依赖审计后独立运行的仓库门禁也全部通过：`pnpm.cmd check:web`（public SDK `15`、local SDK `148`、Web `310`、TypeScript 与
production build）、`cargo +1.85.0 test --workspace --quiet --offline -j 1`（API `223 passed`、storage `239 passed, 41 ignored`）、
Rust format、strict offline Clippy、locked Rust check、`tests/contract/verify-local-contracts.test.ps1` 与 direct local verifier。
direct verifier 仅因未提供 unified diff input 报告 `overall=unobserved`；`git diff --check` 通过。

No source, API, SDK, OpenAPI, Web, migration, provider, secret, database, write, or GraphDiff
behavior changed. The long-term goal remains `active`. When a disposable loopback PostgreSQL
service is available, rerun the named lifecycle PostgreSQL cases and add a separate authenticated
browser smoke with a fixture JWT; until then, deferred runtime evidence stays outside the local
implementation queue.

未修改 source、API、SDK、OpenAPI、Web、migration、provider、secret、database、write 或 GraphDiff behavior。长期目标保持
`active`。当 disposable loopback PostgreSQL service 就绪后，再运行具名 lifecycle PostgreSQL cases，并使用 fixture JWT
新增独立 authenticated browser smoke；在此之前，延期的 runtime evidence 不进入本地 implementation queue。
