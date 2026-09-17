# Contributing to ContextLab / 为 ContextLab 贡献

ContextLab is context-first, evidence-gated, and bilingual. A contribution is welcome when it
closes a named open condition and can be verified locally; it is not welcome merely because it is
attractive, quick, or adjacent to existing code.

ContextLab 以 Context 为先、以证据为门、以中英双语为基本要求。只有当贡献收束了某个已命名的开放条件、并且可在本地验证时才被接受；
仅仅因为"看起来不错""实现很快"或"就在手边"的改动不在此列。

## Before you write code / 动手之前

1. Read `docs/roadmap/goal-governance.md`, `docs/roadmap/completion-criteria.md`, and
   `docs/roadmap/active-long-term-goal.md`.
2. Add a bilingual **Necessity Record / 必要性记录** to a plan under `docs/superpowers/plans/`
   stating the completion criterion served, the unmet dependency or evidence gap, why this is the
   next dependency-ready choice, explicit non-goals, the smallest affected boundary, and the fresh
   verification required.
3. Keep public and private surfaces separate. New public REST, OpenAPI, SDK, Web mutation, migration,
   provider, or transport work needs its own admission decision.

1. 先读 `docs/roadmap/goal-governance.md`、`docs/roadmap/completion-criteria.md` 与
   `docs/roadmap/active-long-term-goal.md`。
2. 在 `docs/superpowers/plans/` 下的计划中新增双语 **Necessity Record / 必要性记录**，写明：服务的完成条件、尚未满足的依赖或证据缺口、
   为什么这是当前依赖就绪的下一选择、明确非目标、最小受影响边界，以及必须先取得的新鲜验证。
3. 严格区分 public 与 private surface。新增 public REST、OpenAPI、SDK、Web mutation、migration、provider 或 transport
   都需要独立的准入决定。

An increment without that record is out of scope.

没有该记录的增量一律视为超出范围。

## Architecture rules / 架构规则

- Business logic lives in reusable Rust crates, never in UI components.
- `GraphDiff::between` is the sole graph-diff calculator. Do not add a second one.
- Persist through the existing storage contracts; do not bypass them from the API or the Web app.
- Keep everything that touches credentials redacted, request-scoped, and fail-closed.
- Never commit real provider keys, DSNs, tokens, JWTs, or production hostnames.

- 业务逻辑属于可复用的 Rust crate，不进入 UI component。
- `GraphDiff::between` 是唯一的 graph-diff calculator，不得新增第二个。
- 通过既有 storage contract 持久化，不得从 API 或 Web app 绕过。
- 一切涉及凭据的路径都必须脱敏、按请求作用域、失败关闭。
- 绝不提交真实 provider key、DSN、token、JWT 或 production hostname。

## Verify before you open a pull request / 提 PR 前必须验证

```bash
cargo fmt --all -- --check
cargo test --workspace
pnpm check:web
```

Also run the contract fixtures when you touch route catalogs, DTO shapes, or the Web BFF:

当改动涉及 route catalog、DTO 形状或 Web BFF 时，还要运行 contract fixture：

```bash
pwsh -NoProfile -File tests/contract/verify-local-contracts.test.ps1
pwsh -NoProfile -File scripts/verify-local-contracts.ps1
```

For storage or migration work, run the named ignored PostgreSQL cases through the disposable
verifier; it only accepts a loopback URL, an explicit disposable database name, a dedicated test
role, and an explicit reset opt-in.

存储或 migration 相关工作必须通过 disposable verifier 运行具名 ignored PostgreSQL case；该 verifier 只接受 loopback URL、
显式 disposable database name、专用 test role 与显式 reset 确认。

```bash
CONTEXTLAB_TEST_DATABASE_URL=postgres://contextlab_test_runner:postgres@localhost:5432/contextlab_test \
CONTEXTLAB_TEST_DATABASE_NAME=contextlab_test \
CONTEXTLAB_TEST_DATABASE_ROLE=contextlab_test_runner \
CONTEXTLAB_ALLOW_DISPOSABLE_DATABASE_RESET=1 \
  bash scripts/verify-disposable-postgres-storage.sh
```

## Report status honestly / 如实报告状态

Use the status vocabulary from `docs/contributing/wave-1-docs-qa.md`: `passed`, `ignored`,
`unobserved`, `blocked`, `inconclusive`. Name the exact command and scope for a `passed` claim, and
never generalize it beyond what actually ran. A local preview is not a live provider or
authenticated browser flow; a passing test suite is not release or production readiness.

使用 `docs/contributing/wave-1-docs-qa.md` 定义的状态词汇：`passed`、`ignored`、`unobserved`、`blocked`、`inconclusive`。
声称 `passed` 时必须写出确切 command 与 scope，不得超出实际运行范围推广。本地 preview 不等于 live provider 或
authenticated browser flow；测试套件通过不等于 release 或 production readiness。

## Documentation / 文档

Every user-visible change updates the bilingual documentation it affects: `README.md`,
`ARCHITECTURE.md`, `docs/api/`, `docs/architecture/`, and the roadmap receipts. Chinese text must be
natural modern Chinese, not a literal translation of the English.

每一处用户可见变更都要同步更新受影响的双语文档：`README.md`、`ARCHITECTURE.md`、`docs/api/`、`docs/architecture/` 与路线图回执。
中文必须是自然现代中文，而不是英文的逐句直译。

## License / 许可

By contributing you agree that your contribution is dual-licensed under the Apache License 2.0 or
the MIT license, at the user's option, as described in `LICENSE-APACHE` and `LICENSE-MIT`. This
matches the `license` field already declared in `Cargo.toml`.

The repository intentionally carries the two full license texts instead of one combined `LICENSE`
file, because that is the layout GitHub and the Rust ecosystem recognise as a dual-licensed project.

提交贡献即表示你同意该贡献按 `LICENSE-APACHE` 与 `LICENSE-MIT` 所述，以 Apache License 2.0 或 MIT license
双许可方式授权，使用者可任选其一。这与 `Cargo.toml` 中已声明的 `license` 字段一致。

仓库刻意保留两份完整许可文本，而不合并成单个 `LICENSE` 文件，因为这是 GitHub 与 Rust 生态识别"双许可项目"的布局。
