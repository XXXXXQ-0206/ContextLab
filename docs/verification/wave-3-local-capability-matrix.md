# Wave 3 Local Capability Bridge Verification Matrix / Wave 3 本地能力桥接验证矩阵

> **Test-first status / 测试优先状态：** `DOC-W3-001` first ran red because the paired architecture record was absent. Its green rerun and all final read-only QA results are recorded below. Documentation QA is not Core, API, Web runtime, public transport, Docker, browser, remote, release, or production evidence.
>
> **测试优先状态：** `DOC-W3-001` 首次运行时因配套架构记录缺失而进入 red。其 green 重跑及全部最终只读 QA 结果记录于下方。文档 QA 不是 Core、API、Web runtime、public transport、Docker、browser、remote、release 或 production 证据。

## Documentation Contract Test / 文档契约测试

| Test ID / 测试标识 | Required behavior / 必需行为 |
| --- | --- |
| `DOC-W3-001` | The paired architecture record and this matrix must declare the admitted local-read boundary, precise Core/API/Web ownership, explicit V1 and redaction/fail-closed rules, deterministic ordering, integration dependencies, and the five evidence states `passed`, `ignored`, `unobserved`, `deferred`, and `blocked`. / 配套架构记录与本矩阵必须声明已准入的本地只读边界、精确的 Core/API/Web 所有权、显式 V1 及脱敏/fail-closed 规则、确定性排序、集成依赖，以及五种证据状态 `passed`、`ignored`、`unobserved`、`deferred` 和 `blocked`。 |

Run this read-only assertion from the repository root. Before the paired architecture record is added, it must fail because that document is absent.

在仓库根目录运行以下只读断言。在配套架构记录加入前，它必须因该文档不存在而失败。

```powershell
$architecturePath = 'docs/architecture/wave-3-local-capability-bridges.md'
$matrixPath = 'docs/verification/wave-3-local-capability-matrix.md'

if (-not (Test-Path -LiteralPath $architecturePath)) {
    throw 'DOC-W3-001 expected architecture bridge record is missing'
}

$requiredTerms = @(
    'contextlab.local-capability-availability.v1',
    'fail-closed',
    'redacted',
    'deterministic ordering',
    'passed',
    'ignored',
    'unobserved',
    'deferred',
    'blocked'
)

$combined = (Get-Content -Raw -LiteralPath $architecturePath) + "`n" + (Get-Content -Raw -LiteralPath $matrixPath)
$missingTerms = $requiredTerms | Where-Object { $combined -notmatch [regex]::Escape($_) }

