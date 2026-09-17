# ContextLab

ContextLab is an open-source Context Engineering platform for designing, versioning, evaluating, and collaborating on production-grade AI workflows.

ContextLab 是一个开源 Context Engineering（上下文工程）平台，用于设计、版本化、评测和协作构建生产级 AI 工作流。

## Product Direction / 产品方向

ContextLab is context-first, not prompt-first. Prompts, memory, knowledge, retrieval, model configuration, tools, MCP servers, variables, schemas, workflows, conversations, experiments, evaluations, and results are all parts of a reproducible Context graph.

ContextLab 以 Context 为核心，而不是以 Prompt 为核心。Prompt、Memory、Knowledge、Retrieval、Model Configuration、Tool、MCP Server、Variable、Schema、Workflow、Conversation、Experiment、Evaluation 和 Result 都属于可复现的 Context Graph。

The long-term goal is to become the Git, GitHub, and Figma of AI Context Engineering: versioned, searchable, comparable, replayable, extensible, and collaborative.

长期目标是成为 AI Context Engineering 领域的 Git、GitHub 和 Figma：可版本化、可搜索、可比较、可回放、可扩展、可协作。

The active convergence criteria live in `docs/roadmap/completion-criteria.md`; a single feature, route, or screen is not enough to close the long-term project goal.

当前长期收束条件位于 `docs/roadmap/completion-criteria.md`；单个 feature、route 或 screen 不足以关闭长期项目目标。

## Current Foundation / 当前地基

