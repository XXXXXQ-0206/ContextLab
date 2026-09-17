# Security Policy / 安全策略

## Reporting a vulnerability / 报告漏洞

Report suspected vulnerabilities privately through GitHub Security Advisories on this repository
("Security" → "Report a vulnerability"). Please do not open a public issue for a flaw that could be
exploited before a fix is available.

请通过本仓库的 GitHub Security Advisories（"Security" → "Report a vulnerability"）私下报告可疑漏洞。在修复可用之前，
请勿为可能被利用的缺陷提交 public issue。

Include, when you can: the affected component and version, the exact command or request that
reproduces the issue, the observed and expected behaviour, and any mitigation you already tested.
Remove real credentials, database URLs, tokens, JWT claims, and personal data from the report.

请尽量包含：受影响的组件与版本、可复现问题的确切 command 或 request、实际与预期行为，以及你已测试过的缓解措施。
请在报告中移除真实凭据、database URL、token、JWT claim 与个人数据。

## Supported scope / 适用范围

The project is currently a pre-release local engineering platform. Security-relevant surfaces are:

- the protected route mode in `server/api`, including principal authentication, Context
  authorization, protected-route rate limiting, and the fail-closed behaviour when configuration is
  missing or invalid;
- the private local SDK, same-origin Web BFF routes, and the Web inspectors that forward a
  request-scoped Bearer credential with cookies omitted and `cache: no-store`;
- PostgreSQL migrations, the guarded commit writer, audit-retention governance, and the restricted
  purge executor;
- anything that could leak a credential, a raw Context body, or an upstream diagnostic message into
  a response, a log, a test fixture, or committed documentation.

本项目当前是 pre-release 本地工程平台。与安全相关的 surface 包括：

- `server/api` 的 protected route mode，含 principal authentication、Context authorization、
  protected-route rate limiting，以及配置缺失或非法时的 fail-closed 行为；
- 私有 local SDK、同源 Web BFF route，以及以 request-scoped Bearer、cookie omission 与 `cache: no-store`
  转发凭据的 Web inspector；
- PostgreSQL migration、guarded commit writer、audit-retention governance 与受限 purge executor；
- 任何可能把凭据、Context 正文或 upstream diagnostic 泄漏到 response、log、test fixture 或已提交文档的路径。

## Known non-goals / 已知非目标

The repository does not yet claim production readiness. Public protected-write promotion, release,
and production rollout remain deferred until their external conditions and receipts exist. Reports
that a preview fixture id does not exist in PostgreSQL mode, or that a public read surface
deliberately omits component bodies, describe intended behaviour rather than vulnerabilities.

本仓库尚未声称 production readiness。在外部条件与回执具备之前，public protected-write promotion、release 与
production rollout 仍然延期。关于"preview fixture id 在 PostgreSQL 模式下不存在"或"public read surface 有意不提供
component 正文"的报告描述的是既定行为，而不是漏洞。

## Disclosure / 披露

We aim to acknowledge a report within 7 days and to agree on a disclosure timeline with the
reporter. Credit is given in the advisory unless the reporter prefers otherwise.

我们计划在 7 天内确认收到报告，并与报告者商定披露时间线。除报告者另有偏好外，我们会在 advisory 中致谢。
