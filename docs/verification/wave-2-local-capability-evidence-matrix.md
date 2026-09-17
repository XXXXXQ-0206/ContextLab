# Wave 2 Local Capability Evidence Matrix / Wave 2 本地能力证据矩阵

> **Purpose / 目的:** Records the focused local validation observed for the Wave 2 availability contracts. This is a documentation and QA receipt, not a registration, integration, runtime, visual, remote, release, or production receipt.
>
> **目的：** 记录 Wave 2 可用性契约已观察到的 focused 本地验证。本文件是文档与 QA 回执，不是注册、集成、运行时、视觉、远端、发布或生产回执。

## Observed Local Commands / 已观察到的本地命令

All commands below were run from the repository root on 2026-07-19. The recorded status applies only to the command and scope shown.

以下所有命令均于 2026-07-19 在仓库根目录执行。记录的状态只适用于所示 command 与 scope。

| Command / 命令 | Observed output / 已观察到的输出 | Status / 状态 | Established scope / 已建立范围 |
| --- | --- | --- | --- |
| `cargo test -p contextlab-adapter-contract --test unavailable_contract` | `3 passed; 0 failed; 0 ignored` | `passed` | Stable operation mapping, required integration reporting, and the versioned unavailable DTO in the shared Rust contract. |
| `cargo test -p contextlab-cli --test command_path` | `2 passed; 0 failed; 0 ignored` | `passed` | CLI parsing and the typed unavailable evaluation result. |
| `cargo test -p contextlab-desktop-tauri-staging --test staging_shell` | `2 passed; 0 failed; 0 ignored` | `passed` | Desktop staging-shell delegation to the shared unavailable adapter and staging configuration declaration. |
| `pnpm --filter @contextlab/web exec tsx --test src/app/capability-state.test.tsx src/app/local-capability-availability-data.test.tsx` | `5 passed; 0 failed; 0 cancelled; 0 skipped; 0 todo` | `passed` | Strict unavailable wire parsing, separate Web V1 resource parsing, presenter state mapping, and accessible static markup. |

The focused local commands establish 12 passing tests. They do not combine into a full workspace check, a browser session, or an integration registration receipt.

这些 focused 本地命令共建立 12 个通过的 test。它们不能合并为完整 workspace check、browser session 或 integration registration 回执。

## Capability Contract Matrix / 能力契约矩阵

| Surface / 表面 | Contract result / 契约结果 | Evidence source / 证据来源 | Boundary / 边界 |
| --- | --- | --- | --- |
| Shared adapter transport / 共享 adapter 传输 | `unavailable` with `shared_integration_not_registered` | Shared Rust contract test / 共享 Rust contract test | No shared core is registered by this adapter. |
| CLI staging shell / CLI staging shell | Typed unavailable status and versioned DTO / 类型化 unavailable 状态与带版本 DTO | CLI command-path test / CLI command-path test | Parsing and status projection only; no core command execution. |
| Desktop staging shell / Desktop staging shell | Delegates to shared unavailable adapter / 委托给共享 unavailable adapter | Desktop staging-shell test / Desktop staging-shell test | No signed or packaged desktop release evidence. |
| Web wire DTO adapter / Web wire DTO adapter | Strictly accepts the current unavailable projection / 严格接受当前 unavailable 投影 | Web capability-state test / Web capability-state test | No live BFF/API retrieval or authenticated browser execution. |
| Web presentation resource / Web 展示资源 | Can represent local `loading`, `error`, `empty`, `available`, and `unavailable` states / 可表示本地 `loading`、`error`、`empty`、`available` 与 `unavailable` 状态 | Web local-resource test / Web local-resource test | Static rendered markup only; it is not a browser visual receipt. |

## Excluded Runtime Matrix / 排除的运行时矩阵

| Surface / 表面 | Status / 状态 | Why it is not claimed / 未声明原因 |
| --- | --- | --- |
| Docker runtime / Docker runtime | `unobserved` | No Docker command or container runtime receipt was produced. |
| PostgreSQL runtime / PostgreSQL runtime | `unobserved` | No PostgreSQL-backed test or database runtime receipt was produced. |
| Browser visual rendering / Browser 视觉渲染 | `unobserved` | Static markup tests do not observe a running browser, layout, or pixels. |
| Authenticated browser-to-service flow / 已认证 browser 到服务流程 | `unobserved` | No authenticated browser, BFF, or protected service smoke was run. |
| Remote CI / 远端 CI | `deferred` | External CI execution is outside this local docs/QA slice. |
| Release packaging / 发布打包 | `deferred` | No release workflow, signing, artifact, or distribution verification was run. |
| Production behavior / 生产行为 | `deferred` | No production deployment, monitoring, rollback, or operator receipt was produced. |

`unobserved` is neither a pass nor a failure. `deferred` identifies intentionally unperformed external work; it does not imply readiness or completion.

`unobserved` 既不是通过也不是失败。`deferred` 指明有意未执行的外部工作；它不表示已就绪或已完成。

## Documentation QA Commands / 文档 QA 命令

Run these read-only checks from the repository root after changing either Wave 2 document:

修改任一 Wave 2 文档后，在仓库根目录运行以下只读检查：

```powershell
Test-Path docs/architecture/wave-2-local-capability-availability.md
Test-Path docs/verification/wave-2-local-capability-evidence-matrix.md
rg -n "Wave 2|本地能力|local capability|passed|unobserved|deferred" docs/architecture/wave-2-local-capability-availability.md docs/verification/wave-2-local-capability-evidence-matrix.md
rg -n ('Docker.*(' + 'pass' + 'ed|suc' + 'cess)|PostgreSQL.*(' + 'pass' + 'ed|suc' + 'cess)|browser.*(' + 'pass' + 'ed|suc' + 'cess)|remote CI.*(' + 'pass' + 'ed|suc' + 'cess)|release.*(' + 'pass' + 'ed|suc' + 'cess)|production.*(' + 'pass' + 'ed|suc' + 'cess)') docs/architecture/wave-2-local-capability-availability.md docs/verification/wave-2-local-capability-evidence-matrix.md
```

The path checks must return `True`. The vocabulary check must return bilingual contract and status lines. The overclaim scan must return no lines; every excluded surface must remain `unobserved` or `deferred`.

路径检查必须返回 `True`。vocabulary check 必须返回双语 contract 与状态行。overclaim scan 不得返回任何行；每个排除的 surface 都必须保持 `unobserved` 或 `deferred`。

## Linked Documents / 关联文档

- [Wave 2 local capability availability contracts](../architecture/wave-2-local-capability-availability.md)
- [Wave 1 contracts and integration boundary](../architecture/wave-1-contracts.md)
- [Wave 1 contract checklist](wave-1-contract-checklist.md)
- [Parallel development plan](../roadmap/parallel-development-plan.md)

This matrix does not authorize project completion, public surface admission, or any change outside `docs/**`.

本矩阵不授权宣布项目完成、准入 public surface 或修改 `docs/**` 之外的任何内容。
