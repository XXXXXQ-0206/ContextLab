# Private Benchmark Vertical Breadth Completion / 私有 Benchmark 垂直宽度收束

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
serves Criterion 3, Benchmark-driven evaluation. Criterion 3 requires reusable datasets and
suites, persisted run details, scorecards, regression thresholds, evaluation diffs, and a visible
Web workspace. The storage breadth receipt is fresh, but the API comparison and Web breadth
receipts are still incomplete.

本增量直接服务条件 3“Benchmark-driven evaluation”。条件 3 要求可复用 dataset 与 suite、持久化 run detail、scorecard、regression
threshold、evaluation diff 与可见的 Web workspace。storage breadth 回执已新鲜取得，但 API comparison 与 Web breadth 回执仍不完整。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The API breadth
fixture currently proves one revised exact commit and calls the real execution service directly;
it does not prove a baseline/revised comparison at the API test boundary. Existing Web tests prove
the presenter and screen shapes with smaller fixtures, but do not prove that two datasets and four
case/run rows survive the full data -> presenter -> screen path with scorecard, regression, and
evaluation-diff metadata.

当前 API breadth fixture 只证明一个 revised 精确 commit，并直接调用真实 execution service；它没有证明 API test boundary 上的
baseline/revised comparison。现有 Web tests 用较小 fixture 证明 presenter 与 screen shape，但没有证明两个 dataset、四个 case/run row
以及 scorecard、regression、evaluation-diff metadata 能完整穿过 data -> presenter -> screen。

**Why now / 为什么现在优先:** This is the smallest dependency-ready evidence increment after
the storage receipt. It closes the exact open breadth questions named in the current Criterion 3
record without adding production behavior, transport, or mutation. A broader benchmark feature or
new Context surface would be premature while these existing contracts lack matching breadth proof.

这是 storage receipt 之后最小且依赖就绪的证据增量，直接收束当前条件 3 中已命名的 breadth 问题，不新增 production behavior、transport 或 mutation。
在既有 contract 尚缺匹配 breadth proof 时，扩展 benchmark feature 或新增 Context surface 均不必要。

**Smallest affected boundary and ownership / 最小受影响边界与所有权:**

- API worker: only `server/api/tests/benchmark_breadth.rs`; extend the in-memory test fixture to
  execute baseline and revised cohorts and assert the existing protected comparison/read path.
- Web worker: only existing `apps/web/src/app/local-benchmark-workspace-*.test.*` files; reuse
  existing data, presenter, and screen modules and assert the same two-dataset/four-case payload.
- Integration Lead: this plan and bilingual roadmap records only. No two workers may edit the same
  file. Production code is admitted only if a test exposes a real contract defect, which must be
  recorded before repair.

- API worker：仅负责 `server/api/tests/benchmark_breadth.rs`，扩展 in-memory fixture 执行 baseline/revised cohort，并断言既有 protected comparison/read path。
- Web worker：仅负责现有 `apps/web/src/app/local-benchmark-workspace-*.test.*`，复用既有 data、presenter、screen module，断言相同的双 dataset/四 case payload。
- Integration Lead：仅负责本计划与双语路线图。两个 worker 不得编辑同一文件。只有测试暴露真实 contract defect 时才允许改 production code，且必须先记录。

**Explicit non-goals / 明确非目标:** No execution POST adapter claim, new API route, OpenAPI or
SDK method, Web mutation, provider/evaluator call, raw case/input/output transport, production
code refactor, migration, Docker/PostgreSQL runtime, authenticated browser or visual claim, secret
access, Git/remote CI/operator/release/production evidence, or second `GraphDiff` calculator.
`GraphDiff::between` remains the sole graph-diff calculator.

不声称 execution POST adapter；不新增 API route、OpenAPI 或 SDK method、Web mutation、provider/evaluator call、raw case/input/output transport、
production code refactor、migration、Docker/PostgreSQL runtime、authenticated browser 或 visual claim、secret access、Git/remote CI/operator/release/production
evidence，也不新增第二个 `GraphDiff` calculator。`GraphDiff::between` 继续是唯一 graph-diff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Observe
focused API and Web red/green breadth tests, then run `cargo fmt --all -- --check`, the offline
workspace Rust tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, the
local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. Record API direct-service versus REST
transport boundaries separately. Keep PostgreSQL/Docker, browser/visual, Git, remote CI, operator,
release, and production facts `unobserved` or `deferred`.

