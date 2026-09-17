# ADR 0001: Modular Monorepo / 模块化 Monorepo

## Status / 状态

Accepted.

已接受。

## Context / 背景

ContextLab must evolve as a long-lived open-source platform rather than a short demo. The project charter requires clean architecture, reusable Rust core logic, a design-system-first frontend, and future desktop, CLI, SDK, plugin, and API surfaces.

ContextLab 必须作为长期演进的开源平台建设，而不是短期演示项目。项目宪章要求采用 Clean Architecture、可复用 Rust Core、设计系统优先的前端，并为桌面端、CLI、SDK、插件和 API 留出长期空间。

## Decision / 决策

Use a modular monorepo with stable ownership boundaries:

采用模块化 monorepo，并保持清晰职责边界：

```text
apps/       # User-facing applications
server/     # API gateway and presentation layer
crates/     # Rust domain, application, and infrastructure crates
packages/   # Frontend design system, UI, shared code, and TypeScript SDK
docs/       # Architecture, ADRs, plans, and contributor guides
```

Business logic belongs in `crates/`. UI and API layers compose the core instead of owning domain behavior.

业务逻辑归属 `crates/`。UI 与 API 层只组合核心能力，不拥有领域行为。

## Consequences / 影响

- Core capabilities can be reused by web, desktop, CLI, and future SDK surfaces.
- 核心能力可以被 Web、桌面端、CLI 和未来 SDK 复用。
- Module boundaries can be tested independently.
- 模块边界可以独立测试。
- New contributors can find ownership by directory.
- 新贡献者可以通过目录快速理解职责归属。
- The repository can grow without turning UI pages or API handlers into business logic containers.
- 仓库扩大后，UI 页面与 API handler 不会变成业务逻辑堆放处。