- Rust workspace for framework-independent domain logic.
- 可独立测试的 Rust workspace，用于承载不依赖框架的领域逻辑。
- `context-core` for Context identity, components, metadata, and validation.
- `context-core` 负责 Context 身份、组件、元数据和领域校验。
- `versioning` for Git-like context commits and branches.
- `versioning` 负责类 Git 的 Context commit 与 branch。
- `diff-engine` for deterministic text diff foundations.
- `diff-engine` 负责确定性的文本 diff 基础。
- `evaluation` for metric measurements and scorecards.
- `evaluation` 负责评测指标与 scorecard。
- `graph` for explicit Context Graph nodes and relationships.
- `graph` 负责显式 Context Graph 节点与关系。
- `model-gateway` for provider configuration without leaking credentials.
- `model-gateway` 负责模型 provider 配置，并避免泄露凭据。
- `storage` for PostgreSQL schema assets and graph projection records.
- `storage` 负责 PostgreSQL schema 资产与 graph projection 记录模型。
- Storage repository contracts now back the Context Graph preview route, with an in-memory implementation ready to be replaced by SQLx.
- Storage repository contract 已经支撑 Context Graph preview route，当前 in-memory 实现可在后续替换为 SQLx 实现。
- `PostgresContextGraphRepository` provides the first SQLx-backed workspace graph projection adapter while preserving the same repository contract.
- `PostgresContextGraphRepository` 提供第一版 SQLx-backed workspace graph projection adapter，并保持同一个 repository contract。
- PostgreSQL workspace graph reads use a read-only transaction and have an opt-in seed integration test.
- PostgreSQL workspace graph read 使用 read-only transaction，并提供 opt-in seed integration test。
- `GET /api/v1/workspaces` exposes paginated workspace discovery with backend-consistent slug and creation-time sorting semantics.
- `GET /api/v1/workspaces` 提供分页 workspace 发现能力，并保持后端一致的 slug 与创建时间排序语义。
- `GET /api/v1/workspaces/{workspace_id}/projects` exposes the first workspace-scoped project discovery API.
- `GET /api/v1/workspaces/{workspace_id}/projects` 提供第一版 workspace-scoped project discovery API。
- `GET /api/v1/projects/{project_id}/experiments` exposes the first project-scoped experiment discovery API with branch names.
- `GET /api/v1/projects/{project_id}/experiments` 提供第一版 project-scoped experiment discovery API，并暴露 branch name。
- `GET /api/v1/projects/{project_id}/contexts` exposes project-scoped Context discovery with optional experiment filtering.
- `GET /api/v1/projects/{project_id}/contexts` 提供 project-scoped Context discovery，并支持可选的 experiment 过滤。
- `GET /api/v1/contexts/{context_id}/commits` exposes the first context-scoped version history discovery API.
- `GET /api/v1/contexts/{context_id}/commits` 提供第一版 context-scoped version history discovery API。
- `GET /api/v1/contexts/{context_id}/components` exposes context-scoped component inventory with reproducible content fingerprints.
- `GET /api/v1/contexts/{context_id}/components` 提供 context-scoped component inventory，并暴露可复现的 content fingerprint。
- `GET /api/v1/contexts/{context_id}/components/{component_id}` exposes component metadata, fingerprints, and timestamps; it does not expose component bodies.
- `GET /api/v1/contexts/{context_id}/components/{component_id}` 提供 component metadata、fingerprint 与 timestamp；它不暴露 component 正文。
- `GET /api/v1/contexts/{context_id}/evaluation-runs` exposes context-scoped benchmark and regression run discovery.
- `GET /api/v1/contexts/{context_id}/evaluation-runs` 提供 context-scoped benchmark 与 regression run discovery。
- `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}` exposes persisted metrics JSON for one benchmark or regression run.
- `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}` 提供单次 benchmark 或 regression run 的持久化 metrics JSON。
- `GET /api/v1/contexts/{context_id}/evaluation-scorecard` exposes context-level numeric metric averages across matching evaluation runs.
- `GET /api/v1/contexts/{context_id}/evaluation-scorecard` 提供 context-level numeric metric average，用于汇总匹配的 evaluation run。
- `GET /api/v1/workspaces/{workspace_id}/context-graph` exposes the first workspace-scoped graph read API.
- `GET /api/v1/workspaces/{workspace_id}/context-graph` 提供第一版 workspace-scoped graph read API。
- `server/api` for the Axum REST API shell.
- `server/api` 负责 Axum REST API 外壳。
- `packages/design-system` and `packages/ui` for the token-first interface foundation.
- `packages/design-system` 与 `packages/ui` 负责 token 优先的界面地基。
- `packages/ts-sdk` for the first TypeScript REST discovery client and shared API DTOs.
- `packages/ts-sdk` 负责第一版 TypeScript REST discovery client 与共享 API DTO。
- `docs/api/openapi.json` for the checked-in OpenAPI contract covering the current public REST read operations and GraphDiff POST operation.
- `POST /api/v1/graph-diffs` compares two explicit Context Graph snapshots without persisting either one; invalid graph input returns the structured `invalid_graph_snapshot` error.
- `POST /api/v1/graph-diffs` 对两个显式 Context Graph snapshot 进行比较且不会持久化任一快照；无效图输入会返回结构化的 `invalid_graph_snapshot` error。
- `docs/api/openapi.json` 负责第一版纳入仓库的 OpenAPI contract，覆盖当前 REST GET surface 和 GraphDiff POST operation。
- `server/api` owns public GET and POST route catalogs that are tested against the OpenAPI contract to prevent API documentation drift.
- `server/api` 维护 public GET 与 POST route catalog，并通过测试与 OpenAPI contract 对齐，防止 API 文档漂移。
- `apps/web` for the first Context Engineering workspace shell, now mirroring Context, commit, component discovery/detail, evaluation run discovery/detail, selected scorecard averages, and workspace Context Graph reads with API-shaped preview data and optional live API loading.
- `apps/web` 负责第一版 Context Engineering 工作台外壳，目前已用 API-shaped preview data 和可选 live API loading 对齐 Context、commit、component discovery/detail、evaluation run discovery/detail、selected scorecard average 与 workspace Context Graph read。

## Repository Shape / 仓库结构

```text
apps/       # Web, desktop, CLI, and docs applications
server/     # API presentation layer
crates/     # Rust core platform crates
packages/   # Design system, UI package, TypeScript SDK, shared frontend code
docs/       # Architecture, ADRs, plans, and contribution guides
```

## Verify / 验证

```bash
cargo test --workspace
cargo run -p contextlab-api
curl http://127.0.0.1:3100/api/v1/context-graph/preview
curl http://127.0.0.1:3100/api/v1/openapi.json
curl "http://127.0.0.1:3100/api/v1/workspaces?page=1&per_page=20&sort=created_at"
curl "http://127.0.0.1:3100/api/v1/workspaces/default/projects?page=1&per_page=20&sort=created_at"
curl "http://127.0.0.1:3100/api/v1/projects/support-ai/experiments?page=1&per_page=20&sort=branch_name"
curl "http://127.0.0.1:3100/api/v1/projects/support-ai/contexts?page=1&per_page=20&sort=created_at"
curl "http://127.0.0.1:3100/api/v1/contexts/support-resolution-agent/commits?page=1&per_page=20&sort=-authored_at"
curl "http://127.0.0.1:3100/api/v1/contexts/support-resolution-agent/components?page=1&per_page=20&kind=knowledge&sort=kind"
curl "http://127.0.0.1:3100/api/v1/contexts/support-resolution-agent/components/refund-policy"
curl "http://127.0.0.1:3100/api/v1/contexts/support-resolution-agent/evaluation-runs?page=1&per_page=20&sort=-executed_at"
curl "http://127.0.0.1:3100/api/v1/contexts/support-resolution-agent/evaluation-runs/safety-regression"
curl "http://127.0.0.1:3100/api/v1/contexts/support-resolution-agent/evaluation-scorecard?suite_name=Safety%20Regression%20Suite&model_version=deepseek-chat"
curl http://127.0.0.1:3100/api/v1/workspaces/default/context-graph
curl http://127.0.0.1:3100/api/v1/providers
pnpm install
pnpm --filter @contextlab/ts-sdk lint
pnpm --filter @contextlab/ts-sdk test
pnpm --filter @contextlab/web lint
pnpm --filter @contextlab/web build
pnpm --filter @contextlab/web dev -- --hostname 127.0.0.1 --port 3000
python apps/web/verify-context-workspace.py
```