先观察 API 与 Web focused breadth test 的 red/green 结果，再运行 `cargo fmt --all -- --check`、offline workspace Rust tests、strict offline Clippy、锁定
Rust `1.85.0` check、`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。单独记录 API direct-service 与 REST transport 边界。
PostgreSQL/Docker、browser/visual、Git、remote CI、operator、release 与 production 事实保持 `unobserved` 或 `deferred`。

## Implementation Checklist / 实施清单

- [x] API baseline/revised comparison fixture and focused test.
- [x] Web two-dataset/four-case data -> presenter -> screen fixture and focused test.
- [x] Fresh verification and bilingual roadmap receipt; Criterion 3 and the long-term goal remain open.

- [x] API baseline/revised comparison fixture 与 focused test。
- [x] Web 双 dataset/四 case data -> presenter -> screen fixture 与 focused test。
- [x] 新鲜验证与双语路线图回执；条件 3 与长期目标继续开放。

## Evidence Boundary / 证据边界

This is a local test/evidence increment only. A green result does not prove PostgreSQL runtime,
authenticated browser, external CI, release, or production readiness. The long-term goal remains
active.

本增量仅是本地 test/evidence。绿灯不代表 PostgreSQL runtime、authenticated browser、external CI、release 或 production readiness。长期目标保持 active。

## Fresh Receipt / 新鲜回执（2026-08-01）

The API fixture now executes baseline and revised cohorts through the real in-memory
`BenchmarkExecutionService`, replays both without additional evaluator calls, reads the protected
workspace comparison and decision-workspace comparison, and asserts two datasets/four cases,
deterministic rows, scorecard coverage, `passed -> regressed` evaluation diff, exact commit/cohort
scope, and recursive redaction. It still does not exercise the execution POST adapter.

Web inspector tests now pass a matching two-dataset/four-case redacted projection through the
existing data -> presenter -> screen lifecycle and assert stable dataset/run ordering, four-run
coverage, regression/evaluation-diff presentation, bilingual accessible status semantics, and raw
payload exclusion. This is a mocked/local Web contract receipt, not authenticated browser or
producer-to-browser runtime evidence.

API fixture 现通过真实 in-memory `BenchmarkExecutionService` 执行 baseline 与 revised cohort，replay 两者且不产生额外 evaluator call，读取受保护的
workspace comparison 与 decision-workspace comparison，并断言两个 dataset/四个 case、确定性 row、scorecard coverage、`passed -> regressed`
evaluation diff、精确 commit/cohort scope 与递归脱敏；仍未执行 execution POST adapter。

Web inspector test 现将匹配的双 dataset/四 case 脱敏 projection 穿过既有 data -> presenter -> screen lifecycle，并断言稳定 dataset/run 排序、四 run
coverage、regression/evaluation-diff presentation、双语 accessible status 语义与 raw payload 排除。这是 mocked/local Web contract 回执，不是
authenticated browser 或 producer-to-browser runtime evidence。

Fresh commands observed / 已观测的新鲜命令：

- `cargo test -p contextlab-api --test benchmark_breadth --offline -- --nocapture`: `1 passed`.
- `pnpm --filter @contextlab/web exec tsx --test src/app/local-benchmark-workspace-inspector.test.tsx`: `6 passed`.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build.
- `powershell -NoProfile -File scripts/verify-local-contracts.ps1 -Root .`: source, graph, safe DTO,
  protected route/catalog, and public-boundary checks passed; `overall=unobserved` without unified diff input.
- Static scan: `GRAPH_DIFF_IMPL_COUNT=1`.

No production code, public route, OpenAPI/SDK method, Web mutation, migration, provider, secret,
PostgreSQL/Docker runtime, authenticated browser/visual, Git, remote CI, operator, release,
production claim, or second `GraphDiff` calculator was added. Criterion 3 and the long-term goal
remain open; the next increment requires a new bilingual Necessity Record.

未新增 production code、public route、OpenAPI/SDK method、Web mutation、migration、provider、secret、PostgreSQL/Docker runtime、authenticated browser/visual、
Git、remote CI、operator、release、production 声明或第二个 `GraphDiff` calculator。条件 3 与长期目标继续开放；下一增量必须先有新的双语 Necessity Record。
