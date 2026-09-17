# OIDC Cache Concurrency Hardening Implementation Plan / OIDC 缓存并发加固实施计划

> **For agentic workers / 面向智能体执行者：** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` for isolated implementation and security review. Steps use checkbox syntax for traceable delivery.

**Goal / 目标：** Keep private protected OIDC fail-closed while requiring registered claims, accepting standards-compliant audiences, and preventing JWKS refresh traffic from serializing cached-key authentication or permanently blocking a legitimate key rotation.

**Architecture / 架构：** `OidcJwksAuthenticator` owns one in-process state machine. Its short mutex critical sections inspect or update cache state only; all JWKS network I/O happens after the guard is dropped. A fresh cached key may authenticate while a separate unknown-`kid` refresh is in flight. Unknown keys receive a bounded per-process refresh cooldown, while an expired or absent cache remains fail-closed. This private runtime change adds no route, OpenAPI operation, SDK method, or GraphDiff behavior.

**Tech Stack / 技术栈：** Rust stable, Tokio, `jsonwebtoken` 9.3.1, Serde JSON, Cargo tests.

---

### Task 1: Lock down OIDC token semantics / 收紧 OIDC token 语义

**Files / 文件：**
- Modify / 修改：`crates/auth/src/oidc.rs`
- Test / 测试：`crates/auth/src/oidc.rs`

- [x] Write a signed-token regression that omits `iss` or `aud`; confirm it fails before the production change.
- [x] Set `Validation::set_required_spec_claims(&["exp", "iss", "aud"])` before decoding, retain exact issuer/audience matching, and rerun the regression to green.
- [x] Write a signed token with future `nbf`; confirm it fails before enabling `validate_nbf` and setting `leeway = 0`, then rerun it to green.
- [x] Write a standards-compliant audience-array positive test, preserve `aud` as a required `serde_json::Value`, and verify the configured client inside the array authenticates.

### Task 2: Specify non-blocking JWKS cache state / 定义不阻塞的 JWKS 缓存状态

**Files / 文件：**
- Modify / 修改：`crates/auth/src/oidc.rs`
- Test / 测试：`crates/auth/src/oidc.rs`

- [x] Write a controlled `JwksSource` test where an unknown `kid` starts a blocked refresh and a second request signed by a still-cached known key completes before the source is released.
- [x] Write a recovery test where a forced refresh fails, the cooldown rejects an immediate retry without another fetch, and a post-cooldown rotation fetch can install a JWKS containing the new key.
- [x] Replace `Mutex<Option<CachedJwks>>` with a cache state that has `cached`, `refresh_in_flight`, and `unknown_kid_refresh_not_before` fields. The state guard must never surround `.await`:

```rust
struct JwksCacheState {
    cached: Option<CachedJwks>,
    refresh_in_flight: bool,
    unknown_kid_refresh_not_before: Option<Instant>,
}
```

- [x] For a fresh cached matching key, derive `DecodingKey` while locked and return it before any refresh decision. For an unknown key, set `refresh_in_flight` and a bounded cooldown, drop the guard, fetch, then reacquire to install a successful document or clear only the in-flight marker after failure.
- [x] For an expired or absent cache, allow one request to fetch while other requests fail closed immediately; never authenticate from stale keys and never wait on the state lock while an HTTP request is pending.

### Task 3: Preserve deterministic refresh behavior / 保持确定性刷新行为

**Files / 文件：**
- Modify / 修改：`crates/auth/src/oidc.rs`
- Test / 测试：`crates/auth/src/oidc.rs`

- [x] Keep the existing successful key-rotation test: first cached document contains `current`, one unknown-`kid` refresh obtains `rotated`, and the successful refresh count is exactly two.
- [x] Keep failures at `AuthenticationError::InvalidToken`; do not expose JWKS URLs, header values, identity values, cache state, or source errors.
- [x] Keep cooldown state bounded without storing attacker-supplied `kid` values; cache expiry naturally clears the document and accepts one new bootstrap fetch.

### Task 4: Document and verify the private boundary / 记录并验证 private 边界

**Files / 文件：**
- Modify / 修改：`README.md`, `docs/superpowers/specs/2026-07-11-trusted-identity-group-rbac-design.md`, `docs/superpowers/plans/2026-07-12-trusted-identity-group-rbac.md`

- [x] State required claims, exact audience behavior, `nbf` clock policy, unknown-`kid` cooldown semantics, and the continued absence of public write/API/SDK changes in English and Chinese.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, and `python apps/web/verify-context-workspace.py` through a disposable local server lifecycle.
- [x] Record Docker unavailability or run the ignored disposable PostgreSQL tests; never read `.env` or connect to an unspecified database.

## Verification Evidence / 验证证据

- [x] `2026-07-13`: `cargo fmt --all -- --check` and `cargo test --workspace` passed; the storage suite reported 114 passed and 9 explicitly ignored disposable-PostgreSQL tests.
- [x] `2026-07-13`：`cargo fmt --all -- --check` 与 `cargo test --workspace` 通过；storage suite 为 114 passed，9 个 disposable-PostgreSQL 测试保持显式 ignored。
- [x] `2026-07-13`: `pnpm check:web` passed lint, tests, and the production build; `verify-context-workspace.py` passed for desktop and mobile through a disposable local server lifecycle.
- [x] `2026-07-13`：`pnpm check:web` 的 lint、test 与 production build 均通过；`verify-context-workspace.py` 通过一次性本地 server lifecycle 完成 desktop 与 mobile 验证。
- [x] `2026-07-13`: Docker Desktop's Linux-engine pipe was unavailable, so no database URL, `.env`, or existing database was read; disposable PostgreSQL and production-migration evidence remain release gates.
- [x] `2026-07-13`：Docker Desktop 的 Linux-engine pipe 不可用，因此没有读取 database URL、`.env` 或既有数据库；disposable PostgreSQL 与 production migration evidence 仍是 release gate。
