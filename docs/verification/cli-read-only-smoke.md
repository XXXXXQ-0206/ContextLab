# CLI Read-only Smoke / CLI 只读 Smoke

This guide covers the local, read-only staging CLI. It is a contributor smoke procedure, not a
production-readiness or Desktop runtime receipt.

本文档覆盖本地、只读 staging CLI。这是贡献者 smoke procedure，不是 production-readiness 或 Desktop runtime 回执。

## Preconditions / 前置条件

Run from the repository root with the locked Rust toolchain and no environment credentials:

在仓库根目录、锁定 Rust toolchain 下运行；不需要 environment credential：

```powershell
cargo build --offline -p contextlab-cli
$cli = (Resolve-Path .\\target\\debug\\contextlab-cli.exe).Path
```

The CLI is intentionally provider-free. The regular adapter-backed commands return a typed
unavailable status until their shared Rust application integration is registered.

CLI 刻意保持 provider-free。普通 adapter-backed command 在 shared Rust application integration 注册前会返回 typed unavailable status。

## Commands and expected exit codes / 命令与预期退出码

### Replay projection, valid input / Replay projection，合法输入

```powershell
$replay = '{"schema_version":1,"context_id":"00000000-0000-0000-0000-000000000001","initialized":true,"commit_id":"00000000-0000-0000-0000-000000000004","context_metadata":null,"components":[{"component_id":"00000000-0000-0000-0000-000000000003","kind":"knowledge","name":"Knowledge","metadata":{"rank":3},"content_hash":"sha256:three"},{"component_id":"00000000-0000-0000-0000-000000000002","kind":"prompt","name":"Prompt","metadata":{"rank":2},"content_hash":"sha256:two"}],"relationships":[{"source_component_id":"00000000-0000-0000-0000-000000000003","target_component_id":"00000000-0000-0000-0000-000000000002"}]}'
& $cli replay inspect $replay
$LASTEXITCODE
```

Expected exit code: `0`. Output is deterministic, sorted, and limited to schema/context/commit
identity, counts, component IDs, and relationship IDs. Raw component content and provider data
are not printed.

预期退出码：`0`。输出确定性排序，只包含 schema/context/commit identity、计数、component ID 与 relationship ID，不打印 raw component
content 或 provider data。

### Capability availability, valid unavailable input / 能力可用性，合法 unavailable 输入

```powershell
$capability = '{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-evaluation","availability":"unavailable","reason":"shared_integration_not_registered"}'
& $cli capability inspect $capability
$LASTEXITCODE
```

Expected exit code: `2`. Exit `2` is the intentional local unavailable status; the output remains
bilingual and redacted.

预期退出码：`2`。`2` 是刻意表达 local unavailable 的状态；输出保持双语与脱敏。

### Capability contract drift, invalid input / 能力契约漂移，非法输入

```powershell
$invalidCapability = '{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-diff-engine","availability":"unavailable","reason":"shared_integration_not_registered"}'
& $cli capability inspect $invalidCapability
$LASTEXITCODE
```

Expected exit code: `64`, with no standard output. The parser fails closed when the stable
operation/integration mapping is inconsistent.

预期退出码：`64`，且没有 standard output。当 stable operation/integration mapping 不一致时，parser 会 fail closed。

### Adapter-backed read-only path / Adapter-backed 只读路径

```powershell
& $cli workspace inspect workspace-alpha
$LASTEXITCODE
```

Expected exit code: `2`, with an explicit unavailable message. This is a staging boundary, not a
claim that the Context Core application adapter is already registered.

预期退出码：`2`，并输出明确的 unavailable message。这是 staging boundary，不代表 Context Core application adapter 已经注册。

## Verification receipt / 验证回执

Fresh local verification for this document:

本次文档的新鲜本地验证：

| Command / 命令 | Result / 结果 |
| --- | --- |
| cargo test --offline -p contextlab-adapter-contract --test unavailable_contract | 8 passed |
| cargo test --offline -p contextlab-cli --test command_path | 5 passed |
| cargo test --offline -p contextlab-desktop-tauri-staging --test staging_shell | 5 passed |
| cargo fmt --all -- --check | passed |
| scoped strict Clippy for the three CLI/Desktop packages | passed |
| cargo +1.85.0 check --workspace --all-targets --locked --offline | passed |

### Executable process receipt / 可执行进程回执 (2026-08-01)

The offline CLI build passed with `cargo build --offline -p contextlab-cli`. The four documented
process paths were then observed from the built binary:

| Path / 路径 | Observed result / 观测结果 |
| --- | --- |
| `replay inspect <valid snapshot>` | Exit `0`; deterministic output contained schema/context/commit identity, `2` components, `1` relationship, sorted component IDs, and the relationship ID pair only. / 退出 `0`；确定性输出只包含 schema/context/commit identity、`2` 个 component、`1` 条 relationship、排序后的 component ID 与 relationship ID pair。 |
| `capability inspect <valid unavailable contract>` | Exit `2`; bilingual unavailable projection rendered and no credentials/provider data appeared. / 退出 `2`；渲染双语 unavailable projection，未出现 credential/provider data。 |
| `capability inspect <operation/integration mismatch>` | Exit `64`; stdout was empty and stderr contained only the bilingual invalid-contract error. / 退出 `64`；stdout 为空，stderr 仅包含双语 invalid-contract error。 |
| `workspace inspect workspace-alpha` | Exit `2`; output was `workspace inspect: unavailable; awaiting contextlab-context-core registration`. / 退出 `2`；输出为 `workspace inspect: unavailable; awaiting contextlab-context-core registration`。 |

The process receipt is local, provider-free, and read-only. It does not claim the Context Core
adapter is registered or that the Desktop/Tauri runtime was launched.

该进程回执是本地、provider-free、只读证据。不声称 Context Core adapter 已注册，也不声称 Desktop/Tauri runtime 已启动。

The Desktop shell is a Rust staging library and has no observed Tauri runtime smoke in this
environment. Docker/PostgreSQL, authenticated browser, Git change-set, remote CI, operator
rehearsal, release, and production evidence remain unobserved or deferred.

Desktop shell 是 Rust staging library，本环境没有观测到 Tauri runtime smoke。Docker/PostgreSQL、authenticated browser、Git change-set、remote
CI、operator rehearsal、release 与 production evidence 继续为 unobserved 或 deferred。
