# Private Benchmark Multi-Dataset Breadth Receipt / 私有 Benchmark 多 Dataset 宽度回执

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
serves Criterion 3, Benchmark-driven evaluation. Criterion 3 requires reusable suites and
datasets, run details, scorecards, regression thresholds, and evaluation-diff views to be
persisted, queryable, and visible in the Web workspace. The current contracts cover these shapes,
but the full local path lacks a fresh breadth receipt with more than one dataset and multiple cases.

本增量直接服务条件 3“Benchmark-driven evaluation”。条件 3 要求可复用 suite/dataset、run detail、scorecard、回归阈值与
evaluation-diff view 可持久化、可查询并在 Web workspace 可见。当前 contract 已覆盖这些 shape，但完整本地路径缺少一份包含多个
dataset 与多个 case 的新鲜宽度回执。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** Existing green
receipts prove the benchmark authoring, execution, sealed evidence, decision discovery, workspace
projection, and Web parsing contracts separately. The authoring API fixture uses one dataset, and
the execution fixture does not prove that all datasets and cases survive one complete
authoring-to-projection path. Without this evidence, Criterion 3 breadth remains unobserved even
though the underlying domain supports it.

既有绿灯回执分别证明了 benchmark authoring、execution、sealed evidence、decision discovery、workspace projection 与 Web parsing contract。
但 authoring API fixture 只有一个 dataset，execution fixture 也没有证明全部 dataset/case 能在一条完整 authoring-to-projection path 中保留。
没有这份证据，条件 3 的 breadth 仍是未观测，即使底层 domain 已支持该能力。

**Why now / 为什么现在优先:** The local contract verifier, benchmark workflow, and all existing
storage/API/Web boundaries are freshly green. A test-only breadth receipt is the smallest
dependency-ready increment directly tied to an open named criterion; it is more necessary than a
new benchmark feature or unrelated workflow expansion.

当前 local contract verifier、benchmark workflow 与 storage/API/Web 边界均已有新鲜绿灯。仅测试的 breadth 回执是当前最小且依赖
已满足、直接对应开放条件的增量，比新增 benchmark feature 或无关 workflow 扩展更必要。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Add
focused fixtures/tests only in the existing benchmark storage and API test boundaries, reusing the
existing authoring command, execution service, sealed decision/evidence repositories, and redacted
workspace projection. Update this plan and the bilingual roadmap/criteria/parallel ledger with
observed receipts only. No production code is admitted unless a test exposes a real contract defect;
any such defect must be recorded before repair.

仅在既有 benchmark storage 与 API test boundary 增加 focused fixture/test，复用现有 authoring command、execution service、sealed
decision/evidence repository 与脱敏 workspace projection。仅用观测到的回执更新本计划及双语 roadmap/criteria/parallel ledger。
除非测试暴露真实 contract defect，否则不准入 production code；如发现 defect，必须先记录再修复。

**Explicit non-goals / 明确非目标:** No new benchmark domain behavior, provider/evaluator call,
migration, public REST/OpenAPI/public SDK method, Web mutation, UI feature, raw case/input/output
transport, Docker/PostgreSQL runtime, browser/visual claim, secret access, remote CI, operator
rehearsal, release, production promotion, or second GraphDiff calculator. `GraphDiff::between`
remains the sole graph-diff calculator.

不新增 benchmark domain behavior、provider/evaluator call、migration、public REST/OpenAPI/public SDK method、Web mutation、UI feature、
raw case/input/output transport、Docker/PostgreSQL runtime、browser/visual 声明、secret access、remote CI、operator rehearsal、release、
production promotion 或第二个 GraphDiff calculator。`GraphDiff::between` 继续是唯一图谱 diff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:**
Observe red/green focused breadth tests for exact binding scope, all dataset/case coverage,
persisted run/provenance counts, decision-bound workspace query, server-owned scorecard/regression/
evaluation-diff metadata, and raw-payload rejection. Then run the workspace Rust tests, format,
strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, the local contract verifier, and
the one-GraphDiff static check. Keep PostgreSQL/Docker, authenticated browser, Git, remote CI,
operator, release, and production `unobserved` or `deferred` unless directly observed.

先观察 focused breadth test 的红绿结果，覆盖 exact binding scope、全部 dataset/case、持久化 run/provenance count、decision-bound workspace query、
server-owned scorecard/regression/evaluation-diff metadata 与 raw-payload rejection。随后运行 workspace Rust test、format、strict offline Clippy、
锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verifier 与唯一 GraphDiff 静态检查。PostgreSQL/Docker、authenticated browser、Git、
remote CI、operator、release 与 production 若未直接观测，保持 `unobserved` 或 `deferred`。