The API starts on `127.0.0.1:3100` by default.

API 默认监听 `127.0.0.1:3100`。

The OpenAPI contract is checked in at `docs/api/openapi.json` and served at `GET /api/v1/openapi.json`; `cargo test -p contextlab-api` verifies GET/POST route catalog parity, and `pnpm --filter @contextlab/ts-sdk test` verifies that the SDK public client surface, including GraphDiff POST, stays aligned with that contract.

OpenAPI contract 位于 `docs/api/openapi.json`，并通过 `GET /api/v1/openapi.json` 对外提供；`cargo test -p contextlab-api` 会验证 GET/POST route catalog 对齐，`pnpm --filter @contextlab/ts-sdk test` 会验证包含 GraphDiff POST 在内的 SDK public client surface 与该 contract 保持一致。

### Disposable PostgreSQL CI / 一次性 PostgreSQL CI

GitHub Actions provisions an empty PostgreSQL 16.14 service and runs the named storage migration and integration suite through `scripts/verify-disposable-postgres-storage.sh`. The verifier accepts only a loopback PostgreSQL URL, checks an explicit disposable database name and dedicated test role, requires an explicit reset opt-in, and resets the disposable `public` schema before each named test. It never reads a database URL from `.env`.

GitHub Actions 会创建空的 PostgreSQL 16.14 服务，并通过 `scripts/verify-disposable-postgres-storage.sh` 运行指定的存储迁移与集成套件。该验证器只接受 loopback PostgreSQL URL，校验显式的一次性数据库名称与专用测试角色，要求显式确认 reset，并在每个指定测试前重置一次性 `public` schema；它绝不会从 `.env` 读取数据库 URL。

This is non-production migration and storage-integration evidence only. It does not prove production upgrade or rollback compatibility, restricted purge execution, public audit REST/SDK/Web access, or any graph-diff engine beyond `GraphDiff`.

这仅是非生产迁移与存储集成证据；它不证明生产升级或回滚兼容性、受限清理执行、公开审计 REST/SDK/Web 访问，也不引入 `GraphDiff` 之外的图差异引擎。

After every successful remote `verify` job, GitHub Actions retains a 90-day `contextlab-ci-evidence` artifact containing the checked-out commit/provider SHA equality, a workflow SHA-256 fingerprint, UTC capture time, and a SHA-256 for the capture file. It excludes environment values, database URLs, credentials, and logs. This capture is evidence input for an independently reviewed remote receipt; it is not a production claim or public-write approval.

每次远端 `verify` job 成功后，GitHub Actions 会保留 90 天的 `contextlab-ci-evidence` artifact，其中包含 checked-out commit/provider SHA equality、workflow SHA-256 fingerprint、UTC capture time 与 capture file 的 SHA-256。它不包含 environment value、database URL、credential 或 log。该捕获是独立审阅远端回执的 evidence input，不是 production claim 或 public-write approval。

The same successful job retains a separate 90-day `contextlab-rehearsal-evidence` artifact. It contains only SHA-256 manifests for regular migrations, privileged SQL assets, and the rehearsal runner; it does not include database configuration or prove operator approval, non-production isolation, execution, cleanup, or release readiness.

同一个成功 job 还会保留单独的、90 天有效期的 `contextlab-rehearsal-evidence` artifact。它只包含 regular migration、privileged SQL asset 与 rehearsal runner 的 SHA-256 manifest；不包含 database configuration，也不证明 operator approval、non-production isolation、execution、cleanup 或 release readiness。

These `unobserved` external receipts are deferred release evidence. They remain mandatory before public protected-write promotion, release, or production rollout, but they do not pause dependency-ready private Context-first development. Local tests never replace or impersonate the external receipts.

