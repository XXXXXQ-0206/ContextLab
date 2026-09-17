# Private CLI Read-only Smoke Receipt / 私有 CLI 只读 Smoke 回执

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
serves Criterion 8, CLI/Desktop and developer experience, and the charter's shared-Rust-core
principle. The repository already contains a provider-free CLI with a replay projection inspector,
capability availability inspector, and typed unavailable adapter path; the named gap is a fresh
executable smoke receipt for their documented exit codes and redacted output.

本增量直接服务条件 8“CLI/Desktop 与开发者体验”及章程中的 shared Rust core 原则。仓库已有 provider-free CLI、replay projection inspector、
capability availability inspector 与 typed unavailable adapter path；当前命名缺口是对文档所列 exit code 与脱敏输出取得新鲜可执行 smoke 回执。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The existing unit and
adapter tests prove the CLI contract, but the contributor smoke document still lacks a current
binary-build-and-process receipt. Without it, Criterion 8 has no direct local process evidence for
the read-only staging surface.

现有 unit 与 adapter test 已证明 CLI contract，但 contributor smoke 文档缺少当前 binary build-and-process 回执。没有它，条件 8 的只读 staging
surface 缺少直接的本地进程证据。

**Why now / 为什么现在优先:** This is the smallest dependency-ready increment after the
benchmark evidence wave. It uses already implemented behavior, closes a named local evidence gap,
and does not compete with the still-open public/release gates or introduce new product scope.

这是 benchmark evidence wave 之后最小且依赖就绪的增量。它只验证已有行为、收束一个命名的本地证据缺口，不与仍开放的 public/release gate 竞争，也不引入新产品范围。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Run the
existing `apps/cli` binary and update only `docs/verification/cli-read-only-smoke.md` plus this
plan and the roadmap receipts. No production source or test file is admitted unless the executable
smoke exposes a real defect; any such repair requires a separate root-cause note.

仅运行既有 `apps/cli` binary，并只更新 `docs/verification/cli-read-only-smoke.md`、本计划与路线图回执。除非 executable smoke 暴露真实 defect，
否则不准入 production source 或 test file；如需修复，必须另记 root cause。

**Explicit non-goals / 明确非目标:** No CLI write, Context mutation, provider call, network or
credential access, Desktop/Tauri runtime claim, public REST/OpenAPI/SDK method, Web mutation,
Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal, release,
production claim, or second `GraphDiff` calculator. `GraphDiff::between` remains the sole graph-diff
calculator.

不新增 CLI write、Context mutation、provider call、network 或 credential access、Desktop/Tauri runtime 声明、public REST/OpenAPI/SDK method、Web mutation、
Docker/PostgreSQL runtime、authenticated browser、Git、remote CI、operator rehearsal、release、production 声明或第二个 `GraphDiff` calculator。
`GraphDiff::between` 继续是唯一 graph-diff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Build
the CLI offline and run the documented valid replay, valid unavailable capability, invalid capability,
and adapter-backed unavailable commands. Capture stdout/stderr and exact exit codes. Then run the
focused CLI tests, format, strict offline Clippy for the CLI/Desktop packages, locked Rust check,
and the existing local verifier/static GraphDiff check. Keep Desktop runtime, PostgreSQL/Docker,
browser, Git, remote CI, operator, release, and production facts `unobserved` or `deferred`.

离线构建 CLI，并运行文档中的 valid replay、valid unavailable capability、invalid capability 与 adapter-backed unavailable command。记录 stdout/stderr 与精确
exit code。随后运行 CLI focused tests、fmt、CLI/Desktop scoped strict Clippy、锁定 Rust check 与既有 local verifier/static GraphDiff check。Desktop runtime、
PostgreSQL/Docker、browser、Git、remote CI、operator、release 与 production 事实保持 `unobserved` 或 `deferred`。

## Implementation Checklist / 实施清单

- [x] Build the CLI and run all four documented read-only smoke commands.
- [x] Record exact output boundaries and exit codes in the bilingual smoke guide.
- [x] Run focused and scope-matched verification without closing Criterion 8 or the long-term goal.

- [x] 构建 CLI 并运行四条文档化只读 smoke command。
- [x] 在双语 smoke guide 中记录精确 output boundary 与 exit code。
- [x] 运行 focused 与范围匹配验证，不关闭条件 8 或长期目标。

## Evidence Boundary / 证据边界

This is local process evidence for a provider-free, read-only staging binary. It does not prove
Desktop runtime, production readiness, external release gates, or any public write surface. The
long-term goal remains active.

本增量是 provider-free、只读 staging binary 的本地进程证据。不证明 Desktop runtime、production readiness、external release gate 或任何 public write surface。
长期目标保持 active。

## Fresh Receipt / 新鲜回执（2026-08-01）

`cargo build --offline -p contextlab-cli` passed. The built process returned the documented exit
codes: valid replay `0` with deterministic redacted identity/count output; valid unavailable
capability `2` with bilingual presentation; capability operation/integration drift `64` with empty
stdout and only the bilingual invalid-contract stderr; and adapter-backed workspace `2` with the
explicit unavailable registration message. Focused tests passed for the adapter contract `8`, CLI
command path `5`, and Desktop staging shell `5`; scoped strict Clippy passed. Locked workspace
check, PostgreSQL/Docker, authenticated browser, Git, remote CI, operator, release, and production
remain `unobserved` or `deferred`.

`cargo build --offline -p contextlab-cli` 通过。构建出的进程返回文档化 exit code：valid replay 为 `0`，输出确定性且脱敏的 identity/count；valid unavailable
capability 为 `2`，输出双语 presentation；capability operation/integration drift 为 `64`，stdout 为空且 stderr 只有双语 invalid-contract；adapter-backed
workspace 为 `2`，输出明确的 unavailable registration message。adapter contract focused test `8`、CLI command path `5` 与 Desktop staging shell `5` 均通过；
scoped strict Clippy 通过。锁定 workspace check、PostgreSQL/Docker、authenticated browser、Git、remote CI、operator、release 与 production 继续为 `unobserved` 或 `deferred`。
