# Docs/QA Evidence Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reconcile the bilingual roadmap with directly observed local evidence for the completed pairwise and quality slices without promoting local checks into runtime or release claims.

**Architecture:** This is a documentation-only closure. Dated receipts remain historical; one current evidence snapshot is authoritative for present status, and every status uses `passed`, `failed`, `ignored`, `unobserved`, or `deferred` with an explicit scope.

**Tech Stack:** Markdown, PowerShell, Rust/Cargo, pnpm/TypeScript, Playwright source smoke.

---

## Necessity Record / 必要性记录

**Criterion or principle served / 服务的条件或原则：** Completion Criteria 1, 2, 3, 4, 6, 7, and 9; bilingual governance; truthful, reproducible evidence boundaries.

**Completion condition or charter principle / 完成条件或宪章原则：** 完成条件 1、2、3、4、6、7、9；双语治理；真实且可复现的证据边界。

**Unmet dependency, risk, or evidence gap / 未满足的依赖、风险或证据缺口：** The roadmap still described C+E and D+E bridges as queued, omitted an A+G row, retained a superseded strict-Clippy failure, and mixed historical green receipts with current status. Visual smoke, Docker/PostgreSQL runtime, authenticated browser E2E, remote CI, operator/release/production, and Git change-set evidence also needed explicit present-tense classification.

路线图仍将 C+E 与 D+E bridge 写为排队状态，缺少 A+G 行，保留了已过期的 strict-Clippy 失败，并混用了历史绿灯回执与当前状态。visual smoke、Docker/PostgreSQL runtime、authenticated browser E2E、remote CI、operator/release/production 与 Git change-set evidence 也需要明确的当前时态分类。

**Why this is next / 为什么现在做：** The implementation files and focused tests are present, so documentation reconciliation is dependency-ready and prevents completed local slices from remaining falsely queued. It also prevents an existing source-level smoke script from being treated as a passing browser receipt.

实现文件与聚焦测试均已存在，因此文档对账已经依赖就绪，可避免已完成的本地切片继续被误写为排队状态，也可避免把仅仅存在的 source-level smoke script 当成已通过的浏览器回执。

**Non-goals / 非目标：** No product code, manifest, app, script, workflow, public API, runtime, database, release, production, or worker-output change. No attempt to repair the failing visual smoke assertion. No claim that an independently launched worker completed anything.

不修改产品代码、manifest、app、script、workflow、public API、runtime、database、release、production 或其他 worker 输出；不修复 visual smoke 失败断言；不声称任何独立启动的 worker 已完成任务。

**Smallest boundary and bilingual docs / 最小边界与双语文档：** `docs/roadmap/parallel-development-plan.md`, `docs/roadmap/completion-criteria.md`, `docs/roadmap/active-long-term-goal.md`, and this plan. English and Chinese status statements must carry equivalent scope and evidence labels.

**Required evidence / 所需证据：** Direct local source inspection; focused pairwise tests; `cargo fmt --all -- --check`; `cargo test --workspace --quiet`; final `cargo clippy --workspace --all-targets -- -D warnings`; final `pnpm check:web`; the existing `python apps/web/verify-context-workspace.py`; link, parity, and overclaim scans. A failed command remains failed and is never converted to passing by source presence.

## Files / 文件

- Create: `docs/superpowers/plans/2026-07-23-docs-qa-evidence-refresh.md`
- Modify: `docs/roadmap/parallel-development-plan.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Read only: current Rust, TypeScript, Web, API, architecture, verification, and governance evidence

## Closure Steps / 收束步骤

- [x] Read the required governance, architecture, roadmap, and source evidence.
- [x] Verify F+G adapter/CLI/Desktop, A+G benchmark/Web, C+E Workflow/Plugin, and D+E Knowledge/Plugin boundaries with focused local tests.
- [x] Run current formatting, workspace test, strict Clippy, and full Web gates.
- [x] Run the existing visual smoke and preserve its observed failure.
- [x] Correct the ownership, pairwise, verification, and active-status records bilingually.
- [x] Run final documentation link, bilingual-parity, stale-status, matrix, and overclaim checks after the edits.

## Current Local Evidence Snapshot / 当前本地证据快照

| Surface / 表面 | Status / 状态 | Direct observation / 直接观测 | Boundary / 边界 |
| --- | --- | --- | --- |
| F+G adapter/CLI/Desktop | `passed` | Adapter contract `4`, CLI `4`, Desktop `3`; Web suite `116`. / adapter contract `4`、CLI `4`、Desktop `3`；Web suite `116`。 | Local tests only; no signed/package runtime. / 仅本地测试；不含签名或打包运行时。 |
| A+G benchmark workspace | `passed` | Evaluation projection `5`; Web benchmark workspace coverage is included in Web `116`. / evaluation projection `5`；Web benchmark workspace 覆盖包含在 Web `116` 中。 | Provider-free projection and Web adaptation only. / 仅 provider-free projection 与 Web adaptation。 |
| C+E Workflow/Plugin | `passed` | Focused bridge `7`. / 聚焦 bridge `7`。 | Core capability bridge only; no provider or public transport. / 仅核心 capability bridge；不含 provider 或 public transport。 |
| D+E Knowledge/Plugin | `passed` | Focused citation bridge `3`. / 聚焦 citation bridge `3`。 | Redacted provider-free bridge only. / 仅脱敏、无 provider 的 bridge。 |
| Quality slices | `passed` | Formatting passed; workspace tests passed (API `154`; storage `166 passed, 36 ignored`); final strict workspace Clippy passed; final Web gate passed (public SDK `14`, local SDK `53`, Web `116`, production build). / 格式、workspace test、最终 strict Clippy 与最终 Web gate 均通过。 | Local compile/test/build evidence only. / 仅本地编译、测试与构建证据。 |
| Preview visual smoke | `failed` | `python apps/web/verify-context-workspace.py` stopped at `missing text: aggregated averages`. / 命令在 `missing text: aggregated averages` 处停止。 | No passing desktop/mobile screenshot receipt. / 没有通过的桌面或移动截图回执。 |
| Docker/PostgreSQL runtime | `unobserved` | Docker is disabled; no database command was run. / Docker 已禁用；未运行数据库命令。 | Ignored PostgreSQL tests are not runtime proof. / ignored PostgreSQL test 不是运行时证明。 |
| Authenticated browser E2E | `unobserved` | No logged-in browser-to-BFF-to-Axum flow was run. / 未运行登录态 browser-to-BFF-to-Axum flow。 | The failed preview smoke is unauthenticated. / 失败的 preview smoke 不含认证。 |
| Remote CI, operator, release, production | `deferred` | No external receipt was observed. / 未观察到外部回执。 | External deployment gates only. / 仅外部部署门禁。 |
| Git change-set | `unobserved` | `git status --short` reported that the directory is not a Git repository. / `git status --short` 报告该目录不是 Git repository。 | No diff, commit, or branch attribution. / 不提供 diff、commit 或 branch 归属。 |

The first broad Web and strict-Clippy attempts observed transient missing-export/missing-file failures while concurrent files were appearing. Only the final reruns above are used for current green status; no worker completion is inferred from that concurrency.

首次 broad Web 与 strict-Clippy 尝试在并发文件写入期间观察到短暂的 missing-export/missing-file 失败。当前绿灯状态只采用上表的最终重跑结果；不从这些并发变化推断任何 worker 已完成任务。
