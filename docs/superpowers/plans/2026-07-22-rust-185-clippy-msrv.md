# Rust 1.85 Clippy MSRV Plan / Rust 1.85 Clippy MSRV 计划

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment directly
serves Criterion 8, reliable quality gates, and the charter requirements for maintainable,
reproducible Rust infrastructure. The workspace declares Rust `1.85` as its MSRV, so strict
Clippy must be able to analyze the shared authorization core at that version.

本增量直接服务条件 8“可靠质量门禁”，以及宪章对可维护、可复现 Rust 基础设施的要求。工作区将
Rust `1.85` 声明为 MSRV，因此严格 Clippy 必须能在该版本分析共享的授权核心。

**Unmet dependency and risk / 未满足依赖与风险：** `cargo clippy --workspace --all-targets --
-D warnings` stops at `TrustedExternalGroups::is_empty` because calling `Vec::is_empty` in a
`const fn` is only stable from Rust `1.87`. This suppresses all later workspace lint findings and
leaves the declared MSRV incompatible with a public authorization helper.

`cargo clippy --workspace --all-targets -- -D warnings` 会在 `TrustedExternalGroups::is_empty`
处停止，因为在 `const fn` 中调用 `Vec::is_empty` 直到 Rust `1.87` 才稳定。这会遮蔽后续 workspace
lint finding，并使声明的 MSRV 与公开的授权 helper 不兼容。

**Why now / 为什么现在：** The current private benchmark workflow is freshly verified, while this
single deterministic baseline failure blocks every full strict-Clippy receipt. Removing it advances
the quality gate without delaying independent Workflow/Plugin, Knowledge/Plugin, or benchmark Web
work.

当前私有 benchmark workflow 已完成新鲜验证，而这一确定的单一基线失败会阻断每一次完整
strict-Clippy 回执。修复它可推进质量门禁，同时不延迟独立的 Workflow/Plugin、Knowledge/Plugin 或
benchmark Web 工作。

**Explicit non-goals / 明确非目标：** Do not raise the MSRV, add a lint allowance, change RBAC or
external-group validation behavior, refactor authentication, alter public transport, or change
OpenAPI, SDK, Web, storage, evaluation, or `GraphDiff` behavior.

不提升 MSRV、不添加 lint allow、不改变 RBAC 或 external-group validation 行为、不重构认证，也不改变
public transport、OpenAPI、SDK、Web、storage、evaluation 或 `GraphDiff` 行为。

**Smallest boundary and docs / 最小边界与文档：** Only `crates/auth/src/authorization.rs` and its
existing focused tests may change. This record and the quality evidence ledger are updated only
after fresh commands finish.

只允许修改 `crates/auth/src/authorization.rs` 及其既有聚焦测试。仅在新鲜命令结束后更新本记录和质量
证据台账。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Observe the current
strict-Clippy failure, prove empty/non-empty group behavior, then run auth tests, strict auth
Clippy, workspace strict Clippy, formatting, and workspace tests. Any subsequent Clippy finding is
recorded as its own evidence gap rather than hidden or fixed opportunistically.

先观察当前 strict-Clippy 失败，证明空/非空 group 行为，然后运行 auth 测试、strict auth Clippy、
workspace strict Clippy、格式化与 workspace 测试。后续 Clippy finding 要作为独立证据缺口记录，绝不
隐藏或顺手扩大修复范围。
