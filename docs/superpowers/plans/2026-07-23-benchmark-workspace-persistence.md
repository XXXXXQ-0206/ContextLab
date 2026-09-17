# Benchmark Workspace Persistence Plan / Benchmark Workspace 持久化计划

**Goal / 目标：** Admit the smallest truthful storage dependency for the existing redacted
`BenchmarkWorkspaceProjectionV1`: an immutable repository that joins one sealed benchmark decision
evidence artifact, its deterministic execution receipt, and its exact execution plan, then reads a
single or comparable baseline/revised projection at an exact project/Context/commit/cohort scope. /
为既有脱敏 `BenchmarkWorkspaceProjectionV1` 准入最小且真实的存储依赖：以不可变 repository 绑定一份
sealed benchmark decision evidence、其确定性 execution receipt 与精确 execution plan，并在精确
project/Context/commit/cohort scope 下读取单份或可比较的 baseline/revised projection。

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment advances
completion criterion 3, which requires benchmark run, scorecard, regression, and evaluation-diff
views to be persisted and queryable. It preserves the charter's reproducibility, exact immutable
Context scope, framework-independent repository ports, and secure-by-default redaction boundary. /
本增量推进收束条件 3，其中要求 benchmark run、scorecard、regression 与 evaluation-diff view 可持久化、
可查询；同时保持项目宪章规定的可复现性、精确不可变 Context scope、框架无关 repository port 与默认
安全的脱敏边界。

**Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口：** The prior local-read
increment identified `BenchmarkWorkspaceProjectionV1Reader` as missing because existing evidence
storage does not retain the receipt cohort ID, decision namespace, or dataset/case-to-run provenance
needed by the projection. Deriving them from decision IDs or list positions would invent identity;
rebuilding scorecards or regression policy would change ownership. / 先前的 local-read 增量已确认缺少
`BenchmarkWorkspaceProjectionV1Reader`：既有 evidence storage 未保留 projection 所需的 receipt cohort
ID、decision namespace 与 dataset/case-to-run provenance。若从 decision ID 或列表位置推导，会伪造
identity；若重建 scorecard 或 regression policy，则会越过既有领域所有权。

**Why this is the next dependency-ready choice / 为什么这是当前依赖就绪的选择：** The evaluation
crate already owns the sealed receipt, deterministic plan, redacted V1 projection, and sole
evaluation-diff implementation. Storage already owns sealed decision evidence with authoritative
project/Context/commit scope. Joining those accepted contracts in one append-only memory adapter is
therefore dependency-ready and narrower than transport, Web, provider, or PostgreSQL integration. /
evaluation crate 已拥有 sealed receipt、确定性 plan、脱敏 V1 projection 与唯一 evaluation-diff 实现；
storage 已拥有带权威 project/Context/commit scope 的 sealed decision evidence。因此，在一个 append-only
memory adapter 中绑定这些已接受 contract 已具备依赖，且边界小于 transport、Web、provider 或 PostgreSQL
integration。

**Explicit non-goals / 明确非目标：** No provider or evaluator call, raw benchmark read API, raw
case/model-output projection, policy/scorecard/diff re-evaluation, public or private route, SDK, Web,
existing benchmark storage module change, existing PostgreSQL adapter change, Docker, database
provisioning, secret or environment-file read, release, or production claim. / 不调用 provider 或
evaluator，不增加 raw benchmark read API，不投影 raw case/model output，不重算 policy/scorecard/diff，
不增加 public/private route、SDK 或 Web，不修改既有 benchmark storage module 或 PostgreSQL adapter，
不使用 Docker、不配置数据库、不读取 secret/environment file，也不声明 release 或 production。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Add one
private storage contract and standalone in-memory adapter in
`crates/storage/src/benchmark_workspace_projection.rs`, focused integration tests, one module export,
and this bilingual plan. The write command validates evidence/receipt/plan identity before storage;
the reader accepts exact typed scope and returns only `BenchmarkWorkspaceProjectionV1`. The adapter
stores sealed source contracts privately and exposes no raw-content getter. / 仅新增
`crates/storage/src/benchmark_workspace_projection.rs` 中的私有存储 contract 与独立 memory adapter、
聚焦 integration test、一个 module export 和本双语计划。write command 在存储前校验
evidence/receipt/plan identity；reader 只接受精确 typed scope，并且只返回
`BenchmarkWorkspaceProjectionV1`。adapter 私下保存 sealed source contract，不提供 raw-content getter。