这些 `unobserved` 外部回执属于延期发布证据。它们在 public protected-write promotion、release 或生产推广前仍是必需条件，但不会暂停依赖就绪的私有 Context-first 开发；本地测试绝不替代或冒充外部回执。

The private audit purge executor uses a dedicated runtime pool and an operator-provisioned non-login function owner. The runtime role has `EXECUTE` but no direct audit-table read or mutation privilege. A successful remote CI run and a production-like migration rehearsal remain required before this becomes production evidence.

私有 audit purge executor 使用专用 runtime pool 与 operator provision 的无登录 function owner。runtime role 只有 `EXECUTE`，没有直接读取或修改 audit table 的权限。在将其视为生产证据前，仍需要成功的远端 CI 运行和生产相似环境的 migration rehearsal。

Provider keys are loaded from local `.env` files. API responses only expose whether a provider is configured plus a redacted key fingerprint.

Provider key 从本地 `.env` 读取。API 响应只暴露 provider 是否已配置以及脱敏后的 key 指纹。

Workspace graph reads use the in-memory repository by default. Set `CONTEXTLAB_GRAPH_REPOSITORY=postgres` and `CONTEXTLAB_DATABASE_URL` to route workspace graph reads through PostgreSQL while keeping the preview graph deterministic. `DATABASE_URL` is accepted as a compatibility fallback.

Workspace graph read 默认使用 in-memory repository。设置 `CONTEXTLAB_GRAPH_REPOSITORY=postgres` 与 `CONTEXTLAB_DATABASE_URL` 后，workspace graph read 会走 PostgreSQL，同时 preview graph 继续保持确定性。`DATABASE_URL` 作为兼容 fallback 被支持。

The API route mode defaults to `public`, which exposes only the checked-in read and supplied-snapshot GraphDiff contracts. An explicitly private `protected` mode installs guarded Context commits and the local component lifecycle contract only when `CONTEXTLAB_GRAPH_REPOSITORY=postgres`; its lifecycle routes remain absent from public OpenAPI and the public TypeScript SDK. `CONTEXTLAB_AUTH_MODE=hmac` is the compatible default and requires `CONTEXTLAB_AUTH_HS256_SECRET`, `CONTEXTLAB_AUTH_ISSUER`, and `CONTEXTLAB_AUTH_AUDIENCE`. `CONTEXTLAB_AUTH_MODE=oidc` accepts only RS256 JWTs from an HTTPS JWKS endpoint and requires `CONTEXTLAB_OIDC_ISSUER`, `CONTEXTLAB_OIDC_AUDIENCE`, `CONTEXTLAB_OIDC_JWKS_URL`, and `CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS` (`1..=3600`). OIDC keys are cached for that bounded lifetime and an unknown `kid` performs one synchronized refresh. Both modes validate issuer and audience, then form an opaque, case-sensitive `PrincipalIdentity` from the exact validated issuer (`IdentitySourceId`) and JWT subject (`PrincipalId`). PostgreSQL writes authorization audit events through that composite identity and fails closed when any required setting is invalid or absent.

API route mode 默认是 `public`，只暴露已纳入仓库的 read 与 supplied-snapshot GraphDiff contract。只有当 `CONTEXTLAB_GRAPH_REPOSITORY=postgres` 时，显式私有的 `protected` mode 才会安装 guarded Context commit 与本地 component lifecycle contract；其 lifecycle route 仍不进入 public OpenAPI 或 public TypeScript SDK。兼容默认值 `CONTEXTLAB_AUTH_MODE=hmac` 需要 `CONTEXTLAB_AUTH_HS256_SECRET`、`CONTEXTLAB_AUTH_ISSUER` 与 `CONTEXTLAB_AUTH_AUDIENCE`。`CONTEXTLAB_AUTH_MODE=oidc` 只接受来自 HTTPS JWKS endpoint 的 RS256 JWT，并需要 `CONTEXTLAB_OIDC_ISSUER`、`CONTEXTLAB_OIDC_AUDIENCE`、`CONTEXTLAB_OIDC_JWKS_URL` 与 `CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS`（`1..=3600`）。OIDC key 会在该有界生命周期内缓存，遇到未知 `kid` 时只执行一次同步 refresh。两种 mode 都会校验 issuer 与 audience，再把精确的已验证 issuer（`IdentitySourceId`）和 JWT subject（`PrincipalId`）组合成不透明且大小写敏感的 `PrincipalIdentity`。PostgreSQL 通过该组合身份写入 authorization audit event，并在必需配置无效或缺失时 fail closed。

