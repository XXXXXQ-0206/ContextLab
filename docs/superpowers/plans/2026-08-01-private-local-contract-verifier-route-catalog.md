# Private Local Contract Verifier Route Catalog Repair / 私有本地契约 Verifier 路由目录修复

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
serves Criterion 8, Reliable release gates, by making the repository's local contract verifier
agree with the protected Benchmark read catalog that is already implemented. It also preserves the
architecture rule that route ownership is explicit, private, and fail-closed.

本增量直接服务条件 8“可靠发布门禁”：让仓库本地 contract verifier 与当前已经实现的受保护
Benchmark read route catalog 保持一致。同时保持路由 ownership 显式、private 且 fail-closed 的架构原则。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The server currently
has exactly two protected GET variants for the same read capability: the cohort-keyed workspace
route and the decision-keyed workspace route. `scripts/verify-local-contracts.ps1` still asserts
one path, one method, and one handler, so the broad local gate stops at the stale
`benchmark-workspace-route-method-count:2` baseline instead of evaluating the remaining checks.

当前服务对同一只读能力已经有且仅有两个受保护 GET variant：cohort-keyed workspace route 与
decision-keyed workspace route。`scripts/verify-local-contracts.ps1` 仍断言单 path、单 method、单 handler，
因此宽范围本地门禁在过时的 `benchmark-workspace-route-method-count:2` baseline 停止，无法继续评估其余检查。

**Why now / 为什么现在优先:** The Benchmark workspace, Context Graph snapshot, Knowledge/Memory,
workflow, and graph-relationship slices already have fresh scoped receipts. Fixing this verifier
baseline is the smallest dependency-ready local quality increment and supplies Criterion 8 evidence
without adding another feature or repeating an already closed product slice.

Benchmark workspace、Context Graph snapshot、Knowledge/Memory、workflow 与图谱关系切片均已有范围匹配的新鲜回执。
修正该 verifier baseline 是当前最小且依赖已满足的本地质量增量，可直接补充条件 8 证据，不新增功能，也不重复已收束切片。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Change
only `scripts/verify-local-contracts.ps1`, its isolated fixture test
`tests/contract/verify-local-contracts.test.ps1`, this plan, and the bilingual roadmap/completion
audit entries after fresh commands pass. The verifier must accept the exact two local GET variants,
require their exact handler registrations, and retain fail-closed checks for wrong methods, non-local
paths, duplicate/missing registrations, and public-router leakage.

仅修改 `scripts/verify-local-contracts.ps1`、其隔离 fixture 测试
`tests/contract/verify-local-contracts.test.ps1`、本计划，以及在命令真实通过后更新双语 roadmap/completion
audit。Verifier 必须只接受精确的两个 local GET variant，要求精确 handler registration，并继续对错误 method、非 local path、
重复或缺失 registration 及 public-router leakage fail closed。

**Explicit non-goals / 明确非目标:** No REST/OpenAPI/public SDK change, Web change, Rust domain
change, benchmark behavior change, public write, mutation route, migration, provider call, secret
access, Docker/PostgreSQL execution, browser or visual claim, remote CI, operator rehearsal, release,
production promotion, or second `GraphDiff` calculator. `GraphDiff::between` remains the sole graph
diff calculator.

不修改 REST/OpenAPI/public SDK、Web、Rust domain、benchmark behavior；不新增 public write、mutation route、migration、provider call、
secret access、Docker/PostgreSQL 执行、browser/visual 声明、remote CI、operator rehearsal、release、production promotion 或第二个
`GraphDiff` calculator。`GraphDiff::between` 继续是唯一图谱 diff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:**
Observe the focused PowerShell fixture red/green result and the live verifier output. Then run
`cargo fmt --all -- --check`, `cargo test --workspace --quiet --no-fail-fast`, strict offline
workspace Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and the one-`GraphDiff` static check.
Record PostgreSQL/Docker, authenticated browser, Git, remote CI, operator, release, and production
as `unobserved` or `deferred` unless directly observed.

先观察 PowerShell fixture 的 focused red/green 结果与 live verifier 输出；随后运行
`cargo fmt --all -- --check`、`cargo test --workspace --quiet --no-fail-fast`、strict offline workspace Clippy、
锁定 Rust `1.85.0` check、`pnpm check:web` 与唯一 `GraphDiff` 静态检查。PostgreSQL/Docker、authenticated browser、Git、
remote CI、operator、release 与 production 若未直接观测，必须保持 `unobserved` 或 `deferred`。

