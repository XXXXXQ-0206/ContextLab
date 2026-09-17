# Private Lifecycle Read Transport Receipt / 私有生命周期读取传输回执

## Necessity Record / 必要性记录

### Criterion and charter principle / 对应条件与宪章原则

This increment directly advances Criterion 1 and the charter's secure-by-default, Context-first
principles. A selected exact Context commit is only a dependable product boundary when its protected
read transport preserves scope, request-memory authentication, cache privacy, and typed availability
states.

本增量直接推进条件 1，并落实宪章的 secure-by-default 与 Context-first 原则。选定的精确 Context commit 只有在
受保护读取传输保持 scope、请求内存认证、缓存隐私与 typed availability state 时，才是可靠的产品边界。

### Evidence gap and dependency / 证据缺口与依赖

The protected lifecycle route, BFF, local SDK parser, and read-only inspector already exist and have
passed rendered Web tests. Independent review found that the browser-side lifecycle loader does not
explicitly set `cache: "no-store"`, and the current focused evidence does not separately pin BFF
cookie omission/private no-store headers, 503-to-unavailable mapping, or the inspector unavailable
state.

受保护 lifecycle route、BFF、local SDK parser 与只读 inspector 已存在并通过渲染 Web 测试。独立审查发现浏览器端
lifecycle loader 没有显式设置 `cache: "no-store"`，当前聚焦证据也没有单独固定 BFF 的 cookie omission/private no-store
header、503 到 unavailable 的映射或 inspector unavailable state。

### Why now / 为什么现在

The transport boundary is dependency-ready and is the smallest remaining local proof adjacent to
the newly completed Inspector. Fixing it now prevents a future reader from relying on implicit cache
behavior and turns an observed review gap into a reusable contract receipt without expanding product
surface.

传输边界依赖已满足，并且是刚完成 Inspector 旁边最小的本地证明缺口。现在固定它可以避免后续 reader 依赖隐式缓存行为，
并在不扩大产品表面的前提下，把已发现的审查缺口转化为可复用的 contract receipt。

### Minimal boundary / 最小受影响边界

- Explicit `cache: "no-store"` in the existing lifecycle read loader only.
- Focused tests for the existing BFF/proxy, loader, SDK scope rejection, and Inspector unavailable/a11y state.
- Preserve exact `(Context, commit)` identity, Bearer-only request-memory credentials, `credentials: "omit"`, and `private, no-store` response semantics.

- 仅在现有 lifecycle read loader 中显式设置 `cache: "no-store"`。
- 为现有 BFF/proxy、loader、SDK scope rejection 与 Inspector unavailable/a11y state 增加聚焦测试。
- 保持精确 `(Context, commit)` identity、仅 Bearer 的请求内存凭据、`credentials: "omit"` 与 `private, no-store` response 语义。

### Explicit non-goals / 明确非目标

No Rust/storage/migration changes, new API route, OpenAPI/public SDK method, Web mutation, operator
transport, PostgreSQL runtime, authenticated browser, visual smoke, provider, secrets, release,
production, remote CI, or second GraphDiff calculator. Local tests must not be described as browser,
PostgreSQL, remote, operator, release, or production evidence.

本增量不包含 Rust/storage/migration 变更、新 API route、OpenAPI/public SDK method、Web mutation、operator transport、
PostgreSQL runtime、authenticated browser、visual smoke、provider、secret、release、production、remote CI 或第二个
GraphDiff calculator。Local tests 不得被描述为 browser、PostgreSQL、remote、operator、release 或 production evidence。

### Fresh verification before the next increment / 下一增量前的新鲜验证

Focused transport/proxy and Inspector tests must pass, followed by `pnpm check:web`, Rust format/test,
strict offline Clippy, locked Rust 1.85.0 check, and the local contract verifier. Evidence must retain
`passed`/`ignored`/`unobserved`/`deferred` boundaries; the increment advances Criterion 1 but does not
close it or the active long-term goal.

必须通过 transport/proxy 与 Inspector 聚焦测试，随后运行 `pnpm check:web`、Rust format/test、strict offline Clippy、
锁定 Rust 1.85.0 check 与 local contract verifier。证据必须保留 `passed`/`ignored`/`unobserved`/`deferred` 边界；本增量
仅推进条件 1，不关闭条件或 active 长期目标。

## Ownership / 文件归属

- Luna transport worker: existing lifecycle loader/proxy tests and the minimal loader cache option.
- Integration Lead: this record, bilingual roadmap receipt, final verification, and evidence boundary.

## Exit record / 退出记录

Completed locally. Focused lifecycle data/proxy/presenter/inspector tests passed (`41 passed`), the
full Web suite passed (`294 passed`), and `pnpm check:web` passed with public SDK `15`, local SDK
`135`, Web `294`, and production build. The loader now explicitly sets `cache: "no-store"`; BFF
tests pin Bearer-only, cookie omission, exact scope, `private, no-store`, and 503/unavailable
semantics. Rust format, workspace tests (`220 passed`, `41 ignored`), strict offline Clippy, locked
Rust `1.85.0`, and local contract checks passed. No external runtime or release receipt is inferred.

已在本地完成。lifecycle data/proxy/presenter/inspector 聚焦测试通过（`41 passed`）；完整 Web suite 通过
（`294 passed`）；`pnpm check:web` 通过，public SDK `15`、local SDK `135`、Web `294` 与 production build 均通过。
loader 现显式设置 `cache: "no-store"`；BFF 测试固定 Bearer-only、cookie omission、exact scope、`private, no-store`
与 503/unavailable 语义。Rust format、workspace tests（`220 passed`、`41 ignored`）、strict offline Clippy、锁定
Rust `1.85.0` 与 local contract checks 通过。不从本地工作推断任何 external runtime 或 release receipt。