Protected mode additionally requires `CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS` (`1..=1000`), `CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS` (`1..=3600`), and `CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS` (`1..=100000`). After authentication, the bounded in-process sliding window uses exactly the authenticated `PrincipalIdentity` plus a protected route operation; local lifecycle state reads use `ContextLifecycleRead`, while guarded commits and local lifecycle mutations use `ContextCommitWrite`. It does not bucket by Context id. Quota rejection returns `429 rate_limit_exceeded` with an integer `Retry-After`. Capacity exhaustion or internal limiter failure closes with `503 rate_limit_unavailable`. A limiter rejection never reaches authorization audit or the commit writer.

protected mode 还要求 `CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS`（`1..=1000`）、`CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS`（`1..=3600`）与 `CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS`（`1..=100000`）。authentication 完成后，有界 in-process sliding window 使用认证后的 `PrincipalIdentity` 与 protected route operation 作为 key：本地 lifecycle state read 使用 `ContextLifecycleRead`，guarded commit 与本地 lifecycle mutation 使用 `ContextCommitWrite`。它不按 Context id 分桶。quota rejection 返回带整数 `Retry-After` 的 `429 rate_limit_exceeded`；容量耗尽或 limiter 内部故障则 fail closed 为 `503 rate_limit_unavailable`。limiter rejection 不会进入 authorization audit 或 commit writer。

Authentication therefore runs before quota accounting. This private limiter does not change public OpenAPI, the TypeScript SDK, or GraphDiff. Migration `0010_shared_protected_route_rate_limits.sql` adds a private PostgreSQL adapter that shares the same auth port across replicas: one transaction-scoped advisory lock serializes policy validation, clock observation, expiry reclamation, capacity accounting, and one issuer/subject/operation timestamp queue update. It remains unwired to the protected HTTP route, so the in-process limiter remains the current runtime default; this storage evidence is not public-write readiness. Protected OIDC may additionally enable one configured top-level group claim only when `CONTEXTLAB_OIDC_GROUPS_CLAIM` and `CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS` (`1..=3600`) are both set. It accepts only a verified array of bounded opaque group IDs, requires `iss` and `aud` claims that exactly match the configured issuer and audience (including a standards-compliant audience array), rejects a future `nbf` without clock-skew leeway, requires a non-future `iat`, and rejects an expiry window exceeding the configured maximum. JWKS refresh is single-flight: a cached known key does not wait on an unknown-`kid` refresh, while unknown or failed refresh attempts enter a bounded one-second per-process cooldown without storing attacker-supplied key IDs. HMAC principals always have no external groups. Migration `0008_context_authorization_audit_governance.sql` adds private storage-only retention policy revisions, per-workspace active policy scopes, hold-by-default event disposition, and append-only purge-selection manifests. `contextlab-storage` also has a private redacted review repository, but no operator transport or public REST, OpenAPI, SDK, Web, or GraphDiff operation. OIDC discovery, public group-binding management, remote CI evidence, an operator-approved production change rehearsal, and public write-route promotion remain open; tests use in-memory JWKS sources and ephemeral keys and never contact a production identity provider.

因此 authentication 先于 quota accounting 执行。该 private limiter 不改变 public OpenAPI、TypeScript SDK 或 GraphDiff。迁移 `0010_shared_protected_route_rate_limits.sql` 增加了一个私有 PostgreSQL adapter，可通过同一个 auth port 在多个副本之间共享：一个 transaction-scoped advisory lock 会串行化 policy validation、clock observation、expiry reclamation、capacity accounting 与单个 issuer/subject/operation timestamp queue update。它尚未接入 protected HTTP route，因此 in-process limiter 仍是当前 runtime default；这份 storage evidence 不等于 public-write readiness。protected OIDC 只有在同时设置 `CONTEXTLAB_OIDC_GROUPS_CLAIM` 和 `CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS`（`1..=3600`）时，才会额外启用一个配置好的顶层 group claim。它只接受已验证的、由有界不透明 group ID 组成的 array，要求 `iss` 与 `aud` claim 精确匹配已配置的 issuer 与 audience（包括符合标准的 audience array），以零时钟偏差拒绝未来 `nbf`，要求 `iat` 不晚于当前时间，并拒绝超过配置上限的 expiry window。JWKS refresh 为 single-flight：缓存中已知的 key 不会等待未知 `kid` 的 refresh；未知或失败 refresh 会进入有界的一秒钟进程内冷却，且不保存攻击者提供的 key ID。HMAC principal 始终没有 external group。迁移 `0008_context_authorization_audit_governance.sql` 增加了私有、仅存储层的留存策略修订、按 workspace 的 active policy scope、默认 `hold` 的事件 disposition 与追加式 purge-selection manifest。`contextlab-storage` 也具备私有 redacted review repository，但没有 operator transport，也不新增 public REST、OpenAPI、SDK、Web 或 GraphDiff operation。OIDC discovery、public group-binding management、远端 CI evidence、operator 批准的 production change rehearsal 与 public write-route promotion 仍是开放项；测试使用内存 JWKS source 与临时 key，绝不连接 production identity provider。

