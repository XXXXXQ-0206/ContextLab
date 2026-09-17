# Private CLI Read-only Smoke Receipt / 私有 CLI 只读 Smoke 回执

**Status / 状态:** completed / verified locally / 已完成，本地已验证

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与章程原则

This increment directly supplies the named Criterion 8 requirement for documented smoke tests and
the charter requirement that CLI and Desktop reuse the shared Rust core. It documents only the
already implemented read-only staging commands and their fail-closed exit codes.

本增量直接补齐条件 8 已命名的 documented smoke tests 要求，以及 CLI/Desktop 复用 shared Rust core 的章程原则。它只记录已经实现的
只读 staging command 与 fail-closed exit code。

### Gap, priority, and dependencies / 缺口、优先级与依赖

The CLI tests already execute replay projection and capability availability inspection, but
contributors do not have a concise bilingual command-level smoke procedure. The dependency is
ready because the CLI binary, adapter contract, deterministic fixtures, and tests are present.
This is the nearest local evidence gap after the Workflow inspector receipt and is smaller and
more directly criterion-aligned than adding new product surface.

CLI tests 已实际执行 replay projection 与 capability availability inspection，但贡献者缺少简洁的双语 command-level smoke procedure。CLI
binary、adapter contract、确定性 fixture 与 tests 均已存在，因此依赖就绪。这是 Workflow inspector 回执之后最近的本地证据缺口，比增加
新产品 surface 更小且更直接对齐条件。

### Smallest boundary / 最小边界

Own only this plan and docs/verification/cli-read-only-smoke.md. Verify existing CLI,
adapter-contract, and Desktop staging tests without changing Rust crates, public API/SDK,
OpenAPI, Web, Tauri runtime, storage, secrets, Docker/PostgreSQL, or release wiring.

仅负责本计划与 docs/verification/cli-read-only-smoke.md。验证既有 CLI、adapter-contract 与 Desktop staging tests，不修改 Rust crate、
public API/SDK、OpenAPI、Web、Tauri runtime、storage、secrets、Docker/PostgreSQL 或 release wiring。

### Explicit non-goals / 明确非目标

- No new CLI command, domain implementation, network transport, provider, mutation, public write, or Desktop runtime claim.
- No production or release readiness claim; external evidence remains deferred.
- No second GraphDiff calculator; GraphDiff::between remains the sole graph-diff calculator.

- 不增加 CLI command、domain implementation、network transport、provider、mutation、public write 或 Desktop runtime claim。
- 不作 production 或 release readiness 声明；外部证据继续延期。
- 不增加第二个 GraphDiff calculator；GraphDiff::between 仍是唯一 graph-diff calculator。

### Fresh verification before the next increment / 下一增量前的新鲜验证

Run the documented binary smoke commands, focused CLI/adapter/Desktop tests, workspace format,
strict scoped Clippy, and locked Rust 1.85.0 check. Record Desktop runtime and Docker/
PostgreSQL/browser/remote/release evidence as unobserved or deferred, not passed.

运行文档中的 binary smoke commands、CLI/adapter/Desktop focused tests、workspace format、strict scoped Clippy 与锁定 Rust 1.85.0 check。
Desktop runtime 与 Docker/PostgreSQL/browser/remote/release evidence 记录为 unobserved 或 deferred，不写成 passed。

## Ownership / 所有权

Docs/QA owns both documentation files. Integration Lead owns command execution and evidence
classification. Any code defect found by the smoke commands must become a separate Necessity
Record rather than being hidden in this documentation increment.

Docs/QA 负责两个文档文件。Integration Lead 负责命令执行与证据分类。如果 smoke command 发现代码缺陷，必须建立独立 Necessity Record，
不得隐藏在本 documentation increment 中。

## Completion receipt / 收束回执

The bilingual smoke guide is complete and records only observed local behavior. A built CLI
successfully rendered the deterministic replay projection with exit code `0`; a valid unavailable
capability projection returned exit code `2`; an operation/integration mismatch failed closed with
exit code `64`; and the ordinary workspace adapter path returned its explicit unavailable status
with exit code `2`.

双语 smoke guide 已完成，只记录已观测的本地行为。构建后的 CLI 成功渲染确定性 replay projection，退出码为 `0`；合法 unavailable
capability projection 返回退出码 `2`；operation/integration mismatch 以退出码 `64` fail closed；普通 workspace adapter path 返回明确
unavailable status，退出码为 `2`。

| Verification / 验证 | Status / 状态 | Fresh receipt / 新鲜回执 |
| --- | --- | --- |
| Binary smoke commands | `passed` | `cargo build --offline -p contextlab-cli`; replay `0`, capability unavailable `2`, capability contract drift `64`, workspace unavailable `2`. / 均已真实执行。 |
| Adapter/CLI/Desktop focused tests | `passed` | Adapter `8 passed`, CLI `5 passed`, Desktop staging `5 passed`. / adapter `8`、CLI `5`、Desktop staging `5` 项通过。 |
| Format and scoped Clippy | `passed` | `cargo fmt --all -- --check`; strict scoped Clippy for the three packages. / 均通过。 |
| Locked MSRV | `passed` | `cargo +1.85.0 check --workspace --all-targets --locked --offline`. / 通过。 |

No CLI command, Rust domain implementation, network transport, public API/SDK/OpenAPI method,
Web mutation, provider, secret, Docker/PostgreSQL runtime, Tauri runtime, release, or production
claim was added. Tauri runtime, Docker/PostgreSQL, authenticated browser, Git change-set, remote
CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. The long-term
goal remains active.

没有新增 CLI command、Rust domain implementation、network transport、public API/SDK/OpenAPI method、Web mutation、provider、secret、
Docker/PostgreSQL runtime、Tauri runtime、release 或 production claim。Tauri runtime、Docker/PostgreSQL、authenticated browser、Git
change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active。