**PostgreSQL decision / PostgreSQL 决策：** PostgreSQL is explicitly deferred. A real adapter would
require wiring in the existing PostgreSQL repository module, which is outside this worker's write
ownership. Adding migration `0019` alone would create an always-unused persistence shape rather than
a truthful adapter, so no migration is admitted in this increment. / PostgreSQL 明确延期。真实 adapter
需要修改当前 worker 无权写入的既有 PostgreSQL repository module；单独新增 `0019` migration 只会产生
始终未被使用的持久化形状，而不是可信 adapter，因此本增量不准入 migration。

**Fresh verification required / 所需新鲜验证：** Observe focused RED from the missing storage
module before implementation. Then run the focused projection repository test, all storage tests,
storage formatting, workspace formatting, and strict storage Clippy over all targets. Verify exact
scope rejection, cross-contract mismatch rejection, deterministic ordering, identical replay,
conflict rejection, and baseline/revised projection without raw-field serialization. Record
PostgreSQL runtime, authenticated transport, browser, remote CI, release, and production evidence as
`unobserved` or `deferred`, never inferred. / 实现前观察由缺失 storage module 导致的聚焦 RED；随后运行
聚焦 projection repository test、全部 storage test、storage formatting、workspace formatting 与覆盖全部
target 的 storage strict Clippy。验证精确 scope 拒绝、跨 contract mismatch 拒绝、确定性排序、相同 replay、
冲突拒绝，以及不序列化 raw field 的 baseline/revised projection。PostgreSQL runtime、authenticated
transport、browser、remote CI、release 与 production evidence 必须记为 `unobserved` 或 `deferred`，不得
推断。

## TDD Steps / TDD 步骤

- [x] Add focused repository contract tests and observe the expected missing-contract RED.
- [x] Implement only the validated immutable record, reader/writer ports, and memory adapter.
- [x] Run focused GREEN and storage regression tests.
- [x] Run formatting and strict storage Clippy.
- [x] Record exact evidence and the remaining PostgreSQL adapter gap.

## Evidence Record / 证据记录

Observed locally on 2026-07-23 without Docker, PostgreSQL provisioning, provider/evaluator calls,
secret or environment-file reads, route registration, SDK/Web changes, browser execution, or raw
benchmark reads: / 2026-07-23 在未使用 Docker、未配置 PostgreSQL、未调用 provider/evaluator、未读取
secret/environment file、未注册 route、未修改 SDK/Web、未执行 browser 且未读取 raw benchmark 的条件
下，本地观察到：

- **RED / 红灯：** `cargo test -p contextlab-storage --test
  benchmark_workspace_projection` failed with `E0432` because the nine requested projection
  repository contracts were absent from `contextlab_storage`. The failure was the intended missing
  dependency, not a fixture or runtime error. / 该命令因九个 projection repository contract 尚未存在而
  以 `E0432` 失败；这是预期的缺失依赖，不是 fixture 或 runtime 错误。
- **Focused GREEN / 聚焦绿灯：** The first implementation run reached `3 passed; 1 failed`; the
  remaining assertion had assumed alphabetical metric order, while the existing domain order is
  `LatencyMs, Accuracy`. Correcting only that test expectation produced `4 passed; 0 failed`; the
  same focused command passed `4/4` again after formatting. / 首次实现运行达到 `3 passed; 1 failed`；
  剩余 assertion 错把 metric 顺序假设为字母序，而既有 domain order 为 `LatencyMs, Accuracy`。仅修正
  测试预期后得到 `4 passed; 0 failed`，格式化后同一聚焦命令再次通过 `4/4`。