**Shared-limiter hardening / 共享限流硬化。** Migration `0011_shared_protected_route_rate_limit_hardening.sql` supersedes the broad-lock mechanics described above: every request takes a deterministic per-key advisory lock, while the existing global lock is used only when a previously unseen key needs expiry reclamation, active-key capacity accounting, and insertion. The persisted policy is immutable after its first successful insertion; a different replica policy fails closed and requires an operator-approved deployment change rather than automatic rotation. Future or unordered stored timestamps also fail closed. This remains a private storage adapter, not a public-write promotion.

迁移 `0011_shared_protected_route_rate_limit_hardening.sql` 覆盖上文的宽泛锁机制：每个请求都取得确定性的 per-key advisory lock，现有 global lock 只在此前未见 key 需要 expiry reclamation、active-key capacity accounting 与 insertion 时使用。持久化 policy 在首次成功插入后保持不可变；不同 replica policy 会 fail closed，并需要 operator 批准的 deployment change，而不是自动 rotation。future 或无序的存储 timestamp 也会 fail closed。这仍是私有 storage adapter，不是 public-write promotion。

Workspace membership resolution is separate from the Context authorization policy: PostgreSQL resolves an active direct `owner`, `editor`, or `reader` role only for the exact identity-source and subject pair. When no direct role exists, it may resolve active `reader` or `editor` bindings in `workspace_external_group_role_bindings` for the principal's verified OIDC groups and exact identity source. `contextlab-auth` applies the reusable role-to-permission policy: a direct role always overrides group roles, and group bindings can never grant `owner`. Migration `0006_principal_identity_namespace.sql` retains existing rows under the explicit `legacy` source sentinel; operators must assign a real trusted source before relying on new identity isolation. This changes only the private protected path and does not add public group-management or commit-write contracts.

workspace membership resolution 与 Context authorization policy 已分离：PostgreSQL 会先为精确匹配的 identity-source 与 subject 组合解析 active 的 direct `owner`、`editor` 或 `reader` role。direct role 不存在时，它才会为 principal 的已验证 OIDC group 与精确 identity source 解析 `workspace_external_group_role_bindings` 中 active 的 `reader` 或 `editor` binding。`contextlab-auth` 负责应用可复用的 role-to-permission policy：direct role 永远覆盖 group role，group binding 永远不能授予 `owner`。迁移 `0006_principal_identity_namespace.sql` 会把既有行保留在显式 `legacy` source sentinel 下；operator 必须在依赖新的身份隔离前为其分配真实且可信的 source。这只改变 private protected path，不会新增 public group-management 或 commit-write contract。

Workspace discovery uses the same runtime repository selection. `GET /api/v1/workspaces` accepts `page`, `per_page`, `search`, and `sort`; supported sort values are `name`, `-name`, `created_at`, and `-created_at`. Responses include `items[]` with `id`, `name`, `slug`, and `created_at`, plus `pagination`.

Workspace discovery 使用同一套运行时 repository 选择。`GET /api/v1/workspaces` 支持 `page`、`per_page`、`search` 和 `sort`；可用排序值为 `name`、`-name`、`created_at` 和 `-created_at`。响应包含带 `id`、`name`、`slug`、`created_at` 的 `items[]`，以及 `pagination`。

Project discovery is workspace-scoped. `GET /api/v1/workspaces/{workspace_id}/projects` accepts the same `page`, `per_page`, `search`, and `sort` query parameters. Project items include `id`, `workspace_id`, `name`, `slug`, and `created_at`.

Project discovery 以 workspace 为作用域。`GET /api/v1/workspaces/{workspace_id}/projects` 支持同样的 `page`、`per_page`、`search` 和 `sort` query 参数。Project item 包含 `id`、`workspace_id`、`name`、`slug` 和 `created_at`。

Experiment discovery is project-scoped. `GET /api/v1/projects/{project_id}/experiments` accepts `page`, `per_page`, `search`, and `sort`; supported sort values are `name`, `-name`, `branch_name`, `-branch_name`, `created_at`, and `-created_at`. Experiment items include `id`, `project_id`, `name`, `branch_name`, and `created_at`.