if ($missingTerms) {
    throw "DOC-W3-001 missing required terms: $($missingTerms -join ', ')"
}
```

## Evidence-State Definitions / 证据状态定义

| Status / 状态 | Meaning in this matrix / 本矩阵中的含义 |
| --- | --- |
| `passed` | This task ran the named read-only command and observed the stated result within the stated local documentation scope. / 本任务运行了所列只读 command，并在所列本地文档范围内观察到该结果。 |
| `failed` | The named command ran and returned a reproducible non-zero result; this is evidence of an open local gate, not an environmental block. / 所列 command 已运行并返回可复现的非零结果；这是尚未收束的本地门禁证据，不是环境阻塞。 |
| `ignored` | The surface is intentionally excluded from this documentation-only command scope; this is not a test-runner result and does not imply success or failure. / 该表面有意排除在仅文档 command 范围之外；这不是 test-runner 结果，也不表示成功或失败。 |
| `unobserved` | No command or reviewable receipt was produced by this task. / 本任务未产生 command 或可审阅回执。 |
| `deferred` | Work is intentionally left for its owning future increment or external evidence gate. / 工作有意留给其所属的未来增量或外部证据门禁。 |
| `blocked` | A requested receipt cannot be produced in this task because a concrete prerequisite is unavailable. / 因具体前置条件不可用，本任务无法产生所请求的回执。 |

## Observed Documentation Receipts / 已观察到的文档回执

The documentation-only entries below are commands personally run from the repository root on 2026-07-19. Their scope is documentation QA only; fresh implementation receipts follow in a separate section.

下列仅文档条目均为 2026-07-19 在仓库根目录亲自运行的 command。其范围仅为文档 QA；新鲜实现回执位于后续独立章节。

| Command or check / 命令或检查 | Observed result / 已观察结果 | Status / 状态 | Established scope / 已建立范围 |
| --- | --- | --- | --- |
| `DOC-W3-001` before the architecture record existed | Exited `1` with `DOC-W3-001 expected architecture bridge record is missing`. / 以 `DOC-W3-001 expected architecture bridge record is missing` 退出 `1`。 | `passed` | Expected red phase: the assertion proved it detects a missing paired record. / 预期 red 阶段：断言证明它能检测缺失的配套记录。 |
| `DOC-W3-001` after this documentation change | Exited `0` with no output. / 以 `0` 退出且无输出。 | `passed` | The paired records contain the required V1, fail-closed, redacted, deterministic-ordering, and five-state vocabulary. / 配套记录包含所需的 V1、fail-closed、redacted、deterministic-ordering 及五状态词汇。 |
| Read-only ownership path inspection for `crates/evaluation`, `crates/workflow`, `crates/knowledge`, `crates/memory`, `crates/mcp`, `crates/plugin-runtime`, `server/api`, `packages/local-sdk`, and `apps/web/src/app` | Each path returned `True`. / 每个路径均返回 `True`。 | `passed` | Confirms only that the documented ownership paths exist locally. / 仅确认文档中的所有权路径在本地存在。 |
| `git status --short` | Exited with `fatal: not a git repository (or any of the parent directories): .git`. / 以 `fatal: not a git repository (or any of the parent directories): .git` 退出。 | `unobserved` | Repository metadata is unusable, so no Git change-set or commit-binding evidence is available; this does not block local product work. / 仓库元数据不可用，因此没有 Git change-set 或 commit-binding 证据；这不阻断本地产品工作。 |

## Fresh Local Integration Receipts / 新鲜本地集成回执

| Command or check / 命令或检查 | Observed result / 已观察结果 | Status / 状态 | Established scope / 已建立范围 |
| --- | --- | --- | --- |
| Focused Core tests for `contextlab-evaluation`, `contextlab-mcp`, `contextlab-plugin-runtime`, `contextlab-workflow`, `contextlab-knowledge`, and `contextlab-memory` | Passed locally. / 本地通过。 | `passed` | Covers sealed benchmark receipt projection, capability V1 schema rejection, provider-free Workflow status projection, and redacted Knowledge/Memory projections. / 覆盖 sealed benchmark receipt projection、capability V1 schema rejection、provider-free Workflow status projection，以及脱敏的 Knowledge/Memory projection。 |
| Strict Clippy for the six Wave 3 Core crates | Passed locally with `-D warnings`. / 使用 `-D warnings` 在本地通过。 | `passed` | Applies only to the owned Core crates, not the repository-wide auth baseline. / 仅适用于所属 Core crate，不代表仓库范围 auth baseline。 |
| `cargo fmt --all -- --check` and `cargo test --workspace --quiet` | Passed locally; API reported `153 passed` and storage reported `166 passed, 36 ignored`. / 本地通过；API 报告 `153 passed`，storage 报告 `166 passed, 36 ignored`。 | `passed` | Fresh workspace formatting, compilation, and test receipt; ignored PostgreSQL tests remain ignored. / 新鲜的 workspace 格式、编译与测试回执；ignored PostgreSQL test 仍保持 ignored。 |
| `cargo test -p contextlab-api local_workflow_capability_status --quiet` | One focused private-route test passed. / 一项聚焦 private-route test 通过。 | `passed` | Confirms the existing local route is authenticated and default-off; it does not promote the route to public OpenAPI. / 确认既有 local route 已认证且默认关闭；不将其提升为 public OpenAPI。 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Failed only at the Rust 1.85 const-context `Vec::is_empty` compatibility lint in `crates/auth/src/authorization.rs:320`. / 仅在 `crates/auth/src/authorization.rs:320` 的 Rust 1.85 const-context `Vec::is_empty` 兼容性 lint 处失败。 | `failed` | A separate quality sidecar owns the minimal behavior-equivalent repair; no passing result is claimed until that command is freshly observed. / 独立质量侧车负责最小且行为等价的修复；在重新观察到该 command 通过前不声称绿灯。 |
| `pnpm check:web` | Public SDK `14`, local SDK `51`, Web `109`, TypeScript checks, and the optimized Next.js build passed. / public SDK `14`、local SDK `51`、Web `109`、TypeScript check 与优化 Next.js build 通过。 | `passed` | Fresh cross-stack Web receipt covering local SDK parsers, same-origin BFF, data/presenter/screen paths, and private route compilation. The default recursive Web glob now collects nested BFF route tests. / 新鲜的跨栈 Web 回执，覆盖 local SDK parser、同源 BFF、data/presenter/screen path 与 private route 编译；默认递归 Web glob 现会收集嵌套 BFF route test。 |
| `scripts/verify-wave2-local-contract.test.sh` and `scripts/verify-wave2-local-contract.sh` | Self-test and static contract checks passed; overall is `unobserved` only because safe Git diff evidence is unavailable. / 自测与静态 contract check 通过；overall 仅因缺少安全 Git diff evidence 为 `unobserved`。 | `passed` | Confirms one application-layer `GraphDiff` calculation in the existing verifier; it does not create Git, Docker, or external evidence. / 确认既有 verifier 中只有一个 application-layer `GraphDiff` calculation；不产生 Git、Docker 或外部证据。 |

## Bridge and Runtime Matrix / 桥接与运行时矩阵

This table reports the recorded local implementation boundary for Wave 3. Later source inspection confirms the private Benchmark discovery and Workflow binding read transports described below. The `passed` labels refer to previously recorded local source/test receipts, not commands rerun by this documentation correction. It is not a project-wide completion or release status.

本表报告 Wave 3 已记录的本地实现边界。后续源码检查确认了下述私有 Benchmark discovery 与 Workflow binding read transport。`passed` 标记指此前记录的本地源码/测试回执，并非本次文档纠正重新运行的 command；它也不是项目范围的完成或发布状态。

| Surface / 表面 | Status / 状态 | Evidence boundary / 证据边界 |
| --- | --- | --- |
| Core deterministic V1 projections in `crates/evaluation/**`, `crates/workflow/**`, `crates/knowledge/**`, `crates/memory/**`, `crates/mcp/**`, and `crates/plugin-runtime/**` | `passed` | Focused tests, strict Clippy, formatting, and workspace tests passed; policy remains Core-owned. / 聚焦 test、strict Clippy、格式与 workspace test 通过；policy 仍由 Core 所有。 |
| Private local API and local-SDK composition in `server/api/**` and `packages/local-sdk/**` | `passed` | Previously recorded local receipts cover Workflow availability, exact-commit Workflow binding summaries, and exact project/Context/commit sealed benchmark-decision discovery as private authenticated reads with fail-closed parsers. Knowledge/Memory and Plugin/MCP domain projections still have no transport claim. / 此前记录的本地回执覆盖 Workflow availability、精确 commit 的 Workflow binding summary，以及精确 project/Context/commit 的 sealed benchmark-decision discovery；它们都是带 fail-closed parser 的 private authenticated read。Knowledge/Memory 与 Plugin/MCP domain projection 仍不声称已有 transport。 |
| Web `data -> presenter -> screen` adaptation in `apps/web/src/app/**` | `passed` | Previously recorded type/unit/build receipts cover the Workflow availability control, commit-scoped binding inspector, and Benchmark discovery/selection adapters. They are source/test receipts only; authenticated browser and visual E2E remain `unobserved`. / 此前记录的 type/unit/build 回执覆盖 Workflow availability control、commit-scoped binding inspector 与 Benchmark discovery/selection adapter。它们只是源码/测试回执；authenticated browser 与 visual E2E 仍为 `unobserved`。 |
| CLI and Desktop staging adapters | `passed` | Prior F+G focused tests and strict Clippy remain the applicable local compatibility receipt; Wave 3 does not change their behavior. / 先前 F+G 的聚焦 test 与 strict Clippy 仍是适用的本地兼容性回执；Wave 3 不改变其行为。 |
| Private Context-to-Workflow source binding and read | `passed` | Previously recorded focused Workflow (`2 passed`), storage (`3 passed`), API (`4 passed`), local SDK (`35 passed`), Web (`84 passed` plus typecheck), and nested BFF (`7 passed`) receipts cover the immutable binding contract and its private redacted read adapters. The transport exists, but no public surface or runtime assertion is claimed; the separate runtime rows remain authoritative. / 此前记录的 Workflow 聚焦测试（`2 passed`）、storage（`3 passed`）、API（`4 passed`）、local SDK（`35 passed`）、Web（`84 passed` 加 typecheck）与嵌套 BFF（`7 passed`）回执覆盖 immutable binding contract 及其私有脱敏 read adapter。transport 已存在，但不声称 public surface 或 runtime assertion；独立运行时行仍是权威状态。 |
| Private sealed benchmark decision discovery and diff selection | `passed` | Fresh workspace evidence covers safe-summary discovery plus exact-commit selection: API `153 passed`, storage `166 passed, 36 ignored`, local SDK `51`, Web `109`, and production Web build. Focused receipts include timestamp `5`, nested BFF security `4`, public exclusion `8`, and selection `5`. No public write or policy/diff calculation was added. / 新鲜 workspace evidence 覆盖 safe-summary discovery 与精确 commit selection：API `153 passed`、storage `166 passed, 36 ignored`、local SDK `51`、Web `109` 与 production Web build。聚焦回执包括 timestamp `5`、嵌套 BFF security `4`、public exclusion `8` 与 selection `5`。不新增 public write 或 policy/diff calculation。 |
| PostgreSQL-backed authenticated runtime | `unobserved` | PostgreSQL adapter source exists, but this documentation correction has no database connection, PostgreSQL-backed authenticated service-flow, or runtime receipt. / PostgreSQL adapter 源码存在，但本次文档纠正没有 database connection、PostgreSQL-backed authenticated service-flow 或 runtime 回执。 |
| Browser visual or authenticated E2E | `unobserved` | No browser, visual, authentication, or end-to-end service-flow receipt is attached to this correction. / 本次纠正未附带 browser、visual、authentication 或端到端 service-flow 回执。 |
| Remote CI and operator rehearsal | `deferred` | These external gates are intentionally outside the current local engineering scope; no remote system or operator-approved rehearsal was used. / 这些外部门禁明确不属于当前本地工程范围；未使用 remote system 或 operator-approved rehearsal。 |
| Release and production promotion | `deferred` | No release or production promotion is claimed or attempted by this local documentation slice. / 本地文档切片不声明也不尝试 release 或 production promotion。 |
| Public REST, OpenAPI, public SDK, and public writes | `deferred` | These surfaces are expressly excluded from the admitted wave and were not edited or exercised. / 这些表面明确排除在已准入波次之外，未被编辑或执行。 |

## Private Workflow Binding Read Documentation Entry / 私有 Workflow Binding 读取文档条目

This row reconciles the Docs/QA record for `docs/superpowers/plans/2026-07-22-private-workflow-binding-read.md` with the later implementation now present in source. It preserves the distinction between previously recorded local source/test receipts and unobserved runtime evidence.

本行将 `docs/superpowers/plans/2026-07-22-private-workflow-binding-read.md` 的 Docs/QA 记录与源码中现存的后续实现对齐，并继续区分此前记录的本地源码/测试回执与未观测的 runtime evidence。

| Increment / 增量 | Passed / 已通过 | Ignored / 已忽略 | Unobserved / 未观测 | Deferred / 延期 |
| --- | --- | --- | --- | --- |
| Private Workflow binding read implementation / 私有 Workflow binding read 实现 | Existing focused records report API `4 passed`, local SDK `35 passed`, Web `84 passed` plus typecheck, and BFF route `7 passed`; the fresh recursive `pnpm check:web` receipt is recorded above. / 既有聚焦记录报告 API `4 passed`、local SDK `35 passed`、Web `84 passed` 加 typecheck，以及 BFF route `7 passed`；新鲜递归 `pnpm check:web` 回执记录在上方。 | PostgreSQL tests remain runner-ignored while no runtime is used; nested BFF tests are no longer excluded by the default recursive Web glob. / 未使用 runtime 时 PostgreSQL test 仍被 runner ignored；嵌套 BFF test 已不再被默认递归 Web glob 排除。 | PostgreSQL-backed authenticated runtime, browser/visual E2E, and Git change-set evidence remain `unobserved`. / PostgreSQL-backed authenticated runtime、browser/visual E2E 与 Git change-set evidence 仍为 `unobserved`。 | Remote CI, operator rehearsal, release, and production promotion are `deferred`; no public REST/OpenAPI/public SDK write, Web mutation, workflow execution, provider call, or second `GraphDiff` calculator is admitted. / remote CI、operator rehearsal、release 与 production promotion 为 `deferred`；不准入 public REST/OpenAPI/public SDK write、Web mutation、Workflow execution、provider call 或第二个 `GraphDiff` calculator。 |

The implementation is present in source and has previously recorded focused local receipts. This Docs/QA correction does not refresh those product-test receipts and does not convert them into PostgreSQL-backed authenticated runtime, browser/visual, remote, operator, release, or production evidence.

实现已存在于源码，并有此前记录的聚焦本地回执。本次 Docs/QA 纠正不刷新这些产品测试回执，也不把它们转换成 PostgreSQL-backed authenticated runtime、browser/visual、remote、operator、release 或 production evidence。

### Fresh Revalidation Receipt / 新鲜复核回执

The current worktree fresh revalidation supersedes the historical counts in the row above for local Workflow binding read: API `4 passed`, storage `3 passed`, local SDK `70 passed`, BFF route `9 passed`, focused Web binding/presenter/inspector `8 passed`, `cargo fmt --all -- --check`, and `pnpm check:web` with public SDK `14`, local SDK `70`, Web `160`, and a successful production build. The newly observed route and Web tests cover typed upstream auth/authorization/availability failures, exact scope, raw-field rejection, five states, and initial accessibility semantics. PostgreSQL-backed authenticated runtime, authenticated browser/visual E2E, and Git change-set evidence remain `unobserved`; remote CI, operator rehearsal, release, and production promotion remain `deferred`. This is a local implementation receipt only.

当前 worktree 的新鲜复核取代上方 row 中关于本地 Workflow binding read 的历史计数：API `4 passed`、storage `3 passed`、local SDK `70 passed`、BFF route `9 passed`、聚焦 Web binding/presenter/inspector `8 passed`、`cargo fmt --all -- --check`，以及 `pnpm check:web`（public SDK `14`、local SDK `70`、Web `160`，并成功完成 production build）。新观察到的 route 与 Web test 覆盖 typed upstream auth/authorization/availability failure、精确 scope、raw-field rejection、五种状态与初始 accessibility 语义。PostgreSQL-backed authenticated runtime、authenticated browser/visual E2E 与 Git change-set evidence 仍为 `unobserved`；remote CI、operator rehearsal、release 与 production promotion 仍为 `deferred`。这只是本地实现回执。

## Final Documentation QA / 最终文档 QA

Run these read-only checks from the repository root after changing the Wave 3 architecture or verification records. The checks are documentation-only: they do not run product tests or runtime probes. The first command is `DOC-W3-001`; it must complete silently after the paired records are present. The link check must return only `True`; the vocabulary check must return bilingual contract and evidence-state lines. The overclaim scan must return no lines.

修改 Wave 3 architecture 或 verification 记录后，在仓库根目录运行以下只读检查。这些检查仅针对文档，不运行 product test 或 runtime probe。第一条 command 是 `DOC-W3-001`；配套记录存在后它必须静默完成。link check 必须只返回 `True`；vocabulary check 必须返回双语 contract 与 evidence-state 行。overclaim scan 不得返回任何行。

```powershell
$architecturePath = 'docs/architecture/wave-3-local-capability-bridges.md'
$matrixPath = 'docs/verification/wave-3-local-capability-matrix.md'

Test-Path -LiteralPath $architecturePath
Test-Path -LiteralPath $matrixPath

rg -n 'Admitted 2026-07-19|已准入的本地能力桥接|contextlab\.local-capability-availability\.v1|fail-closed|redacted|deterministic ordering|passed|ignored|unobserved|deferred|blocked' $architecturePath $matrixPath

$passPrefix = 'pass'
$passSuffix = 'ed'
$sucPrefix = 'suc'
$sucSuffix = 'cess'
$overclaimPattern = "docker.{0,80}($passPrefix$passSuffix|$sucPrefix$sucSuffix)|postgresql.{0,80}($passPrefix$passSuffix|$sucPrefix$sucSuffix)|browser.{0,80}($passPrefix$passSuffix|$sucPrefix$sucSuffix)|remote.{0,80}($passPrefix$passSuffix|$sucPrefix$sucSuffix)|production.{0,80}($passPrefix$passSuffix|$sucPrefix$sucSuffix)"
$overclaimMatches = rg -n -i $overclaimPattern $architecturePath $matrixPath
if ($LASTEXITCODE -eq 1) {
    exit 0
}
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
$overclaimMatches
throw 'Wave 3 documentation contains an external-success overclaim'
```

The two path checks, `DOC-W3-001`, vocabulary check, and empty overclaim scan are the only green documentation QA criteria. They do not elevate any `ignored`, `unobserved`, `deferred`, or `blocked` row.

两个路径检查、`DOC-W3-001`、vocabulary check 与空 overclaim scan 是唯一的 green 文档 QA 条件。它们不会提升任何 `ignored`、`unobserved`、`deferred` 或 `blocked` 行。

## Linked Documents / 关联文档

- [Wave 3 local capability bridge architecture](../architecture/wave-3-local-capability-bridges.md)
- [Local Context lifecycle API boundary](../api/local-context-lifecycle.md)
- [Workspace and private Workflow binding user flow](../user-flows/wave-1-workspace-integration.md)
- [Active long-term goal](../roadmap/active-long-term-goal.md)
- [Private Workflow binding read plan](../superpowers/plans/2026-07-22-private-workflow-binding-read.md)
- [Wave 2 local capability evidence matrix](wave-2-local-capability-evidence-matrix.md)
- [Wave 2 local capability availability contracts](../architecture/wave-2-local-capability-availability.md)
- [Admitted local capability bridge wave plan](../superpowers/plans/2026-07-19-local-capability-bridge-wave.md)
- [Parallel development plan](../roadmap/parallel-development-plan.md)
