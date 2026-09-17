# Private Branch-Head Discovery / 私有分支 Head 发现

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** This increment
directly advances Criteria 1, 2, 4, 5, 6, and 9. A Context-first version history needs a typed,
read-only way to discover the durable heads of a Context's branches before a future branch, merge,
or replay workflow can select exact commit snapshots. The contract must remain a reusable Rust
storage boundary with a private local adapter and a shared design-system presentation path.

**完成条件与章程原则：** 本增量直接推进条件 1、2、4、5、6 与 9。Context-first 的版本历史在未来分支、合并或回放
workflow 选择精确 commit snapshot 前，需要一种 typed、只读的方式发现一个 Context 的持久化 branch head。该 contract
必须保持为可复用 Rust storage boundary，并通过 private local adapter 与共享 design-system presentation path 消费。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The Rust
`ContextBranchRepository` port and Memory/PostgreSQL implementations already exist, but there is
no stable private read projection consuming it. Callers would otherwise infer branch heads from
commit lists or page-local state, risking stale or cross-Context selection. The missing evidence is
an exact scoped response, deterministic ordering, fail-closed parsing, and an accessible local
inspection state; PostgreSQL runtime and authenticated browser receipts remain unavailable.

**未满足依赖、风险或证据缺口：** Rust `ContextBranchRepository` port 及 Memory/PostgreSQL implementation 已存在，但
还没有消费它的稳定 private read projection。否则 caller 只能从 commit list 或页面本地状态推断 branch head，容易产生
过期或跨 Context 选择。当前缺口是 exact scope response、确定性排序、fail-closed parsing 与可访问的 local inspection state；
PostgreSQL runtime 与 authenticated browser receipt 仍不可用。

**Why now / 为何现在优先:** The preceding commit-associated graph snapshot and version-bound
graph review contracts are verified locally. Branch-head discovery is the smallest dependency-ready
read predecessor for branch/merge/replay and is closer to Criteria 1, 2, and 4 than starting a new
provider, workflow execution, or public transport feature.

**为何现在优先：** 前置的 commit-associated graph snapshot 与 version-bound graph review contract 已完成本地验证。Branch-head
discovery 是 branch/merge/replay 最小且依赖已满足的 read predecessor，相比启动 provider、workflow execution 或 public
transport feature，更直接服务条件 1、2 与 4。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** The
Rust branch-head read model and existing repository adapters remain the source of truth. Add only
the private API response/handler wiring, non-public local SDK parser/client, and a Web
`data -> presenter -> screen` inspection component using existing shared primitives. Update this
plan and the active roadmap with observed receipts only.

**最小受影响边界与双语文档：** Rust branch-head read model 与既有 repository adapter 继续作为 source of truth。只新增
private API response/handler wiring、非公开 local SDK parser/client，以及使用既有 shared primitive 的 Web
`data -> presenter -> screen` inspection component。仅在观测到真实 receipt 后更新本计划与 active roadmap。

**Explicit non-goals / 明确非目标:** No branch creation, merge, rollback, commit mutation,
public REST/OpenAPI/public SDK method, operator transport, database migration, provider call,
credential persistence, raw private content, Docker/PostgreSQL runtime, authenticated browser
claim, release/production claim, or second `GraphDiff` calculator.

**明确非目标：** 不新增 branch creation、merge、rollback、commit mutation、public REST/OpenAPI/public SDK method、operator
transport、database migration、provider call、credential persistence、raw private content、Docker/PostgreSQL runtime、
authenticated browser claim、release/production claim 或第二个 `GraphDiff` calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Run focused
Rust branch-head tests, private API tests, local SDK tests, Web inspector tests, then
`cargo fmt --all -- --check`, `cargo test --workspace --quiet --no-fail-fast`, strict offline
workspace Clippy, locked Rust 1.85 check, `pnpm check:web`, and the one-calculator static check.
Unobserved PostgreSQL runtime, browser, Git, remote CI, operator, release, and production evidence
must remain labeled as such.

**下一增量前的新鲜验证：** 先运行 focused Rust branch-head、private API、local SDK 与 Web inspector tests，再运行
`cargo fmt --all -- --check`、`cargo test --workspace --quiet --no-fail-fast`、strict offline workspace Clippy、锁定 Rust 1.85
check、`pnpm check:web` 与 single-calculator static check。未观测到的 PostgreSQL runtime、browser、Git、remote CI、operator、
release 与 production evidence 必须继续如实标记。

## Shared Contract / 共享契约

```text
GET /api/v1/local/contexts/{context_id}/branches

{
  "schema_version": "contextlab.local-context-branch-heads.v1",
  "context_id": "<uuid>",
  "branches": [
    {
      "branch_name": "main",
      "head_commit_id": "<uuid>|null",
      "revision": 0
    }
  ]
}
```

The server owns Context scope, authorization, audit, rate limiting, and ordering. The local SDK
rejects unknown fields, invalid UUIDs, duplicate branch names, negative/non-integer revisions,
wrong scope, and unsorted responses. Web shows loading, unavailable/error, empty, and ready states
with bilingual labels and `aria-live`/`aria-busy` semantics. The route remains default-deny unless
the existing server-owned local capability gate is enabled.

