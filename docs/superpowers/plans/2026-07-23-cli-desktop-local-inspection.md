# CLI and Desktop Local Capability Inspection

## Necessity Record / 必要性记录

**Criterion and principle / 条件与原则：** This is a narrow contribution to Completion Criteria 1, 2, 6, and 9: ContextLab presentation adapters must consume stable shared contracts, remain independently testable, fail closed at trust boundaries, and retain bilingual documentation. It also follows the charter's CLI/Desktop shared-Rust-core direction without placing business logic in either presentation shell.

这是对完成条件 1、2、6 与 9 的狭窄贡献：ContextLab 的展示适配器必须消费稳定的共享契约、可独立测试、在信任边界 fail closed，并保持中英双语文档。它也遵循宪章中 CLI/Desktop 复用 Rust core 的方向，但不把业务逻辑放入任何展示 shell。

**Unmet dependency and evidence gap / 未满足的依赖与证据缺口：** `contextlab-adapter-contract` already serializes the local capability-availability DTO for unavailable shared integrations, and the CLI/Desktop staging shells already carry that DTO. There is no deterministic end-user CLI inspection path for a serialized DTO, no strict deserialization proof that rejects schema, operation, integration, availability, reason, or extra-field drift, and no Desktop presenter handoff proving that the same parsed contract can carry stable capability metadata. Without those boundaries, a local adapter can accidentally trust malformed input or display inconsistent operation/capability identity.

`contextlab-adapter-contract` 已能为未注册的共享 integration 序列化本地 capability-availability DTO，CLI/Desktop staging shell 也已携带该 DTO。但目前没有面向最终用户、可确定复现的 CLI 序列化 DTO 检查路径；没有严格反序列化证明可拒绝 schema、operation、integration、availability、reason 或额外字段漂移；也没有 Desktop presenter handoff 证明同一已解析契约能够携带稳定 capability metadata。缺少这些边界时，本地适配器可能误信任格式错误的输入，或展示不一致的 operation/capability identity。

**Why now / 为什么现在做：** The shared adapter contract and both staging shells are already dependency-ready and limited to unavailable behavior. This is the smallest local-only integration that turns their existing serialized boundary into an inspectable, testable end-user path before any future shared-core registration, Tauri command binding, or provider integration. It is earlier than workflow execution, public transport, and runtime desktop work because it adds no new domain dependency and closes a directly adjacent contract-validation gap.

共享 adapter contract 和两个 staging shell 均已依赖就绪，且目前只表达 unavailable 行为。这是在未来 shared-core 注册、Tauri command binding 或 provider integration 之前，将既有序列化边界转为可检查、可测试最终用户路径的最小本地增量。它早于 workflow execution、public transport 与 Desktop runtime 工作，因为它不引入新的领域依赖，并收束一个直接相邻的契约校验缺口。

**Smallest boundary / 最小边界：** Only `apps/cli/**` and `apps/desktop/**` may change. The shared adapter-contract crate will own validation and the deterministic local presentation projection. The CLI will accept exactly one serialized DTO argument for a read-only `capability inspect` command and render its source schema, operation ID, stable derived capability ID, unavailable state, and bilingual metadata. The Desktop staging shell will pass the exact same validated contract projection to a typed presenter handoff. Neither shell will calculate domain state.

仅允许修改 `apps/cli/**` 与 `apps/desktop/**`。共享 adapter-contract crate 将拥有校验和确定性的本地展示投影。CLI 将为只读 `capability inspect` 命令接收恰好一个序列化 DTO 参数，并渲染其来源 schema、operation ID、稳定派生的 capability ID、unavailable state 与双语 metadata。Desktop staging shell 将把同一份已校验契约投影交给类型化 presenter handoff。两个 shell 都不得计算领域状态。

**Non-goals / 非目标：** No shared-domain crate registration; no available state; no write command; no filesystem, environment, secret, provider, network, Docker, database, or credential read; no public REST, OpenAPI, SDK, Web, or package change; no Tauri command, window, bundle, runtime, or installation claim; no mutation, evaluation execution, workflow execution, plugin loading, or public release work.

不注册共享领域 crate；不增加 available state；不增加写命令；不读取文件系统、环境、密钥、provider、网络、Docker、数据库或凭据；不修改 public REST、OpenAPI、SDK、Web 或 package；不增加 Tauri command、window、bundle、runtime 或安装声明；不涉及 mutation、evaluation execution、workflow execution、plugin loading 或 public release 工作。