Experiment discovery 以 project 为作用域。`GET /api/v1/projects/{project_id}/experiments` 支持 `page`、`per_page`、`search` 和 `sort`；可用排序值为 `name`、`-name`、`branch_name`、`-branch_name`、`created_at` 和 `-created_at`。Experiment item 包含 `id`、`project_id`、`name`、`branch_name` 和 `created_at`。

Context discovery is project-scoped. `GET /api/v1/projects/{project_id}/contexts` accepts `page`, `per_page`, `search`, `experiment_id`, and `sort`; supported sort values are `name`, `-name`, `created_at`, and `-created_at`. Context items include `id`, `project_id`, `experiment_id`, `name`, `description`, and `created_at`.

Context discovery 以 project 为作用域。`GET /api/v1/projects/{project_id}/contexts` 支持 `page`、`per_page`、`search`、`experiment_id` 和 `sort`；可用排序值为 `name`、`-name`、`created_at` 和 `-created_at`。Context item 包含 `id`、`project_id`、`experiment_id`、`name`、`description` 和 `created_at`。

Commit discovery is context-scoped. `GET /api/v1/contexts/{context_id}/commits` accepts `page`, `per_page`, `search`, `branch_name`, and `sort`; supported sort values are `authored_at`, `-authored_at`, `created_at`, `-created_at`, `branch_name`, and `-branch_name`. Commit items include `id`, `context_id`, `branch_name`, `message`, ordered `parent_commit_ids`, `change_count`, `authored_at`, and `created_at`.

Commit discovery 以 context 为作用域。`GET /api/v1/contexts/{context_id}/commits` 支持 `page`、`per_page`、`search`、`branch_name` 和 `sort`；可用排序值为 `authored_at`、`-authored_at`、`created_at`、`-created_at`、`branch_name` 和 `-branch_name`。Commit item 包含 `id`、`context_id`、`branch_name`、`message`、有序 `parent_commit_ids`、`change_count`、`authored_at` 和 `created_at`。

Component discovery is context-scoped. `GET /api/v1/contexts/{context_id}/components` accepts `page`, `per_page`, `search`, `kind`, and `sort`; supported sort values are `name`, `-name`, `kind`, `-kind`, `created_at`, and `-created_at`. Component items include `id`, `context_id`, `kind`, `name`, `content_hash`, and `created_at`.

Component discovery 以 context 为作用域。`GET /api/v1/contexts/{context_id}/components` 支持 `page`、`per_page`、`search`、`kind` 和 `sort`；可用排序值为 `name`、`-name`、`kind`、`-kind`、`created_at` 和 `-created_at`。Component item 包含 `id`、`context_id`、`kind`、`name`、`content_hash` 和 `created_at`。

Component detail is available at `GET /api/v1/contexts/{context_id}/components/{component_id}`. It returns the list fields plus `metadata` and `updated_at`; bodies remain unavailable through the public read surface. Private guarded storage can atomically capture one immutable UTF-8 body revision for an existing component, bound to its commit with prior/resulting SHA-256 fingerprints. Historical components without a captured revision remain body-unavailable.

Component detail 可通过 `GET /api/v1/contexts/{context_id}/components/{component_id}` 获取。它返回 list 字段，并额外包含 `metadata` 与 `updated_at`；public read surface 仍不提供正文。私有 guarded storage 可为既有 component 原子捕获一条不可变 UTF-8 正文 revision，并通过前后 SHA-256 fingerprint 绑定到对应 commit。没有已捕获 revision 的历史 component 仍不可取得正文。

Evaluation run discovery is context-scoped. `GET /api/v1/contexts/{context_id}/evaluation-runs` accepts `page`, `per_page`, `search`, `suite_name`, `model_version`, and `sort`; supported sort values are `executed_at`, `-executed_at`, `created_at`, `-created_at`, `suite_name`, `-suite_name`, `model_version`, and `-model_version`. Evaluation run items include `id`, `context_id`, `suite_name`, `model_version`, `temperature`, `metric_count`, `executed_at`, and `created_at`; full metrics remain reserved for detail reads.

Evaluation run discovery 以 context 为作用域。`GET /api/v1/contexts/{context_id}/evaluation-runs` 支持 `page`、`per_page`、`search`、`suite_name`、`model_version` 和 `sort`；可用排序值为 `executed_at`、`-executed_at`、`created_at`、`-created_at`、`suite_name`、`-suite_name`、`model_version` 和 `-model_version`。Evaluation run item 包含 `id`、`context_id`、`suite_name`、`model_version`、`temperature`、`metric_count`、`executed_at` 和 `created_at`；完整 metrics 留给 detail read。

