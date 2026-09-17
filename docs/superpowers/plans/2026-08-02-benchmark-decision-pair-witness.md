# Private Benchmark Decision-Pair Witness / 私有 Benchmark Decision-Pair 见证

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与宪章原则

- **Criterion 2 / 条件 2:** version-backed comparisons must remain bound to exact immutable Context commit identities.
  / **条件 2：**基于版本的比较必须绑定精确、不可变的 Context commit identity。
- **Criterion 3 / 条件 3:** benchmark evidence must be queryable and comparable through a redacted, reusable local workflow.
  / **条件 3：**benchmark evidence 必须通过脱敏、可复用的本地 workflow 被查询和比较。
- **Charter / 宪章：** server-owned Rust/storage facts remain authoritative; adapters validate and present without recomputing policy or Diff.
  / **宪章：** server-owned Rust/storage fact 保持权威；adapter 只校验和呈现，不重新计算 policy 或 Diff。

### Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口

The protected decision-bound benchmark workspace already resolves the requested `(project, Context,
commit, decision)` pair to an immutable cohort and redacted projection, but its response exposes only
`commit_id` and `cohort_id`. A consumer therefore cannot fail closed if a server-side resolver or
adapter accidentally maps the correct cohort to the wrong decision. The direct decision-diff response
has exact decision scope, but the workspace response lacks the same server-owned identity witness.

现有受保护 decision-bound benchmark workspace 已将请求的 `(project, Context, commit, decision)` pair 解析到不可变
cohort 与脱敏 projection，但 response 只暴露 `commit_id` 与 `cohort_id`。因此当 server resolver 或 adapter 错误地把正确
cohort 映射到错误 decision 时，consumer 无法 fail closed。direct decision-diff response 已有精确 decision scope，
但 workspace response 缺少同等的 server-owned identity witness。

### Why now / 为什么现在

The direct multi-dataset decision-diff receipt is now fresh and proves the underlying sealed evidence
and comparison engine. The missing witness is the smallest next contract repair because it reuses the
existing decision resolver, response envelope, local SDK, BFF, and Web `data -> presenter -> screen`
boundary. It is more necessary than another benchmark feature or a public transport, and it prevents
future workspace consumers from silently trusting cohort identity as decision identity.

direct multi-dataset decision-diff 回执现已新鲜通过，底层 sealed evidence 与 comparison engine 已有证据。缺失 witness
是当前最小的下一项 contract repair：它复用既有 decision resolver、response envelope、local SDK、BFF 与 Web
`data -> presenter -> screen` boundary。它比继续增加 benchmark feature 或 public transport 更必要，可防止未来 workspace
consumer 将 cohort identity 静默当作 decision identity。

### Explicit non-goals / 明确非目标

- No new benchmark execution, evaluator, policy, dataset, scorecard, or Diff algorithm.
  / 不新增 benchmark execution、evaluator、policy、dataset、scorecard 或 Diff algorithm。
- No public REST/OpenAPI/public SDK method, public write, Web mutation, operator transport, migration, provider, or scheduler.
  / 不新增 public REST/OpenAPI/public SDK method、public write、Web mutation、operator transport、migration、provider 或 scheduler。
- No raw cases, inputs, expected outputs, measurements, model outputs, or secrets cross the response boundary.
  / raw case、input、expected output、measurement、model output 或 secret 不得穿过 response boundary。
- No PostgreSQL/Docker, authenticated browser, remote CI, operator, release, or production claim.
  / 不声称 PostgreSQL/Docker、authenticated browser、remote CI、operator、release 或 production 已通过。
- `GraphDiff::between` remains the sole graph-diff calculator.
  / `GraphDiff::between` 继续是唯一 graph-diff calculator。

### Smallest affected boundary and bilingual documentation / 最小边界与双语文档

- Private local API response: optional `decision_pair_witness` only for a decision-bound comparison.
  / private local API response：仅在 decision-bound comparison 返回可选 `decision_pair_witness`。