- **Storage regression / Storage 回归：** `cargo test -p contextlab-storage` exited `0`: library
  tests reported `166 passed, 36 ignored`; six integration binaries reported
  `22 + 5 + 5 + 4 + 1 + 3 = 40 passed`; doc tests reported no failures. The `36` ignored tests are
  the existing disposable-PostgreSQL cases and were not presented as runtime evidence. / 命令退出码
  为 `0`：library test 为 `166 passed, 36 ignored`；六个 integration binary 合计 `40 passed`；doc test
  无失败。`36` 个 ignored test 是既有 disposable PostgreSQL case，不作为 runtime evidence。
- **Formatting and lint / 格式化与 lint：** `cargo fmt -p contextlab-storage -- --check` passed.
  `cargo clippy -p contextlab-storage --all-targets -- -D warnings` passed. A fresh
  `cargo fmt --all -- --check` remains **blocked** only by unrelated pre-existing drift in
  `crates/workflow/tests/workflow_deterministic_replay.rs`; no outside-owned file was changed. /
  storage 范围格式检查与覆盖全部 target 的 strict Clippy 均通过。workspace-wide formatting 新鲜复查
  仍仅被 `crates/workflow/tests/workflow_deterministic_replay.rs` 中无关的既有漂移阻塞；未修改 ownership
  外文件。
- **Contract evidence / Contract 证据：** Focused tests prove exact project/Context/commit/cohort
  reads, deterministic case and metric ordering, identical replay, changed-receipt conflict,
  cross-contract namespace rejection, existing evaluation-diff delegation, and serialized raw-field
  exclusion. The write constructor reproduces the complete sealed decision evidence from the
  receipt and exact definitions and requires equality with the supplied evidence, so it neither
  invents identity nor re-evaluates policy. / 聚焦测试证明了精确 project/Context/commit/cohort read、
  确定性 case/metric ordering、相同 replay、变更 receipt conflict、跨 contract namespace rejection、
  既有 evaluation-diff delegation 与序列化 raw-field 排除。write constructor 使用 receipt 与精确
  definition 重建完整 sealed decision evidence，并要求与 supplied evidence 相等，因此既不伪造 identity，
  也不重算 policy。
- **Boundary audit / 边界审计：** No `0019_benchmark_workspace_projection_receipts.sql` exists.
  The new source contains no provider/fetch/route/environment/secret access or raw-content getter;
  sealed plans remain private and the reader returns only `BenchmarkWorkspaceProjectionV1`. The
  `model_output` string appears only in a negative redaction assertion. / 不存在 `0019` migration；
  新源码不包含 provider/fetch/route/environment/secret 访问或 raw-content getter。sealed plan 保持私有，
  reader 只返回 `BenchmarkWorkspaceProjectionV1`；`model_output` 仅出现在负向脱敏 assertion 中。
- **Remaining gap / 剩余缺口：** Durable PostgreSQL receipt/provenance persistence and its adapter
  implementation remain **unobserved** and deferred to an increment that owns the existing
  PostgreSQL repository wiring. Authenticated transport and browser evidence are **unobserved**;
  remote CI, operator rehearsal, release, and production promotion remain **deferred** under current
  governance. The invalid workspace `.git` metadata still prevents a trustworthy `git status`
  receipt. / 持久 PostgreSQL receipt/provenance persistence 及其 adapter 实现仍为 **unobserved**，并
  延期至拥有既有 PostgreSQL repository wiring 的增量。authenticated transport 与 browser evidence 为
  **unobserved**；remote CI、operator rehearsal、release 与 production promotion 按当前治理保持
  **deferred**。workspace 的无效 `.git` metadata 仍导致无法取得可信 `git status` 回执。
