# API Strict-Clippy Baseline / API 严格 Clippy 基线

## Necessity Record / 必要性记录

### Completion criterion and principle / 完成条件与原则

This increment serves Completion Criterion 8, reliable local quality gates, and the
charter principles of declared-MSRS compatibility, readable code, and maintainable
reusable infrastructure.

本增量服务于完成条件 8（可靠的本地质量门禁），以及项目宪章中已声明
MSRV 兼容性、可读代码和可维护的可复用基础设施原则。

### Evidence gap / 证据缺口

After the auth Rust 1.85 const-context repair and the storage strict-Clippy cleanup,
`cargo clippy -p contextlab-api --all-targets -- -D warnings` exposes the third
quality-gate root cause in the API crate: redundant enum prefixes, a constructor with
too many arguments, discarded `Router` builders, and test-only complex types.

在 auth 的 Rust 1.85 const-context 修复与 storage 严格 Clippy 清理之后，
`cargo clippy -p contextlab-api --all-targets -- -D warnings` 暴露了 API crate
的第三个质量门禁根因：冗余枚举前缀、参数过多的构造函数、被丢弃的
`Router` builder，以及仅测试使用的复杂类型。

### Why now / 为何现在

These warnings block a focused, dependency-ready quality gate across all API-backed
local capabilities. Their repair is behavior-preserving and smaller than starting a
new capability while a repeatable API lint signal remains unavailable.

这些告警阻断所有 API 支撑的本地能力的聚焦、依赖已满足的质量门禁。它们的
修复保持行为不变，并且比在可重复的 API lint 信号尚不可用时启动新能力更小。

### Minimal boundary / 最小边界

Only `server/api/src/lib.rs` and `server/api/src/routes.rs` are eligible for code
changes. The refactor may add or adapt API-library tests that preserve constructor
composition and route selection. It changes neither HTTP paths nor OpenAPI, public
SDK, RBAC, audit, rate-limit, writes, storage, Docker, or secrets.

只有 `server/api/src/lib.rs` 与 `server/api/src/routes.rs` 可以修改代码。本次
重构可新增或调整 API library 测试以保持构造组合和路由选择。它不会改变 HTTP
路径、OpenAPI、公开 SDK、RBAC、审计、限流、写入、storage、Docker 或密钥。

### Non-goals / 非目标

- No lint allowances, MSRV increase, or broad API redesign.
- No public route, write surface, provider, database, or deployment work.
- No changes outside this record and the two owned API source files.

- 不新增 lint allow、不提高 MSRV、不进行广泛 API 重设计。
- 不新增公开路由、写入面、provider、数据库或部署工作。
- 不修改本记录和两个负责的 API 源文件以外的文件。

### Fresh verification / 新鲜验证

Before this increment is recorded as passed, observe command output from:

在将本增量记录为通过前，必须观察以下命令输出：

```text
cargo fmt --all -- --check
cargo test -p contextlab-api --lib --quiet
cargo clippy -p contextlab-api --all-targets -- -D warnings
```

## Observed evidence / 已观测证据

The focused regression first failed to compile because the typed repository input did
not exist. After the minimal grouping implementation, it passed with `1 passed` and
confirmed that the public workspace read route remains selectable.

聚焦回归最初因 typed repository 输入尚不存在而无法编译。完成最小分组实现后，
它以 `1 passed` 通过，并确认公开工作区读取路由仍可被选择。

Observed on 2026-07-23 / 于 2026-07-23 观察到：

```text
cargo fmt --all -- --check
passed

cargo fmt --all
passed

cargo test -p contextlab-api --lib --quiet
154 passed; 0 failed; 0 ignored

cargo clippy -p contextlab-api --all-targets -- -D warnings
passed
```

No workspace-wide Clippy, Docker, PostgreSQL runtime, browser, remote CI, operator,
or production verification was run. This closes only the API crate's current strict
Clippy baseline; it does not claim a wider release gate or product completion.

未运行 workspace-wide Clippy、Docker、PostgreSQL runtime、浏览器、远端 CI、
operator 或生产验证。本记录只收束 API crate 当前的严格 Clippy 基线；不声称
更广泛的发布门禁或产品完成。
