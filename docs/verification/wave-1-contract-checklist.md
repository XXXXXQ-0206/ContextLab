# Wave 1 Contract Checklist / Wave 1 契约检查清单

> **Purpose / 目的:** Lightweight documentation-only contract test artifact. It validates references and evidence wording; it does not execute product code, a database, a browser, Docker, remote CI, an operator rehearsal, or a production workflow.
>
> **目的：** 轻量文档专用契约 test artifact。它验证引用与证据措辞；不执行产品代码、数据库、browser、Docker、remote CI、operator rehearsal 或 production workflow。

## Safe Commands / 安全命令

Run from the repository root with read-only commands:

从仓库根目录运行以下只读命令：

```powershell
Test-Path docs/architecture/wave-1-contracts.md
Test-Path docs/api/wave-1-integration-contracts.md
Test-Path docs/contributing/wave-1-docs-qa.md
Test-Path docs/adr/0003-wave-1-parallel-contract-evidence.md
Test-Path docs/user-flows/wave-1-workspace-integration.md
Test-Path docs/verification/wave-1-contract-checklist.md
rg -n "Wave 1|波次|workspace integration|passed|ignored|unobserved|blocked" docs/architecture docs/api docs/contributing docs/adr docs/user-flows docs/verification
rg -n "Docker.*(passed|success)|browser.*(passed|success)|remote CI.*(passed|success)|operator.*approved.*production|production.*deployed|public-write readiness.*passed" docs/architecture docs/api docs/contributing docs/adr docs/user-flows docs/verification
```

The first two groups must return the six paths and matching bilingual/boundary lines. The final search must return no overclaiming assertion; mentions that explicitly preserve an unobserved or deferred boundary are allowed.

前两组命令必须返回六个路径以及匹配的双语/边界行。最后一条 search 不得返回过度声明；明确保留 unobserved 或 deferred 边界的提及是允许的。

## Current Record / 当前记录

| Check / 检查 | Result / 结果 | Scope / 范围 |
| --- | --- | --- |
| Six H documentation paths exist | `passed` | Architecture, API, contribution, ADR, user-flow, verification. |
| Bilingual Wave 1 terms and evidence vocabulary are searchable | `passed` | `docs/**` H pages and linked roadmap/API pages. |
| Workspace integration requirements are explicit | `passed` | Root workspace/manifests remain Integration Lead-owned; consumers use typed projections or `unavailable`. |
| Public/private boundary is explicit | `passed` | No new public REST/OpenAPI/public SDK surface is admitted by H. |
| PostgreSQL adapter runtime | `ignored` / `unobserved` | Declared disposable database prerequisite is absent; compile/static records do not prove runtime. |
| Docker-backed runtime and authenticated browser E2E | `unobserved` | No receipt is created or inferred by this docs task. |
| Remote CI, operator rehearsal, release, and production | `unobserved` / deferred | External evidence protocol remains separate and open. |
| Scoped Clippy baseline | `blocked` / open where recorded | Existing roadmap records name the repository baseline; H does not relabel it as passed. |
| Local Git binding | `blocked` / unavailable | The current `.git` directory has no usable metadata, so commit identity is not inferred. |

## Evidence Rules / 证据规则

- `passed` applies only to the named docs command and scope.
- `passed` 只适用于所指明的文档 command 与 scope。
- `ignored` means the test did not run because its prerequisite was not admitted.
- `ignored` 表示由于 prerequisite 未准入，test 没有执行。
- `unobserved` means no reviewable receipt exists; it is not a failure and not a pass.
- `unobserved` 表示没有可审阅回执；它既不是失败，也不是通过。
- `blocked` names a concrete local blocker and does not block independent Wave 1 documentation.
- `blocked` 必须写明具体本地 blocker，且不阻断独立的 Wave 1 文档工作。
- No local docs check can establish Docker, browser, remote, operator, release, production, or public-write evidence.
- 任何本地 docs check 都不能建立 Docker、browser、remote、operator、release、production 或 public-write evidence。

## Linked Sources / 关联来源

- `docs/roadmap/parallel-development-plan.md`
- `docs/roadmap/active-long-term-goal.md`
- `docs/roadmap/completion-criteria.md`
- `docs/roadmap/external-release-evidence-protocol.md`
- `docs/api/rest-api.md`
- `docs/api/local-context-lifecycle.md`

This checklist is a documentation QA artifact, not a product test suite and not a release receipt.

本清单是文档 QA artifact，不是产品 test suite，也不是 release receipt。
