# Protected Local Commit Graph-Diff Read / 受保护的本地 Commit Graph-Diff 读取

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
advances Criterion 1 (Context-first platform coverage), Criterion 2 (replayable version history),
Criterion 4 (the Context Graph as a system backbone), and Criterion 6 (collaboration safety). A
version-backed graph diff returns immutable Context graph labels, relationships, commit existence,
and capture metadata. Those facts must follow the same authenticated, authorized, audited, rate-
limited, cache-safe read boundary as other private Context projections.

本增量直接推进条件 1（Context-first 平台覆盖）、条件 2（可回放版本历史）、条件 4（Context Graph 作为系统骨架）以及条件 6（协作安全）。version-backed graph diff 会返回不可变 Context 图的 label、关系、commit 存在性与 capture metadata。这些事实必须和其他 private Context projection 一样，经过认证、授权、审计、限流与缓存安全的读取边界。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The existing
`GET /api/v1/contexts/{context_id}/graph-diff` route is registered in the public catalog and its
handler resolves snapshots before any principal, `ContextPermission::Read`, audit decision, or
principal-operation quota. It can disclose private graph evolution and is not uniformly marked
`private, no-store`. The earlier exact-scope snapshot contract protects referential integrity, not
read confidentiality.

现有 `GET /api/v1/contexts/{context_id}/graph-diff` 路由注册在 public catalog 中；其 handler 在任何 principal、`ContextPermission::Read`、审计决策或 principal-operation quota 之前就解析 snapshot。它可能泄露 private graph evolution，并且没有统一标记 `private, no-store`。此前的 exact-scope snapshot contract 保护的是引用完整性，而不是读取保密性。

**Why now / 为什么现在优先:** The exact commit-snapshot repository, authenticated protected-read
middleware, RBAC/audit helpers, deterministic `GraphDiff::between`, local SDK credential pattern,
and same-origin Web BFF conventions already exist. Leaving this public metadata read in place would
undermine the local Context lifecycle and replay evidence that the prior increments established.

exact commit-snapshot repository、authenticated protected-read middleware、RBAC/audit helper、确定性的 `GraphDiff::between`、local SDK credential pattern 与 same-origin Web BFF convention 均已存在。若继续保留该 public metadata read，会破坏此前增量已建立的本地 Context lifecycle 与 replay evidence。

**Decision and minimal boundary / 决策与最小边界:** Retire the public alias and public SDK/OpenAPI
operation. Move the read to the default-off protected-local route
`GET /api/v1/local/contexts/{context_id}/graph-diff`. Authenticate and apply a dedicated
`ContextCommitGraphDiffRead` quota before the handler; authorize `ContextPermission::Read` and
record its decision before any repository lookup; apply `Cache-Control: private, no-store` to every
outcome. Preserve the query names, response projection, exact scope resolution, structured errors,
and the sole `GraphDiff::between` calculation. Expose the route only through the non-public local
SDK and a same-origin, Bearer-only BFF. The server-rendered public workspace must not substitute a
server credential; it reports graph diff availability as unavailable until a local authenticated
client requests it.

移除 public alias 与 public SDK/OpenAPI operation。将读取迁移到默认关闭的 protected-local route：
`GET /api/v1/local/contexts/{context_id}/graph-diff`。在 handler 前进行 authentication 与专用
`ContextCommitGraphDiffRead` quota；在任何 repository lookup 前执行 `ContextPermission::Read` authorization 并记录决策；对所有结果设置 `Cache-Control: private, no-store`。保持 query 名称、response projection、exact scope resolution、structured error 与唯一的 `GraphDiff::between` calculation。仅通过 non-public local SDK 与 same-origin、Bearer-only BFF 暴露该 route。server-rendered public workspace 不得替代用户使用 server credential；在 local authenticated client 发起请求前，它应将 graph diff 标为 unavailable。

**Explicit non-goals / 明确非目标:** No public REST route, public OpenAPI operation, public SDK
method, public write, graph editing, storage migration, provider call, operator transport, Docker,
release, or production claim. Do not change graph-diff semantics or introduce another calculator.

不新增 public REST route、public OpenAPI operation、public SDK method、public write、graph editing、storage migration、provider call、operator transport、Docker、release 或 production claim。不改变 graph-diff semantics，也不增加第二个 calculator。

**Ownership and bilingual documentation / 所有权与双语文档:** Server routing, authorization,
audit/rate-limit wiring, and Rust contracts own `server/api/**`; public-contract removal owns
`packages/ts-sdk/**` and `docs/api/openapi.json`; credentialed private client parsing owns
`packages/local-sdk/**`; Web BFF/data/presenter/screen adaptation owns `apps/web/src/app/**`. This
record and the roadmap ledgers are the bilingual documentation boundary. Each owner must not edit
another owner's files.