**Required fresh evidence / 所需新鲜证据：** Observe focused CLI and Desktop tests fail before implementation, then pass after the shared fail-closed parser and presenter handoff exist. Run `cargo fmt --all -- --check`, focused strict Clippy for the adapter-contract, CLI, and Desktop staging packages, and focused package tests. Record that the output is deterministic from supplied JSON only and that Desktop runtime/Tauri invocation evidence remains unobserved.

必须先观察聚焦 CLI 与 Desktop 测试在实现前失败，再在共享 fail-closed parser 与 presenter handoff 完成后通过。运行 `cargo fmt --all -- --check`、adapter-contract、CLI 与 Desktop staging package 的聚焦严格 Clippy，以及聚焦 package tests。记录输出只由提供的 JSON 确定生成，并记录 Desktop runtime/Tauri invocation 证据仍未观测。

## Review Regression Root Cause / 评审回归根因

**Rust 1.85 compiler boundary / Rust 1.85 编译器边界：** The CLI entry point used a let-chain in an `if` condition. Although accepted by the current stable toolchain, that syntax is unstable on the declared Rust 1.85 MSRV and fails with `E0658`. The repair must preserve the exact `capability inspect` dispatch and exit behavior using Rust 1.85-compatible control flow.

CLI 入口在 `if` 条件中使用了 let-chain。该语法虽可被当前 stable 工具链接受，但在项目声明的 Rust 1.85 MSRV 上仍不稳定，并以 `E0658` 失败。修复必须使用兼容 Rust 1.85 的控制流，同时保持 `capability inspect` 的分派与退出行为完全不变。

**Projection mapping authority / 投影映射权威来源：** `AdapterResponse::local_capability_availability` combined the operation identifier from `AdapterRequest` with the integration identifier stored independently in `AdapterResponse`. A malformed or future adapter response could therefore serialize an invalid operation/integration pair. The request's typed operation-to-integration mapping must be the single authority for both wire fields so the projection cannot emit a mismatched pair.

`AdapterResponse::local_capability_availability` 将来自 `AdapterRequest` 的 operation identifier 与独立存储在 `AdapterResponse` 中的 integration identifier 组合。格式错误或未来新增的 adapter response 因而可能序列化出无效的 operation/integration 配对。请求中的类型化 operation-to-integration 映射必须成为两个 wire 字段的唯一权威来源，使投影无法发出错配组合。

**Serialized trust boundary / 序列化信任边界：** `parse_serialized` treated one canonical JSON byte sequence as the contract by comparing constructed strings. That rejects semantically equivalent JSON with valid field reordering or whitespace while coupling validation to formatting. The parser must deserialize through a Serde wire type with `deny_unknown_fields`, then validate the exact schema version, operation/integration mapping, unavailable literal, and reason literal before constructing the trusted DTO; malformed, unknown, or mismatched input remains fail-closed.

`parse_serialized` 通过比较构造出的字符串，把单一规范 JSON 字节序列误当作契约。这会拒绝字段顺序或空白不同但语义等价的合法 JSON，并使校验与格式耦合。parser 必须通过带 `deny_unknown_fields` 的 Serde wire type 反序列化，再校验精确 schema version、operation/integration 映射、unavailable 字面量与 reason 字面量，之后才能构造可信 DTO；格式错误、未知字段或错配输入继续 fail closed。

## TDD Evidence / TDD 证据

**RED observed / 已观察 RED：** `cargo test -p contextlab-cli --test command_path capability_inspect -- --nocapture` ran two focused tests and failed `0 passed; 2 failed`: the missing command returned exit `64` and the old usage output instead of the specified unavailable inspection. `cargo test -p contextlab-desktop-tauri-staging --test staging_shell desktop_shell_hands -- --nocapture` failed with `E0599` because the presenter handoff did not exist. `cargo test -p contextlab-adapter-contract --test unavailable_contract serialized_local_availability_parser_fails_closed_for_contract_drift -- --nocapture` failed with `E0599` because strict serialized parsing did not exist.

`cargo test -p contextlab-cli --test command_path capability_inspect -- --nocapture` 已运行两项聚焦测试并以 `0 passed; 2 failed` 失败：缺失的命令返回 exit `64` 和原有 usage 输出，而不是指定的 unavailable inspection。`cargo test -p contextlab-desktop-tauri-staging --test staging_shell desktop_shell_hands -- --nocapture` 因 presenter handoff 不存在而以 `E0599` 失败。`cargo test -p contextlab-adapter-contract --test unavailable_contract serialized_local_availability_parser_fails_closed_for_contract_drift -- --nocapture` 因严格序列化 parser 不存在而以 `E0599` 失败。

