# Storage Strict-Clippy Baseline Repair / Storage 严格 Clippy 基线修复

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This repair supports the local quality and reliable-gate evidence required by Completion Criterion 8, together with the charter requirements for Rust MSRV compatibility, maintainable storage boundaries, readable code, and continuously enforced formatting and lint checks.

本修复服务 Completion Criterion 8 所要求的本地质量与可靠门禁证据，同时落实项目宪章关于 Rust MSRV 兼容性、可维护 storage 边界、可读代码以及持续执行格式与 lint 检查的要求。

**Observed gap and root cause / 已观测缺口与根因：** After the auth crate's Rust 1.85 const-context incompatibility was repaired, workspace strict Clippy exposed a second independent root cause in `contextlab-storage`: exactly thirteen warnings promoted to errors. They comprise two needless borrows, three helpers or constructors with more than seven arguments, seven manually implemented enum defaults that are derivable, and one test helper whose `let-else` is equivalent to `?`. These warnings currently prevent fresh strict-Clippy evidence for the storage crate.

在 auth crate 的 Rust 1.85 const-context 不兼容问题修复后，workspace strict Clippy 暴露了第二个独立根因：`contextlab-storage` 中恰好十三条被提升为错误的 warning，包括两个 needless borrow、三个超过七个参数的 helper 或 constructor、七个可派生的 enum Default 手写实现，以及一个可用 `?` 等价表达的测试 helper `let-else`。这些 warning 当前阻止 storage crate 取得新鲜严格 Clippy 证据。

**Why now / 为什么现在优先：** The defect is already reproduced, isolated to storage, and blocks a named quality gate shared by every local product increment. Resolving it now is smaller and lower-risk than carrying a known lint failure into the concurrent benchmark, workflow, and knowledge integration lines.

该缺陷已经复现并隔离到 storage，且阻断所有本地产品增量共享的具名质量门禁。此时修复比把已知 lint failure 带入并行 benchmark、workflow 与 knowledge 集成线更小、风险更低。

**Explicit non-goals / 明确非目标：** No storage semantics, query behavior, ordering policy, public or private transport, migration, schema, authentication, authorization, provider execution, Docker/PostgreSQL runtime, secret handling, public API, SDK, Web, roadmap, MSRV, dependency, or `GraphDiff` change. No lint allow or warning suppression is permitted.

不改变 storage 语义、query 行为、排序策略、公有或私有 transport、migration、schema、authentication、authorization、provider execution、Docker/PostgreSQL runtime、secret handling、public API、SDK、Web、roadmap、MSRV、dependency 或 `GraphDiff`。不得增加 lint allow 或 warning suppression。

**Smallest affected boundary / 最小受影响边界：** Repair only the observed storage warnings. Use typed persisted-row or summary inputs where argument grouping carries domain meaning, derive enum defaults with explicit `#[default]` variants, remove needless references, and simplify the test-only optional pool helper. Existing APIs and call sites are preserved where doing so does not retain the warning; any mechanically required storage-internal call-site adjustment must remain behavior-equivalent.

仅修复已观测的 storage warning。参数分组具有领域含义时使用 typed persisted-row 或 summary input；通过显式 `#[default]` variant 派生 enum default；移除无用引用；简化仅测试使用的 optional pool helper。在不保留 warning 的前提下尽量保持既有 API 与调用点；任何机械必要的 storage 内部调用点调整都必须保持行为等价。

**Fresh verification required / 所需新鲜验证：** Treat the initial strict-Clippy failure as the red baseline. After the minimal repair, run `cargo fmt --all`, `cargo test -p contextlab-storage --quiet`, and `cargo clippy -p contextlab-storage --all-targets -- -D warnings`. The main integration thread owns workspace-wide Clippy and broader concurrent verification.

以初始 strict-Clippy failure 作为红灯基线。完成最小修复后运行 `cargo fmt --all`、`cargo test -p contextlab-storage --quiet` 与 `cargo clippy -p contextlab-storage --all-targets -- -D warnings`。workspace-wide Clippy 与更广的并行验证由主集成线程负责。

## Status / 状态

- [x] Reproduce exactly thirteen strict-Clippy errors / 精确复现十三条 strict-Clippy error。
- [x] Apply the thirteen initially observed behavior-equivalent repairs without lint suppression / 在不抑制 lint 的前提下完成最初观测到的十三项行为等价修复。
- [x] Repair one additional test-only `cloned_ref_to_slice_refs` lint first exposed after those thirteen errors cleared; this is a sequentially observed verification finding, not a revision of the initial count / 修复前十三项清除后才暴露的一条额外、仅测试使用的 `cloned_ref_to_slice_refs` lint；它是顺序观测到的验证发现，不改变最初计数。
- [x] Observe focused storage tests and strict Clippy / 新鲜观察聚焦 storage test 与 strict Clippy。

## Fresh Evidence / 新鲜证据

- RED: `cargo clippy -p contextlab-storage --all-targets -- -D warnings` initially reported the thirteen recorded errors. After their repair, it exposed one further test-only `cloned_ref_to_slice_refs` error, which was repaired separately and recorded above.
- GREEN: `cargo fmt --all` completed successfully.
- GREEN: `cargo test -p contextlab-storage --quiet` completed with `166 passed`, `0 failed`, and `36 ignored` in the main unit group; the additional integration groups completed with `22`, `5`, `5`, `1`, and `3` passing tests.
- GREEN: `cargo clippy -p contextlab-storage --all-targets -- -D warnings` completed successfully after the test-only repair.

- 红灯：`cargo clippy -p contextlab-storage --all-targets -- -D warnings` 最初报告记录的十三个 error。修复后它又暴露一条额外、仅测试使用的 `cloned_ref_to_slice_refs` error；该项已单独修复并如上记录。
- 绿灯：`cargo fmt --all` 成功完成。
- 绿灯：`cargo test -p contextlab-storage --quiet` 成功完成，主 unit group 为 `166 passed`、`0 failed`、`36 ignored`；额外 integration group 分别为 `22`、`5`、`5`、`1` 与 `3` 个通过测试。
- 绿灯：测试专用修复后，`cargo clippy -p contextlab-storage --all-targets -- -D warnings` 成功完成。

Workspace-wide Clippy remains owned by the main integration thread and was intentionally not run here.

workspace-wide Clippy 仍由主集成线程负责，本任务按边界未执行它。