Evaluation run detail is available at `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}`. It returns the list fields plus persisted `metrics` JSON so the scorecard endpoint, regression dashboards, and evaluation diff workflows can use the raw stored payload.

Evaluation run detail 可通过 `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}` 获取。它返回 list 字段，并额外包含持久化的 `metrics` JSON，使 scorecard endpoint、regression dashboard 与 evaluation diff workflow 可以使用原始存储 payload。

Evaluation scorecards are available at `GET /api/v1/contexts/{context_id}/evaluation-scorecard`. The endpoint accepts `search`, `suite_name`, and `model_version`, then returns `{ context_id, run_count, metrics }` where each metric contains a numeric average and sample count. It does not derive pass/fail decisions or regression conclusions.

Evaluation scorecard 可通过 `GET /api/v1/contexts/{context_id}/evaluation-scorecard` 获取。该 endpoint 支持 `search`、`suite_name` 和 `model_version`，并返回 `{ context_id, run_count, metrics }`；每个 metric 包含 numeric average 与 sample count。它不推导 pass/fail decision 或 regression conclusion。

The in-memory preview repository uses readable fixture ids such as `default`. PostgreSQL mode validates workspace ids as UUIDs before querying, so `default` is only a preview fixture id, not a production identifier contract.

In-memory preview repository 使用 `default` 这类易读 fixture id。PostgreSQL 模式会在查询前把 workspace id 校验为 UUID，因此 `default` 只是 preview fixture id，不是生产标识符契约。

The same preview/production distinction applies to project ids such as `support-ai` and context ids such as `support-resolution-agent`. PostgreSQL mode validates project ids as UUIDs before experiment and context queries, validates context ids as UUIDs before commit, component, evaluation run, and evaluation scorecard queries, and validates `experiment_id` context filters as UUIDs when present.

同样的 preview/production 区别也适用于 `support-ai` 这类 project id 和 `support-resolution-agent` 这类 context id。PostgreSQL 模式会在 experiment 与 context query 前把 project id 校验为 UUID，在 commit、component、evaluation run 与 evaluation scorecard query 前把 context id 校验为 UUID，并在提供 `experiment_id` context filter 时把它校验为 UUID。

PostgreSQL integration verification is opt-in and should use an empty disposable database:

PostgreSQL 集成验证是 opt-in，应使用空的、可丢弃的测试数据库：

```bash
CONTEXTLAB_TEST_DATABASE_URL=postgres://contextlab:contextlab@localhost/contextlab_test \
  cargo test -p contextlab-storage projects_seed_workspace_graph_from_postgres -- --ignored --exact
```

The web app starts on `http://localhost:3000` by default.

Web 应用默认运行在 `http://localhost:3000`。

Run the visual workspace verification in a second terminal while the Web dev server is running.

运行 visual workspace verification 时，请保持 Web dev server 在另一个终端中运行。

The current web shell is a design-system-first Context detail workspace. It imports shared DTOs from `@contextlab/ts-sdk` and renders local API-shaped data from `apps/web/src/app/context-workspace-preview.ts` by default, including workspace Context Graph nodes, component inventory fingerprints, selected component metadata detail, evaluation run discovery/detail, selected scorecard averages scoped to the chosen run's suite/model filters, and a read-only persisted commit graph review, so the interface can evolve against stable REST contracts before full generated SDKs are introduced.

当前 Web shell 是 design-system-first 的 Context detail workspace。它从 `@contextlab/ts-sdk` 引入共享 DTO，并默认从 `apps/web/src/app/context-workspace-preview.ts` 渲染本地 API-shaped data，包含 workspace Context Graph node、component inventory fingerprint、selected component metadata detail、evaluation run discovery/detail、按选中 run 的 suite/model filter 收敛的 selected scorecard average，以及只读的持久化 commit 图谱审阅；界面可先围绕稳定 REST contract 演进，再接入完整 generated SDK。

Set `CONTEXTLAB_WEB_API_BASE_URL` to a running API origin, such as `http://127.0.0.1:3100`, to let the Web shell load live discovery data at runtime. If the API request fails, the page falls back to deterministic preview data.

将 `CONTEXTLAB_WEB_API_BASE_URL` 设置为正在运行的 API origin（例如 `http://127.0.0.1:3100`）后，Web shell 会在运行时加载 live discovery data。如果 API 请求失败，页面会回退到确定性的 preview data。
