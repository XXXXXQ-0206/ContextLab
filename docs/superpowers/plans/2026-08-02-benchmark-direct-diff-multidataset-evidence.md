# Benchmark Direct Diff Multi-Dataset Evidence / Benchmark 直接 Diff 多 Dataset 证据

## Necessity Record / 必要性记录

### Named criterion / 对应完成条件

- **Criterion 3 / 条件 3:** benchmark-driven evaluation must persist, query, compare, and expose
  redacted evaluation evidence across reusable datasets.
  / **条件 3：** benchmark 驱动评测必须能够跨可复用 dataset 持久化、查询、比较并暴露脱敏 evaluation evidence。

This increment supplies one missing decision gate for Criterion 3. It does not close Criterion 3,
the remaining completion criteria, or the active long-term goal.

本增量只补齐条件 3 的一个缺失 decision gate。不关闭条件 3、其余完成条件或 active long-term goal。

### Dependency, gap, and risk / 依赖、缺口与风险

The private benchmark workspace already has local multi-dataset persistence, replay, scorecard,
regression, and evaluation-diff coverage. The direct protected benchmark decision-diff read still
uses a single-dataset fixture, so the API comparison path has no fresh receipt proving aggregation
of the same two-dataset sealed evidence. Without this test, workspace breadth and direct decision-diff
coverage can drift apart while both remain individually green.

现有 private benchmark workspace 已覆盖 local multi-dataset persistence、replay、scorecard、regression 与
evaluation-diff。受保护的 direct benchmark decision-diff read 仍使用 single-dataset fixture，因此 API comparison
path 缺少证明同一组 two-dataset sealed evidence 被正确聚合的新鲜回执。没有该测试，workspace breadth 与 direct
decision-diff coverage 可能各自绿色但发生漂移。

### Why now / 为什么现在

The required execution service, sealed evidence projection, exact pair scope, protected route,
redaction rules, and local test fixtures already exist. A test-only API receipt is the smallest
dependency-ready increment that directly closes the named evidence gap; it is more targeted than
adding a new transport, Web surface, dashboard, or decision-pair schema.

所需的 execution service、sealed evidence projection、exact pair scope、protected route、redaction rules 与 local
fixtures 均已存在。test-only API receipt 是直接收束该命名证据缺口的最小依赖就绪增量，比新增 transport、Web surface、
dashboard 或 decision-pair schema 更聚焦。

### Explicit non-goals / 明确非目标

- No new domain behavior, route, OpenAPI, public SDK method, Web mutation, migration, provider,
  dashboard, or per-dataset scoring surface.
  / 不新增 domain behavior、route、OpenAPI、public SDK method、Web mutation、migration、provider、dashboard 或 per-dataset scoring surface。
- No PostgreSQL, Docker, browser, remote CI, operator, release, or production claim.
  / 不声称 PostgreSQL、Docker、browser、remote CI、operator、release 或 production 已通过。
- No raw cases, inputs, expected outputs, measurements, or model outputs cross the redacted API
  response boundary.
  / raw cases、inputs、expected outputs、measurements 或 model outputs 不得穿过脱敏 API response boundary。

### Smallest boundary and ownership / 最小边界与 ownership

The implementation boundary is one API integration test in
`server/api/tests/benchmark_breadth.rs`, using the existing two-dataset sealed execution fixture.
The test must call the protected direct decision-diff read and assert exact project/Context/baseline/revised
commit+decision scope, `sample_count=4`, `required_sample_count=4`, the expected passed-to-regressed
status and accuracy modification, deterministic response shape, and recursive redaction.

实现边界是 `server/api/tests/benchmark_breadth.rs` 中的一个 API integration test，复用已有 two-dataset sealed
execution fixture。测试必须调用 protected direct decision-diff read，并断言精确 project/Context/baseline/revised
commit+decision scope、`sample_count=4`、`required_sample_count=4`、预期 passed-to-regressed status 与 accuracy modification、
deterministic response shape 以及递归脱敏。

### Fresh verification required / 开始下一增量前所需新鲜验证

```powershell
cargo test -p contextlab-api --test benchmark_breadth --offline -- --nocapture
cargo test -p contextlab-api --lib local_benchmark_decision_diff --offline -- --nocapture
cargo test -p contextlab-storage --test benchmark_breadth --offline -- --nocapture
cargo test -p contextlab-evaluation --test benchmark_decision_diff --offline -- --nocapture
cargo fmt --all -- --check
```

Only after those checks pass may the receipt be added to the roadmap. Full workspace/Web gates remain
required after integration; external release evidence is deferred and outside this local increment.

只有上述检查通过后，才能将回执写入路线图。集成后仍必须运行 full workspace/Web gates；external release evidence 继续延期，
不属于本地增量。
