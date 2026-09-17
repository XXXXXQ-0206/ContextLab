# Benchmark Definition and Decision Persistence Plan / Benchmark 定义与判定持久化计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Steps use checkbox (`- [ ]`) syntax.

**Goal / 目标：** Persist immutable benchmark definitions and deterministic decision evidence through private memory/PostgreSQL repository contracts without adding public writes or Web decision logic.

**Architecture / 架构：** `contextlab-evaluation` remains the sole policy calculator. `contextlab-storage` will store project-scoped immutable dataset/suite revisions, exact run bindings, thresholds, overall decisions, and per-metric evidence in one atomic state boundary. Read ports expose validated domain records; adapters never recalculate thresholds independently.

## Necessity Record / 必要性记录

**Criterion / 条件：** Completion criterion 3 requires benchmark suites, datasets, thresholds, and evaluation decisions to be persisted and queryable. The domain contract is now verified, but storage still has only arbitrary `suite_name` and metrics JSON reads.

完成条件 3 要求 benchmark suite、dataset、threshold 与 evaluation decision 可持久化、可查询。领域契约已经验证，但 storage 仍只有任意 `suite_name` 与 metrics JSON read。

**Gap and priority / 缺口与优先级：** Memory and PostgreSQL duplicate scorecard aggregation, evaluation runs are not bound to immutable suite/dataset policy, and no decision evidence is stored atomically. This is the smallest dependency-ready boundary before any honest read-only REST/SDK/Web inspection can exist.

Memory 与 PostgreSQL 当前重复 scorecard 聚合；evaluation run 未绑定不可变 suite/dataset policy，也没有原子存储 decision evidence。这是在诚实新增只读 REST/SDK/Web inspection 前最小且依赖就绪的边界。

**Non-goals / 非目标：** No benchmark execution, provider call, evaluator plugin, public or protected write route, OpenAPI/SDK/Web method, dashboard, A/B orchestration, evaluation diff, Docker, release, or production claim.

不包含 benchmark execution、provider call、evaluator plugin、public 或 protected write route、OpenAPI/SDK/Web method、dashboard、A/B orchestration、evaluation diff、Docker、release 或 production 声明。

**Minimal boundary / 最小边界：** Add private immutable records and repository ports, one shared in-memory atomic state, migration `0016` plus static migration registration tests, and PostgreSQL adapter tests that remain ignored when no database is configured. Reuse typed identities and decisions from `contextlab-evaluation`; retain the existing public read surface unchanged.

增加私有不可变 record 与 repository port、一个共享 in-memory atomic state、迁移 `0016` 及静态 migration registration test，并增加在未配置数据库时保持 ignored 的 PostgreSQL adapter test。复用 `contextlab-evaluation` 的 typed identity 与 decision，既有 public read surface 保持不变。

**Fresh verification / 新鲜验证：** Observe failing memory contract and migration-registration tests first; then run focused storage tests, formatting, clippy for touched crates, full Rust workspace, and boundary searches. PostgreSQL execution remains unobserved until a local non-Docker database is available; local unit/static evidence must not impersonate it.

先观察 memory contract 与 migration-registration 红测；随后运行聚焦 storage test、格式、受影响 crate clippy、完整 Rust workspace 与边界搜索。在本地非 Docker 数据库可用前，PostgreSQL execution 保持 unobserved；unit/static 本地证据不得冒充数据库证据。

## Tasks / 任务

- [x] Define immutable storage records and read/write ports that use `contextlab-evaluation` identities and decisions.
- [x] Add red/green in-memory atomic persistence, duplicate identity rejection, idempotent replay, exact-commit decision isolation, and changed-payload rejection tests.
- [x] Add migration `0016` for dataset/suite revisions, thresholds, run bindings, decisions, and per-metric evidence with project/context/commit integrity.
- [x] Register the migration and add static asset/invariant tests without requiring Docker.
- [x] Implement PostgreSQL adapters and ignored integration tests, including exact-commit isolation, seal guards, and out-of-range temperature rejection; runtime execution remains deferred until a local PostgreSQL service is explicitly available.
- [x] Verify no REST/OpenAPI/SDK/Web mutation or second decision calculator was introduced, then select the read-only inspection increment.

## Fresh Outcome / 新鲜结果

Focused red/green migration and storage contract tests, `cargo fmt --all -- --check`, scoped Clippy, and `cargo test --workspace --quiet` were observed after the final commit-scope, normalized-evaluation, and temperature-range fixes. The workspace run reports evaluation `27 passed` and storage `159 passed, 31 ignored`. PostgreSQL benchmark tests compile but were not executed while Docker is disabled and no local PostgreSQL service is configured; this is not PostgreSQL runtime evidence.

在最终修复 commit scope、规范化 evaluation 与 temperature range 后，已观察到聚焦的 red/green migration 与 storage contract test、`cargo fmt --all -- --check`、范围化 Clippy 以及 `cargo test --workspace --quiet`。工作区运行报告 evaluation `27 passed` 与 storage `159 passed, 31 ignored`。Docker 关闭且未配置本地 PostgreSQL service 时，PostgreSQL benchmark test 已编译但未执行；这不是 PostgreSQL runtime 证据。
