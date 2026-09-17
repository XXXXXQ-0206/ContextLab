# OIDC JWKS Authentication Design / OIDC JWKS 认证设计

## Goal / 目标

Add a replaceable OIDC JWT verifier for the existing private protected runtime profile. It verifies only RS256 tokens against a configured HTTPS JWKS endpoint, validates issuer and audience, refreshes keys safely on rotation, and fails closed without adding a public write contract.

为既有 private protected runtime profile 增加可替换的 OIDC JWT verifier。它仅使用配置的 HTTPS JWKS endpoint 验证 RS256 token，校验 issuer 与 audience，在密钥轮换时安全刷新，并在不增加 public write contract 的前提下 fail closed。

## Decision / 决策

Authentication becomes an async port because an unknown `kid` can require one network refresh and Axum middleware is already async. `PrincipalAuthenticator::authenticate_authorization_header` changes to an async method. HMAC remains a zero-I/O implementation of the same port.

认证将改为 async port，因为未知 `kid` 可能需要一次网络刷新，而 Axum middleware 已经是 async。`PrincipalAuthenticator::authenticate_authorization_header` 改为 async method。HMAC 仍作为同一 port 的 zero-I/O 实现。

`contextlab-auth` defines a `JwksSource` port returning a parsed `jsonwebtoken::jwk::JwkSet`; the HTTP adapter is private infrastructure inside the same crate and uses a minimal HTTPS-only client with an explicit response-size limit and timeout. Tests use an in-memory source, never a real identity provider.

`contextlab-auth` 定义返回已解析 `jsonwebtoken::jwk::JwkSet` 的 `JwksSource` port；HTTP adapter 是同一 crate 内的 private infrastructure，使用最小 HTTPS-only client、明确 response-size limit 与 timeout。test 使用 in-memory source，绝不连接真实 identity provider。

## Key and Cache Policy / 密钥与缓存策略

- Accept only JWT header algorithm `RS256`; reject HS, PS, ES, EdDSA, and missing algorithms.
- Require a non-empty `kid`; select exactly one JWK with that key id.
- Accept only RSA JWKs where `use` is absent or `sig`, and where `alg` is absent or `RS256`.
- Use `DecodingKey::from_jwk` and `Validation::new(Algorithm::RS256)` with required issuer and audience.
- The first request fetches keys. A valid cache is used until its bounded TTL. An unknown `kid` triggers one mutex-protected forced refresh before rejection, so concurrent misses do not stampede the provider.
- Fetch failures and expired caches reject authentication. The verifier does not serve stale keys after expiry, because an old key may have been revoked.
- Cache TTL uses a configured bounded duration; HTTP cache headers may reduce it later but do not extend it beyond the configured maximum in this increment.

- 只接受 JWT header algorithm `RS256`；拒绝 HS、PS、ES、EdDSA 与缺失 algorithm。
- 要求非空 `kid`；必须只选中一个同 key id 的 JWK。
- 只接受 RSA JWK，且 `use` 为缺失或 `sig`，`alg` 为缺失或 `RS256`。
- 使用 `DecodingKey::from_jwk` 与 `Validation::new(Algorithm::RS256)`，并强制 issuer 与 audience。
- 首个请求获取 key。缓存有效期内直接使用；未知 `kid` 会触发一次 mutex-protected forced refresh，再决定拒绝，避免并发 miss 冲击 provider。
- fetch failure 与 cache expiry 均拒绝认证。verifier 在 expiry 后不使用 stale key，因为旧 key 可能已被撤销。
- cache TTL 使用有界配置时长；HTTP cache header 可在后续缩短 TTL，但本增量不得把 TTL 延长到配置上限之外。

## Runtime Configuration / 运行时配置

`CONTEXTLAB_API_ROUTE_MODE=protected` gains `CONTEXTLAB_AUTH_MODE` with `hmac` or `oidc`. `hmac` preserves the existing secret profile. `oidc` requires `CONTEXTLAB_OIDC_ISSUER`, `CONTEXTLAB_OIDC_AUDIENCE`, `CONTEXTLAB_OIDC_JWKS_URL`, and a bounded cache TTL setting. The builder rejects OIDC mode without PostgreSQL, a HTTPS JWKS URL, or complete issuer/audience settings. Errors name missing variables or invalid modes only, never values.

`CONTEXTLAB_API_ROUTE_MODE=protected` 增加 `CONTEXTLAB_AUTH_MODE`，可选 `hmac` 或 `oidc`。`hmac` 保留既有 secret profile。`oidc` 要求 `CONTEXTLAB_OIDC_ISSUER`、`CONTEXTLAB_OIDC_AUDIENCE`、`CONTEXTLAB_OIDC_JWKS_URL` 与有界 cache TTL 设置。builder 拒绝没有 PostgreSQL、非 HTTPS JWKS URL 或 issuer/audience 不完整的 OIDC mode。error 只描述缺失变量或非法 mode，绝不包含具体值。

## Non-Goals / 非目标

No OIDC discovery document fetch, client login flow, token issuance, group/claim-to-role mapping, public route promotion, rate limiting, or audit retention policy. Those are separate increments.

不实现 OIDC discovery document fetch、client login flow、token issuance、group/claim-to-role mapping、public route promotion、rate limiting 或 audit retention policy。它们属于独立增量。

## Verification / 验证

1. Unit tests cover JWK selection, algorithm rejection, issuer/audience validation, empty or duplicate `kid`, expiry, and one forced refresh.
2. Source doubles prove a matching cached key avoids I/O, an unknown `kid` performs one refresh, and fetch failure never authenticates with expired cache.
3. Runtime configuration tests cover `hmac` compatibility, invalid auth mode, missing OIDC values, and HTTP JWKS URL rejection.
4. Public router/OpenAPI/SDK tests remain unchanged; no authenticated token, provider credential, or production IdP is used in tests.

1. unit test 覆盖 JWK selection、algorithm rejection、issuer/audience validation、空或重复 `kid`、expiry 与一次 forced refresh。
2. source double 证明匹配的 cache key 不发生 I/O，未知 `kid` 只刷新一次，fetch failure 绝不使用 expired cache 完成认证。
3. runtime configuration test 覆盖 `hmac` compatibility、非法 auth mode、缺失 OIDC 值与 HTTP JWKS URL rejection。
4. public router/OpenAPI/SDK test 保持不变；test 不使用 authenticated token、provider credential 或 production IdP。