**GREEN and quality receipts / GREEN 与质量回执：** `cargo test -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging --tests` passed with the contract `4 passed`, CLI `4 passed`, and Desktop `3 passed`. `cargo fmt --all -- --check` passed. `cargo clippy -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging --all-targets -- -D warnings` passed. An intermediate CLI formatter failure rendered `availability: \"unavailable\"`; it was repaired by replacing `{:?}` with `{}` before the final fresh receipts.

`cargo test -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging --tests` 已通过：contract `4 passed`、CLI `4 passed`、Desktop `3 passed`。`cargo fmt --all -- --check` 通过。`cargo clippy -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging --all-targets -- -D warnings` 通过。中途曾出现 CLI formatter 问题，输出为 `availability: \"unavailable\"`；在最终新鲜回执前已将 `{:?}` 改为 `{}` 修复。

**Review repair RED / 评审修复 RED：** The new mapping regression `local_capability_projection_uses_the_requests_operation_integration_mapping` failed `0 passed; 1 failed` with left `contextlab-diff-engine` and right `contextlab-evaluation`. The new semantic JSON regression `serialized_local_availability_parser_accepts_equivalent_json_formatting` failed `0 passed; 1 failed` with `LocalCapabilityAvailabilityParseError`. `cargo +1.85.0 check -p contextlab-cli` failed with `E0658` at `apps/cli/src/main.rs:13` because let expressions in that condition are unstable.

新增映射回归测试 `local_capability_projection_uses_the_requests_operation_integration_mapping` 以 `0 passed; 1 failed` 失败，left 为 `contextlab-diff-engine`，right 为 `contextlab-evaluation`。新增语义 JSON 回归测试 `serialized_local_availability_parser_accepts_equivalent_json_formatting` 以 `0 passed; 1 failed` 和 `LocalCapabilityAvailabilityParseError` 失败。`cargo +1.85.0 check -p contextlab-cli` 在 `apps/cli/src/main.rs:13` 以 `E0658` 失败，原因是该条件位置的 let expression 尚不稳定。

**Review repair GREEN / 评审修复 GREEN：** Both focused regressions passed independently (`1 passed; 0 failed`). `cargo test --offline -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging --tests` passed with adapter contract `6 passed`, CLI `4 passed`, and Desktop `3 passed`. `cargo +1.85.0 check --offline -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging` passed. Package-scoped formatting passed for all three packages. Strict Clippy passed for all three packages on both current stable and Rust 1.85 with `--all-targets -- -D warnings`.

两项聚焦回归测试分别通过（`1 passed; 0 failed`）。`cargo test --offline -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging --tests` 通过：adapter contract `6 passed`、CLI `4 passed`、Desktop `3 passed`。`cargo +1.85.0 check --offline -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging` 通过。三个 package 的聚焦格式检查通过。三个 package 在 current stable 与 Rust 1.85 上的严格 Clippy 均以 `--all-targets -- -D warnings` 通过。

**Fresh workspace-format blocker / 当前 workspace 格式阻塞：** The first fresh `cargo fmt --all -- --check` returned exit `1` for pre-existing, out-of-scope drift in `crates/storage/tests/benchmark_workspace_projection.rs` and `crates/workflow/tests/workflow_deterministic_replay.rs`. A final rerun still returned exit `1`, now only for `crates/workflow/tests/workflow_deterministic_replay.rs`. Neither file was modified because this repair exclusively owns `apps/cli/**` and this plan. `cargo fmt -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging -- --check` returned exit `0` with no output.

首次新鲜执行 `cargo fmt --all -- --check` 返回 exit `1`，差异位于既有且越界的 `crates/storage/tests/benchmark_workspace_projection.rs` 与 `crates/workflow/tests/workflow_deterministic_replay.rs`。最终复跑仍返回 exit `1`，此时仅剩 `crates/workflow/tests/workflow_deterministic_replay.rs`。本修复仅拥有 `apps/cli/**` 与本计划，因此未修改这两个文件。`cargo fmt -p contextlab-adapter-contract -p contextlab-cli -p contextlab-desktop-tauri-staging -- --check` 返回 exit `0` 且无输出。

**Unobserved boundary / 未观测边界：** The Desktop staging shell was tested as a Rust library only. No Tauri command, window, bundle, installed desktop application, provider, network, Docker, database, filesystem, environment, or secret runtime was started or inspected.

Desktop staging shell 仅作为 Rust library 测试。未启动或检查 Tauri command、window、bundle、已安装 Desktop 应用、provider、网络、Docker、数据库、文件系统、环境或密钥 runtime。
