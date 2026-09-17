# Private Atomic Branch-Head Graph Witness / 私有原子 Branch-Head Graph Witness

## Contract / 契约

`ContextGraphBranchHeadReviewWitnessRepository` is a private reusable Rust storage contract. It
accepts an exact source `(ProjectId, ContextId, CommitId)` and a typed `BranchName`; it returns an
immutable `ContextGraphBranchHeadReviewWitness` whose target commit is selected by the complete
validated `CommitHistory`. Unknown and unborn branches fail closed, and a target that does not equal
the selected head is rejected as an invariant violation.

`ContextGraphBranchHeadReviewWitnessRepository` 是 private、可复用的 Rust storage contract。它接收 exact source
`(ProjectId, ContextId, CommitId)` 与 typed `BranchName`，返回 immutable `ContextGraphBranchHeadReviewWitness`；target commit
必须由完整、validated `CommitHistory` 选择。unknown/unborn branch 会 fail closed，target 与 selected head 不一致时作为不变量
违反被拒绝。

## Observation Boundary / 观察边界

- Memory reads commit records, branch heads, and source/selected-target snapshots under one `RwLock` read guard.
- PostgreSQL reads the same records under one `REPEATABLE READ READ ONLY` transaction.
- The branch-bound repository does not call the split-port branch-head adapter and does not infer a target from a caller-supplied commit.

- Memory 在一个 `RwLock` read guard 下读取 commit record、branch head、source 与 selected-target snapshot。
- PostgreSQL 在一个 `REPEATABLE READ READ ONLY` transaction 下读取相同记录。
- branch-bound repository 不调用 split-port branch-head adapter，也不从调用方提交的 commit 推断 target。

## Diff Ownership / Diff 所有权

The witness only validates selection and exact scope. `PersistedContextGraphWitnessReviewService`
passes its immutable pair to the existing versioned graph-review projection, where
`GraphDiff::between` remains the sole graph-diff calculator. No API, SDK, BFF, Web, or storage layer
reimplements graph comparison.

witness 只负责 selection 与 exact scope validation。`PersistedContextGraphWitnessReviewService` 将 immutable pair 交给既有
versioned graph-review projection；`GraphDiff::between` 仍是唯一 graph-diff calculator。API、SDK、BFF、Web 与 storage layer
均不重算 graph comparison。

## Evidence Boundary / 证据边界

The local Memory behavior tests and PostgreSQL SQL-shape tests prove the contract shape only. They do
not prove live PostgreSQL, Docker, authenticated browser/visual smoke, Git, remote CI, operator
rehearsal, release, or production readiness. No public REST/OpenAPI/public SDK route or method,
mutation, migration, provider, secret access, or operator transport is part of this boundary.

本地 Memory 行为测试与 PostgreSQL SQL-shape 测试只证明 contract shape，不证明 live PostgreSQL、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production readiness。本边界不包含 public
REST/OpenAPI/public SDK route 或 method、mutation、migration、provider、secret access 或 operator transport。