服务端拥有 Context scope、授权、审计、限流与排序。Local SDK 拒绝 unknown fields、无效 UUID、重复 branch name、负数或非整数
revision、错误 scope 与未排序 response。Web 通过双语 label 与 `aria-live`/`aria-busy` 展示 loading、unavailable/error、empty
与 ready state。该 route 仍默认关闭，只有既有 server-owned local capability gate 开启时可用。

## Implementation Checklist / 实施清单

- [x] Add focused Rust repository/adapter contract coverage without changing the existing port.
- [x] Add the default-off protected local API route and focused auth, scope, rate-limit, audit, and
      public-contract exclusion tests.
- [x] Add the fail-closed non-public local SDK resource and client tests.
- [x] Add Web data, presenter, and screen inspection using shared design-system primitives.
- [x] Run fresh verification, update the roadmap receipt, and keep the long-term goal active.

- [x] 在不改变既有 port 的前提下增加 focused Rust repository/adapter contract coverage。
- [x] 增加 default-off protected local API route，并覆盖 auth、scope、rate-limit、audit 与 public-contract exclusion test。
- [x] 增加 fail-closed 的非公开 local SDK resource 与 client test。
- [x] 使用 shared design-system primitive 增加 Web data、presenter 与 screen inspection。
- [x] 运行新鲜验证、更新 roadmap receipt，并保持长期目标 active。

## Fresh Local Receipt / 新鲜本地回执 (2026-07-29)

Status is `completed` / `verified locally`. The typed branch-head read now spans the reusable
Rust storage port and Memory/PostgreSQL adapters, the default-off protected API route, the
non-public local SDK, the same-origin Web BFF, and the shared `data -> presenter -> screen`
inspector. The API owns exact Context scope, authorization, audit, rate limiting, ordering,
private/no-store headers, and nullable unborn heads. The Web boundary reuses the local SDK's
fail-closed V1 parser for canonical UUIDs, legal branch names, unique server ordering, and exact
response shape; preview-only display targets remain non-empty strings until transport begins.

状态为 `completed` / `verified locally`。类型化 branch-head read 已贯通可复用 Rust storage port 与
Memory/PostgreSQL adapter、default-off protected API route、非公开 local SDK、同源 Web BFF 与共享
`data -> presenter -> screen` inspector。API 负责 exact Context scope、authorization、audit、rate limiting、
ordering、private/no-store header 与 nullable unborn head。Web boundary 复用 local SDK 的 fail-closed V1 parser，
校验 canonical UUID、合法 branch name、唯一且由服务端提供的排序与 exact response shape；仅 preview display target
在 transport 开始前保留为非空字符串。

Fresh evidence / 新鲜证据: focused storage contract `7 passed`; API handler `6 passed` and protected
router `2 passed`; local SDK focused `7 passed` and full package `99 passed`; BFF `3 passed`; Web
branch-head data/presenter/screen `5 passed`; `cargo fmt --all -- --check`; workspace Rust
`cargo test --workspace --quiet --no-fail-fast` passed; strict offline workspace Clippy; locked
offline workspace check; and `pnpm check:web` with public SDK `15`, local SDK `99`, Web `197`,
TypeScript/lint, and production build. Static inspection still finds exactly one `impl GraphDiff`.

新鲜证据：focused storage contract `7 passed`；API handler `6 passed`、protected router `2 passed`；local SDK focused
`7 passed`、全包 `99 passed`；BFF `3 passed`；Web branch-head data/presenter/screen `5 passed`；`cargo fmt --all -- --check`；
workspace Rust `cargo test --workspace --quiet --no-fail-fast` 通过；strict offline workspace Clippy；locked offline workspace
check；以及 `pnpm check:web`（public SDK `15`、local SDK `99`、Web `197`、TypeScript/lint 与 production build）。静态检查
仍观测到唯一一个 `impl GraphDiff`。

The review found and fixed Web parser drift before this receipt: the Web parser no longer sorts or
accepts non-UUID/raw branch names. No public REST/OpenAPI/public SDK method, branch mutation,
merge/rollback, migration, operator transport, provider call, secret access, or second diff
calculator was added. PostgreSQL runtime, Docker, authenticated browser, Git change-set, remote
CI, operator rehearsal, release, and production evidence remain `unobserved` or `deferred`; the
long-term goal remains active.

本次复核先发现并修复了 Web parser drift：Web parser 不再替 response 排序，也不再接受非 UUID 或 raw branch name。没有新增
public REST/OpenAPI/public SDK method、branch mutation、merge/rollback、migration、operator transport、provider call、
secret access 或第二个 diff calculator。PostgreSQL runtime、Docker、authenticated browser、Git change-set、remote CI、
operator rehearsal、release 与 production evidence 仍为 `unobserved` 或 `deferred`；长期目标保持 active。