- Non-public local SDK parser/client: strict schema, exact project/Context/commit/decision matching, fail-closed missing or drifted witness.
  / 非公开 local SDK parser/client：严格 schema、精确 project/Context/commit/decision matching，缺失或漂移 witness 时 fail closed。
- Existing Web adapter/presenter/screen: consume server-owned witness as facts; no local mapping or calculation.
  / 既有 Web adapter/presenter/screen：把 server-owned witness 当作 fact 消费，不进行本地 mapping 或计算。
- Bilingual plan, API contract note, completion receipt, and parallel-development ledger.
  / 双语 plan、API contract note、completion receipt 与 parallel-development ledger。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

```powershell
cargo test -p contextlab-api --test benchmark_breadth --offline -- --nocapture
cargo test -p contextlab-api --lib local_benchmark_workspace --offline -- --nocapture
pnpm --filter @contextlab/local-sdk lint
pnpm --filter @contextlab/local-sdk test
pnpm --filter @contextlab/web lint
pnpm --filter @contextlab/web test
cargo fmt --all -- --check
cargo test --workspace --quiet --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo +1.85.0 check --workspace --all-targets --locked --offline
pnpm check:web
```

The local verifier, PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator,
release, and production facts must remain explicitly `passed`, `ignored`, `unobserved`, or `deferred`;
local tests cannot substitute for external evidence.

local verifier、PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator、release 与 production
事实必须明确记录为 `passed`、`ignored`、`unobserved` 或 `deferred`；本地测试不能替代 external evidence。
## Implementation Receipt / 实施回执

Status: completed locally; the long-term ContextLab goal remains active. Fresh API, SDK, Web, Rust,
format, Clippy, locked Rust 1.85, and local fixture-verifier evidence passed. PostgreSQL runtime,
Docker, browser/visual, Git, remote CI, operator, release, and production evidence remain explicitly
ignored, unobserved, or deferred.

状态：已在本地完成；ContextLab 长期目标继续 active。新鲜 API、SDK、Web、Rust、format、Clippy、锁定 Rust 1.85
与本地 fixture-verifier evidence 通过。PostgreSQL runtime、Docker、browser/visual、Git、remote CI、operator、release
与 production evidence 继续明确标记为 `ignored`、`unobserved` 或 `deferred`。

## Integrated Focused Receipt / 集成 focused 回执

The Integration Lead reran the decision-pair witness boundary after the history increment. The
benchmark API breadth contract reported `1 passed`; the API local benchmark workspace unit filter
reported `10 passed`; and `pnpm --filter @contextlab/local-sdk test` reported `148 passed, 0 failed`.
The previously recorded full `pnpm check:web` also passed with Web `298 passed` and a production
build. No new implementation was admitted because the server-owned witness, strict local SDK
parser, and Web adapter already satisfy this plan's boundary.

提交历史增量完成后，Integration Lead 重新运行 decision-pair witness boundary。benchmark API breadth contract 报告
`1 passed`；API local benchmark workspace unit filter 报告 `10 passed`；`pnpm --filter @contextlab/local-sdk test` 报告
`148 passed, 0 failed`。此前记录的完整 `pnpm check:web` 也以 Web `298 passed` 与 production build 通过。由于
server-owned witness、strict local SDK parser 与 Web adapter 已满足本计划边界，本轮没有新增 implementation 准入。

This is local contract evidence only. PostgreSQL runtime, Docker, browser/visual, Git, remote CI,
operator, release, and production remain `ignored`, `unobserved`, or `deferred`; the long-term goal
remains active. The next implementation requires a new bilingual Necessity Record after open-criteria
audit.

本回执仅是本地 contract evidence。PostgreSQL runtime、Docker、browser/visual、Git、remote CI、operator、release 与 production
继续为 `ignored`、`unobserved` 或 `deferred`；长期目标保持 active。下一项 implementation 仍需在开放条件审计后新增双语
Necessity Record。
