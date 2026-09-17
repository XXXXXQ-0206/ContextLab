# Wave 1 Contribution and QA Contract / Wave 1 贡献与 QA 契约

## Ownership / 所有权

For this Wave 1 H task, the exclusive write boundary is `docs/**`. Do not edit product code, `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`, scripts, `.github`, workflow files, or another agent's files. Existing concurrent work is preserved; never revert it.

对于本次 Wave 1 H 任务，独占写入边界是 `docs/**`。不得编辑产品代码、`Cargo.toml`、`package.json`、`pnpm-workspace.yaml`、scripts、`.github`、workflow 文件或其他 Agent 的文件。必须保留并发工作，绝不回退。

The authoritative ownership table is `docs/roadmap/parallel-development-plan.md`. The architecture, API, user-flow, ADR, and verification pages for H are linked from that table and must remain bilingual.

权威所有权表位于 `docs/roadmap/parallel-development-plan.md`。H 的 architecture、API、user-flow、ADR 与 verification 页面由该表链接，并且必须保持中英双语。

## Contribution Record / 贡献记录

Every Wave 1 contract contribution records:

每次 Wave 1 contract contribution 都必须记录：

- the owning agent and exact file boundary;
- owner 与精确文件边界；
- the typed contract, version, scope, ordering, and integration consumer;
- 类型化 contract、版本、作用域、排序与 integration consumer；
- focused validation and its exact output classification;
- 聚焦验证及其准确输出分类；
- the public/private/fixture/unavailable surface;
- public/private/fixture/unavailable surface；
- remaining ignored, unobserved, blocked, or inconclusive work;
- 剩余的 ignored、unobserved、blocked 或 inconclusive 工作；
- links to the architecture, API, user-flow, ADR, and verification record when the change crosses a boundary.
- 当变更跨越边界时，链接 architecture、API、user-flow、ADR 与 verification 记录。

Source existence is not a passed integration receipt. A compile-only check is not runtime evidence. A local preview is not a live provider or authenticated browser flow.

源代码存在不等于集成回执通过。仅编译检查不等于 runtime evidence。本地 preview 不等于 live provider 或 authenticated browser flow。

## Status Vocabulary / 状态词汇

| Status / 状态 | Required wording / 必须表达 |
| --- | --- |
| `passed` | Name the command, scope, and output. Do not generalize it beyond that scope. |
| `ignored` | Name the declared prerequisite and say that the test did not execute. |
| `unobserved` | Say that no reviewable receipt exists in the current worktree. |
| `blocked` | Name the concrete blocker and the smallest next evidence needed. |
| `inconclusive` | Explain why the receipt cannot establish the claimed snapshot or behavior. |

| `passed`：写出 command、scope 与 output，不得超出该 scope 推广。 |
| `ignored`：写出声明的 prerequisite，并说明 test 未执行。 |
| `unobserved`：说明当前 worktree 没有可审阅回执。 |
| `blocked`：说明具体 blocker 与所需的最小下一项证据。 |
| `inconclusive`：说明回执为何不能证明声称的 snapshot 或行为。 |

## Safe QA Rules / 安全 QA 规则

1. Prefer read-only file inventory, heading search, link-target checks, and status-vocabulary checks for documentation work.
2. Do not start Docker, a browser session, a remote workflow, an operator rehearsal, or a production target to make a document look complete.
3. Do not report a passed database, browser, remote, operator, release, or production state without a reviewable receipt for that exact state.
4. Keep `GraphDiff::between` as the sole graph-diff calculator and keep business logic out of documentation examples that could be copied into UI code.
5. Keep credentials, DSNs, tokens, claims, production hostnames, and raw audit data out of documentation and verification output.

1. 文档工作优先使用只读文件 inventory、heading search、link-target check 与 status-vocabulary check。
2. 不得为了让文档看起来完整而启动 Docker、browser session、remote workflow、operator rehearsal 或 production target。
3. 没有针对精确状态的可审阅回执时，不得报告 database、browser、remote、operator、release 或 production 已通过。
4. 保持 `GraphDiff::between` 为唯一 graph-diff calculator，并避免在可能被复制到 UI code 的文档示例中放置业务逻辑。
5. 文档与验证输出不得包含 credential、DSN、token、claim、production hostname 或 raw audit data。

## Review Checklist / 审阅清单

- [ ] Contract has a named owner, exact scope, version, ordering, and consumer.
- [ ] Contract 有明确 owner、精确 scope、版本、排序与 consumer。
- [ ] Public and private surfaces are separated.
- [ ] Public 与 private surface 已分离。
- [ ] Focused validation output is recorded without expanding its claim.
- [ ] 聚焦验证 output 已记录，且没有扩大其声明范围。
- [ ] Ignored, unobserved, blocked, and inconclusive states are explicit.
- [ ] Ignored、unobserved、blocked 与 inconclusive 状态已明确。
- [ ] Bilingual wording and links are present.
- [ ] 中英双语文案与链接均存在。
- [ ] No code, manifest, script, `.github`, Docker, browser, remote, operator, or production claim was added to the docs-only change.
- [ ] 文档专用变更没有新增 code、manifest、script、`.github`、Docker、browser、remote、operator 或 production claim。

The executable documentation-only record for this checklist is `docs/verification/wave-1-contract-checklist.md`.

本清单的可执行文档专用记录位于 `docs/verification/wave-1-contract-checklist.md`。
