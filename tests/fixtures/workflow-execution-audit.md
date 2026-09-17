# Workflow Execution Producer/Repository Audit

Date: 2026-08-01
Owner: Luna governance/QA
Scope: `docs/roadmap/` and test evidence only. No source, public contract, secret, or external/production evidence was changed or inferred.

## Finding

The three roadmap records are consistent and conservative. They all describe the same current
boundary: the Workflow execution-status projection and private read path are locally verified, but
the default runtime has no execution repository and no execution producer or persistence path.

- `docs/roadmap/active-long-term-goal.md:166-179` records the read/replay projection and says the
  default runtime has no execution repository.
- `docs/roadmap/completion-criteria.md:125-148` records the same boundary and explicitly says no
  execution producer or persistence path is claimed.
- `docs/roadmap/parallel-development-plan.md:150-176` closes only the read-status wave and says
  no producer or persistence path was introduced.

The smallest evidence gap that directly serves a named completion criterion is **Criterion 1**
(`docs/roadmap/completion-criteria.md:345-346`): there is no local receipt proving a Workflow
execution producer writes a run/projection to a repository and that the same exact
Context/Workflow/run identity can be read back and replayed. Existing evidence is domain-only or
fail-closed unavailable evidence:

- `crates/workflow/tests/workflow_execution_status_projection.rs:42-69` constructs an execution in
  memory and projects its log; `:148-164` projects a replay log; `:167-220` checks validation
  failures. These tests do not cross a repository boundary.
- `server/api/src/workflow_execution.rs:214-255` defines a read-only repository port and an
  unavailable default adapter; `:326-335` proves only the unavailable result.
- `server/api/src/lib.rs:564-572` provides repository injection, while `:6302-6346` verifies the
  default protected route is private but returns `503 workflow_execution_status_unavailable`.

The focused command `cargo test -p contextlab-workflow --tests --offline` was green, including the
15 deterministic replay tests and 5 execution-status projection tests. This validates the existing
domain/projection boundary, not producer-to-repository persistence.

## Minimal Necessity Record

The next admitted increment should record these fields before implementation:

- `record_id`: stable bilingual identifier, for example `workflow-execution-producer-repository-v1`
- `date` and `owner`: date, accountable owner, and disjoint files/tests
- `criterion`: `Criterion 1 - Context-first platform coverage`
- `gap`: no producer-to-repository write/read/replay receipt for Workflow executions
- `necessity`: why the receipt is required to establish the Workflow persistence contract without
  claiming the whole criterion or long-term goal is complete
- `minimal_scope`: one private, provider-free execution producer plus one repository contract for
  exact `(ContextId, Workflow binding/revision, run_id)`; create, exact read-back, idempotent replay,
  scope mismatch rejection, and redacted projection are sufficient
- `evidence_command`: the exact focused test command and expected case counts
- `acceptance_receipt`: producer write, repository read-back, replay identity, and fail-closed
  conflict results tied to the same exact scope
- `boundary`: no public start route, public SDK/OpenAPI mutation, provider/network call, PostgreSQL
  runtime, browser E2E, release, or production claim
- `status`: `unobserved` until the receipt exists; current long-term goal remains `active`

This is a documentation/test evidence gap, not a defect in the three current roadmap descriptions.

## Outcome / 结果

The admitted follow-up is now locally verified in
`docs/superpowers/plans/2026-08-01-private-workflow-execution-status-repository.md`: a provider-free
producer validates `WorkflowExecutionStatusProjectionV1`, the storage repository provides exact
Context/run create, replay, and conflict semantics, and the private API adapter reads only the
redacted projection. This fixture remains the independent pre-implementation audit receipt; its
original `unobserved` status describes the state before implementation and must not be read as
external, PostgreSQL, browser, release, or production evidence.

准入的后续工作现已在
`docs/superpowers/plans/2026-08-01-private-workflow-execution-status-repository.md` 中完成本地验证：provider-free producer 校验
`WorkflowExecutionStatusProjectionV1`，storage repository 提供 exact Context/run 的 create、replay 与 conflict 语义，private API adapter 只读取脱敏 projection。本 fixture 仍是实施前的独立审查回执；其中原有 `unobserved` 状态描述的是实施前事实，不得被理解为 external、PostgreSQL、browser、release 或 production evidence。