server routing、authorization、audit/rate-limit wiring 与 Rust contract 由 `server/api/**` 负责；public-contract 移除由 `packages/ts-sdk/**` 与 `docs/api/openapi.json` 负责；credentialed private client parsing 由 `packages/local-sdk/**` 负责；Web BFF/data/presenter/screen adaptation 由 `apps/web/src/app/**` 负责。本记录与 roadmap ledger 是双语文档边界。每位 owner 不得编辑其他 owner 的文件。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Red/green
tests must prove the public route is absent, protected unauthenticated requests do not consume quota
or touch the repository, denied reads audit and fail closed before snapshot access, allowed reads
preserve exact diff projection, every protected outcome is `private, no-store`, and a `POST` to the
read path is rejected. Local SDK/BFF tests must prove canonical scope validation, Bearer-only
forwarding, cookie omission, no server-side credential fallback, and redacted errors. Then run
formatting, focused Rust/TypeScript tests, workspace Rust tests, strict offline Clippy, and
`pnpm check:web`; record PostgreSQL runtime, browser, Git, remote, release, and production only as
actually observed.

红绿测试必须证明 public route 已不存在、protected 的未认证请求不消耗 quota 也不访问 repository、拒绝读取会在 snapshot access 前记录审计并 fail closed、允许读取保持 exact diff projection、每个 protected outcome 都是 `private, no-store`，且对 read path 的 `POST` 被拒绝。local SDK/BFF 测试必须证明 canonical scope validation、Bearer-only forwarding、cookie omission、无 server-side credential fallback 与 redacted error。随后运行 formatting、聚焦 Rust/TypeScript test、workspace Rust test、strict offline Clippy 与 `pnpm check:web`；PostgreSQL runtime、browser、Git、remote、release 与 production 仅按实际观测记录。

## Implementation Steps / 实施步骤

- [x] Add the protected-local server route, dedicated operation, authentication, authorization/audit,
  quota, cache policy, and regression tests.
- [x] Remove the public route, OpenAPI operation, and public SDK method with absence contracts.
- [x] Add the non-public local SDK parser/client method and same-origin Web BFF/data/screen adaptation.
- [x] Run the required fresh verification and record only observed evidence.

- [x] 增加 protected-local server route、专用 operation、authentication、authorization/audit、quota、cache policy 与回归测试。
- [x] 移除 public route、OpenAPI operation 与 public SDK method，并加入 absence contract。
- [x] 增加 non-public local SDK parser/client method 以及 same-origin Web BFF/data/screen adaptation。
- [x] 运行所需的新鲜验证，并且只记录已观测 evidence。

## Fresh Receipt / 新鲜回执

**Observed local commands and results / 已观测本地命令与结果:**

- `cargo test --workspace --quiet` passed. The Rust workspace included API `183 passed` and storage `179 passed, 39 ignored`; all other workspace test binaries also passed with `0 failed`.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo +1.85.0 check --workspace --all-targets --locked` passed.
- `pnpm check:web` passed: public SDK `15/15`, local SDK `92/92`, and Web `185/185`; TypeScript lint checks passed and the Next.js production build compiled successfully and generated static pages.

**已观测本地命令与结果：**

- `cargo test --workspace --quiet` 通过。Rust workspace 包含 API `183 passed` 与 storage `179 passed, 39 ignored`；其余 workspace test binary 也均为 `0 failed`。
- `cargo fmt --all -- --check` 通过。
- `cargo clippy --workspace --all-targets --offline -- -D warnings` 通过。
- `cargo +1.85.0 check --workspace --all-targets --locked` 通过。
- `pnpm check:web` 通过：public SDK `15/15`、local SDK `92/92`、Web `185/185`；TypeScript lint 通过，Next.js production build 成功编译并完成静态页面生成。

**Evidence boundary / 证据边界:** PostgreSQL runtime: `unobserved`; authenticated browser: `unobserved`; Git change-set: `unobserved`. Remote CI: `deferred`; operator rehearsal: `deferred`; release: `deferred`; production: `deferred`. No secrets were read. This is a fresh local implementation receipt only; it does not claim long-term goals, release readiness, or production completion.

**证据边界：** PostgreSQL runtime：`unobserved`；authenticated browser：`unobserved`；Git change-set：`unobserved`。remote CI：`deferred`；operator rehearsal：`deferred`；release：`deferred`；production：`deferred`。未读取 secrets。本回执仅证明本地实现验证，不声称长期目标、release readiness 或 production 已完成。