## Implementation Checklist / 实施清单

- [x] Observe the existing route-count red baseline.
- [x] Update the verifier and isolated fixture to model exactly two protected GET variants.
- [x] Prove wrong method, non-local path, registration drift, and public leakage still fail closed.
- [x] Run fresh repository verification and update bilingual roadmap/completion records.

- [x] 观察现有 route-count 红 baseline。
- [x] 更新 verifier 与隔离 fixture，使其精确建模两个受保护 GET variant。
- [x] 证明错误 method、非 local path、registration 漂移与 public leakage 仍 fail closed。
- [x] 运行仓库新鲜验证并更新双语 roadmap/completion 记录。

## Evidence Boundary / 证据边界

This is a static local contract-gate repair only. A passing verifier is evidence about source
shape, not PostgreSQL runtime, browser behavior, remote CI, release, or production readiness. The
long-term goal remains active after this increment.

本增量仅修复本地静态契约门禁。Verifier 通过只说明源码形状符合约束，不代表 PostgreSQL runtime、browser、remote CI、release 或
production readiness。完成本增量后长期目标仍保持 active。

## Fresh Local Receipt / 新鲜本地回执 (2026-08-01)

The intended red baseline was observed at `benchmark-workspace-route-method-count:2`. The minimal
repair now asserts the exact two protected GET paths and their exact handlers:
`local_benchmark_workspace` and `local_benchmark_workspace_by_decision`. The isolated fixture proves
wrong methods, non-local paths, duplicate or missing handlers, duplicate or missing catalog entries,
and public-router leakage remain blocked.

已观察到预期红 baseline：`benchmark-workspace-route-method-count:2`。最小修复现已断言两个精确的受保护 GET path 及其对应 handler：
`local_benchmark_workspace` 与 `local_benchmark_workspace_by_decision`。隔离 fixture 证明错误 method、非 local path、重复或缺失 handler、
重复或缺失 catalog entry 以及 public-router leakage 继续被阻断。

Focused `powershell -NoProfile -File tests/contract/verify-local-contracts.test.ps1` passed. The live
`powershell -NoProfile -File scripts/verify-local-contracts.ps1 -Root .` reports
`local_contract_source=passed`, `benchmark_workspace_route=passed`,
`benchmark_workspace_public_surface=passed`, `benchmark_definition_schema=passed`,
`benchmark_definition_public_boundary=passed`, `graph_diff_application=passed count=1`, and
`overall=unobserved` because no diff input was supplied. Fresh repository evidence also passed:
`cargo fmt --all -- --check`, offline workspace Rust tests with storage `212 passed, 39 ignored`,
strict offline Clippy, locked Rust `1.85.0` check, and `pnpm check:web` with public SDK `15`, local
SDK `134`, Web `270`, and production build.

聚焦命令 `powershell -NoProfile -File tests/contract/verify-local-contracts.test.ps1` 已通过。live
`powershell -NoProfile -File scripts/verify-local-contracts.ps1 -Root .` 报告
`local_contract_source=passed`、`benchmark_workspace_route=passed`、`benchmark_workspace_public_surface=passed`、
`benchmark_definition_schema=passed`、`benchmark_definition_public_boundary=passed`、
`graph_diff_application=passed count=1`；由于未提供 diff input，`overall=unobserved`。新鲜仓库证据还包括：
`cargo fmt --all -- --check`、Rust workspace（storage `212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0` check，
以及 `pnpm check:web`（public SDK `15`、local SDK `134`、Web `270` 与 production build）均通过。

No product route, REST/OpenAPI/public SDK method, Web mutation, Rust domain behavior, migration,
provider, secret, Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator
rehearsal, release, or production claim was added. The long-term goal remains active. The next
candidate is a fresh Necessity Record for the missing multi-dataset benchmark breadth receipt;
the private exact-commit relationship inspector remains a separate later candidate.

没有新增产品 route、REST/OpenAPI/public SDK method、Web mutation、Rust domain behavior、migration、provider、secret、Docker/PostgreSQL runtime、
authenticated browser、Git、remote CI、operator rehearsal、release 或 production 声明。长期目标保持 active。下一候选是为缺失的多 dataset
benchmark breadth 回执新增 Necessity Record；private exact-commit relationship inspector 作为后续独立候选保留。
