# Protected Runtime Authentication Design / 受保护运行时认证设计

## Goal / 目标

Add an explicit, private protected-router runtime profile for PostgreSQL-backed Context commit writes. It must fail closed when authentication configuration is incomplete, require issuer and audience validation, and leave the default API process public and read-only.

为 PostgreSQL-backed Context commit write 增加显式、私有的 protected-router runtime profile。认证配置不完整时必须 fail closed，必须校验 issuer 与 audience，并保持默认 API process 为 public、只读。

## Decision / 决策

Use `CONTEXTLAB_API_ROUTE_MODE` with the values `public` (default) and `protected`. In `protected` mode, runtime requires all of the following:

- `CONTEXTLAB_GRAPH_REPOSITORY=postgres`
- a non-empty `CONTEXTLAB_AUTH_HS256_SECRET`
- non-empty `CONTEXTLAB_AUTH_ISSUER` and `CONTEXTLAB_AUTH_AUDIENCE`

The builder constructs one PostgreSQL repository bundle, derives the durable audit sink already provided by PostgreSQL composition, and injects that same pool-backed repository as both the membership authorizer and guarded commit writer. It constructs `HmacJwtAuthenticator` only after configuration validation and installs the existing protected router. No configuration error includes a secret value.

使用 `CONTEXTLAB_API_ROUTE_MODE`，可选值为 `public`（默认）和 `protected`。在 `protected` mode，runtime 必须同时满足：

- `CONTEXTLAB_GRAPH_REPOSITORY=postgres`
- 非空 `CONTEXTLAB_AUTH_HS256_SECRET`
- 非空 `CONTEXTLAB_AUTH_ISSUER` 与 `CONTEXTLAB_AUTH_AUDIENCE`

builder 构造一组 PostgreSQL repository bundle，沿用既有 PostgreSQL composition 的 durable audit sink，并将同一个 pool-backed repository 注入为 membership authorizer 与 guarded commit writer。只有配置校验完成后才构造 `HmacJwtAuthenticator` 并安装既有 protected router。任何配置 error 都不得包含 secret value。

## Alternatives / 备选方案

1. **Selected: explicit HMAC protected profile / 采用：显式 HMAC protected profile.** It converts the existing tested verifier into a real opt-in runtime path without publishing mutation contracts. Issuer and audience prevent accepting a token intended for another deployment.
2. **Always enable protected routes / 始终启用 protected route.** Rejected: it changes default server behavior and risks exposing a mutation before the release gate.
3. **Implement OAuth2/OIDC JWKS immediately / 立即实现 OAuth2/OIDC JWKS.** Deferred: a production-grade remote key resolver needs issuer discovery, key rotation, HTTP caching, algorithm policy, outage behavior, and security review. It is a distinct next security increment, not a substitute for explicit runtime composition.

## Boundaries / 边界

- `build_router()` and `try_build_router_from_current_env()` remain public-only in `public` mode.
- `protected` mode remains opt-in and never adds a path to `PUBLIC_POST_ROUTES`, OpenAPI, or the TypeScript SDK.
- Memory/preview storage cannot activate protected mode because it cannot provide persistent membership, guarded-write, or audit infrastructure.
- The profile accepts only HS256. OAuth2/OIDC/JWKS, asymmetric algorithms, key rotation, rate limiting, audit retention, and public promotion remain unfinished.
- The code parses environment pairs for runtime only; tests use fixed non-secret values and no task reads `.env`.

- `public` mode 下 `build_router()` 与 `try_build_router_from_current_env()` 仍为 public-only。
- `protected` mode 保持 opt-in，绝不向 `PUBLIC_POST_ROUTES`、OpenAPI 或 TypeScript SDK 增加 path。
- memory/preview storage 不能启动 protected mode，因为无法提供持久化 membership、guarded-write 或 audit infrastructure。
- profile 仅接受 HS256。OAuth2/OIDC/JWKS、asymmetric algorithm、key rotation、rate limiting、audit retention 与 public promotion 仍未完成。
- 代码仅为 runtime 解析 environment pair；test 使用固定的非 secret 值，任务不读取 `.env`。

## Verification / 验证

1. Missing route mode configuration selects public router, whose Context commit POST remains absent.
2. Unsupported route mode, protected mode with memory storage, and each missing protected authentication value return a structured safe `AppStateConfigError`.
3. A PostgreSQL protected profile with fixed test values builds without a live database; a missing bearer header yields the existing `401 authentication_required`, proving the protected middleware is installed.
4. Existing route catalog/OpenAPI tests continue to prove that public POST paths are unchanged.
5. Full Rust, Web, and workspace verification gates run after implementation.

1. 缺省 route mode 选择 public router，其 Context commit POST 仍不存在。
2. 不支持的 route mode、protected mode 配合 memory storage，以及每个缺失的 protected authentication 值均返回结构化且安全的 `AppStateConfigError`。
3. 使用固定 test 值的 PostgreSQL protected profile 无需 live database 即可构造；缺少 bearer header 时返回既有 `401 authentication_required`，证明 protected middleware 已安装。
4. 既有 route catalog/OpenAPI test 持续证明 public POST path 未改变。
5. 实现后运行完整 Rust、Web 与 workspace verification gate。