## Implementation Checklist / 实施清单

- [x] Add the multi-dataset/multi-case fixture in the storage boundary; the focused test is green.
- [x] Add the exact authoring-to-sealed-decision/workspace breadth assertion in the API boundary; the
  authoring, workspace, and decision reads are exercised, while execution itself is intentionally
  invoked through the real service because its adapter wiring is `pub(crate)` in this test boundary.
- [x] Prove deterministic ordering, full coverage, redacted projection, scorecard metadata,
  regression status, and baseline/revised evaluation-diff metadata in the storage fixture. The
  new breadth fixtures have positive exact-scope assertions and a wrong-project fail-closed read;
  existing focused scope tests cover the remaining dimensions.
- [x] Run fresh verification and update the bilingual roadmap records without closing Criterion 3.

- [x] 在 storage boundary 增加 multi-dataset/multi-case fixture，focused test 已通过。
- [x] 在 API boundary 增加 exact authoring-to-sealed-decision/workspace breadth assertion；执行阶段因
  adapter wiring 为 `pub(crate)`，明确使用真实 service，而不是把它写成 execution POST 证据。
- [x] 证明确定性排序、完整 coverage、脱敏 projection、scorecard/regression/evaluation-diff metadata（storage）与
  exact scope；新增 fixture 覆盖正向 scope 和错误 project 的 fail-closed read，其余维度由既有 focused
  scope tests 覆盖。
- [x] 运行新鲜验证并更新双语路线图记录，不关闭条件 3。

## Evidence Boundary / 证据边界

This is a local test/evidence increment. A green fixture demonstrates breadth of the in-memory
contract path only; it does not claim PostgreSQL runtime, authenticated browser, external CI,
release, or production readiness. The long-term goal remains active.

本增量是本地测试/证据增量。绿 fixture 只证明 memory contract path 的 breadth，不代表 PostgreSQL runtime、authenticated browser、external CI、
release 或 production readiness。长期目标保持 active。

## Fresh Receipt / 新鲜回执（2026-08-01）

The storage breadth fixture and API breadth fixture each passed `1 passed`. The storage fixture
proves two datasets and four cases across baseline/revised exact commits, deterministic ordering,
idempotent replay, four persisted runs per cohort, scorecard/regression metadata, baseline/revised
evaluation-diff metadata, decision-bound projection resolution, and redacted projection markers.
The API fixture proves one revised exact commit with two datasets/four cases through protected
authoring, workspace, and decision-workspace reads. Its execution step calls the real
`BenchmarkExecutionService` directly; it is not evidence for the execution POST adapter, API-level
baseline/revised comparison, or full REST execution transport.

storage breadth fixture 与 API breadth fixture 均为 `1 passed`。storage fixture 证明 baseline/revised 精确 commit 上的两个
dataset、四个 case、确定性排序、幂等 replay、每 cohort 四个持久化 run、scorecard/regression metadata、baseline/revised
evaluation-diff metadata、decision-bound projection resolution 与脱敏 projection marker。API fixture 证明一个 revised 精确 commit 上
受保护的 authoring、workspace 与 decision-workspace read 保留两个 dataset/四个 case。其 execution 阶段直接调用真实
`BenchmarkExecutionService`；这不是 execution POST adapter、API-level baseline/revised comparison 或完整 REST execution transport 的证据。

Fresh commands observed / 已观测的新鲜命令：

- `cargo test -p contextlab-storage --test benchmark_breadth --offline -- --nocapture`: `1 passed`.
- `cargo test -p contextlab-api --test benchmark_breadth --offline -- --nocapture`: `1 passed`.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build.
- `powershell -NoProfile -File scripts/verify-local-contracts.ps1 -Root .`: local source, graph, benchmark route,
  safe DTO, and public-boundary checks passed; `overall=unobserved` because no unified diff input was supplied.
- Static production scan: `GRAPH_DIFF_IMPL_COUNT=1`.

These are local memory/test and protected-read receipts only. PostgreSQL/Docker runtime, authenticated browser/visual smoke,
Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; no public write,
OpenAPI/SDK write, Web mutation, migration, provider, secret access, or second `GraphDiff` calculator was added. Criterion 3
and the long-term goal remain open.

这些仅是本地 memory/test 与 protected-read 回执。PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未新增 public write、OpenAPI/SDK write、Web mutation、migration、
provider、secret access 或第二个 `GraphDiff` calculator。条件 3 与长期目标继续开放。
