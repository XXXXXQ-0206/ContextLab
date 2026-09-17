# Design System Foundation / 设计系统地基

ContextLab uses a three-layer design token model so the interface can scale across web, desktop, docs, and future embedded surfaces without duplicating visual decisions.

ContextLab 采用三层 design token 模型，使 Web、桌面端、文档和未来嵌入式界面可以共享视觉决策，而不是重复造样式。

## Token Layers / Token 分层

1. Primitive tokens: raw values such as colors, spacing, radius, type scale, shadow, and motion.
2. Semantic tokens: purpose aliases such as background, foreground, panel, border, accent, warning, and danger.
3. Component tokens: component-specific contracts such as button background, panel border, and control radius.

1. Primitive token：颜色、间距、圆角、字号、阴影、动效等原始值。
2. Semantic token：background、foreground、panel、border、accent、warning、danger 等用途别名。
3. Component token：button background、panel border、control radius 等组件级契约。

## Packages / 包职责

- `packages/design-system` owns token source code and CSS variables.
- `packages/design-system` 负责 token 源码与 CSS 变量。
- `packages/ui` owns reusable primitives and components that consume tokens.
- `packages/ui` 负责消费 token 的可复用 primitive 与 component。
- `Button`, `StatusPill`, `Panel`, and `PanelHeader` provide shared primitives for dense operational views.
- `Button`、`StatusPill`、`Panel` 与 `PanelHeader` 为高信息密度工作台提供共享 primitive。
- `Input`, `Textarea`, and `Select` provide labeled native controls with token-driven focus, disabled, description, and responsive behavior for operational forms.
- `Input`、`Textarea` 与 `Select` 为 operational form 提供带标签的原生控件，以及 token 驱动的 focus、disabled、description 和响应式行为。
- `CodeChip`, `DefinitionGrid`, `StatGrid`, and `StackTable` provide shared dense display primitives for identifiers, key-value inspection, summary facts, and responsive operational tables.
- `CodeChip`、`DefinitionGrid`、`StatGrid` 与 `StackTable` 为 identifier、key-value inspection、summary fact 和响应式 operational table 提供共享的高密度展示 primitive。
- `apps/web` composes UI package exports into product surfaces.
- `apps/web` 只组合 UI package 输出，形成产品界面。

## Current Interface Direction / 当前界面方向

The first screen is a dense operational workspace for Context Engineering. It exposes context detail, route contracts, live workspace graph topology, graph relationship inspection, component inventory fingerprints, selected component metadata detail, version-backed Context Graph diff review, commit history, evaluation run discovery/detail, and selected scorecard averages in one surface.

第一屏是面向 Context Engineering 的高信息密度工作台，直接呈现 Context detail、route contract、live workspace graph topology、graph relationship inspection、component inventory fingerprint、selected component metadata detail、基于版本的 Context Graph 差异审阅、commit history、evaluation run discovery/detail 和 selected scorecard average。

The Web shell consumes live SDK/API data with deterministic preview and preview-fallback modes, plus shared UI primitives rather than page-local business logic. This keeps the interface ready for React Query and generated SDK adapters without rewriting the product surface.

Web shell 消费 live SDK/API data，并提供确定性的 preview 与 preview-fallback mode；同时复用共享 UI primitive，而不是把业务逻辑写进页面局部样式。这样后续接入 React Query 与 generated SDK adapter 时，可以保留现有产品界面。

The Web workspace now follows a `data -> presenter -> screen` boundary. `context-workspace-data.ts` owns SDK and preview fallback reads, `context-workspace-presenter.ts` turns API-shaped records into deterministic semantic view models, and `context-workspace-screen.tsx` adapts those view models into reusable UI primitives.

Web workspace 现在遵循 `data -> presenter -> screen` 边界：`context-workspace-data.ts` 负责 SDK 与 preview fallback 读取，`context-workspace-presenter.ts` 将 API-shaped record 转换为确定性的语义 view model，`context-workspace-screen.tsx` 再把这些 view model 适配为可复用 UI primitive。

The Context Graph panel consumes the workspace Context Graph SDK/API payload through the same boundary. The presenter is responsible for layout coordinates, compact graph facts, and relationship view models; the screen renders those relationships with `StackTable` and `StatusPill` without reconstructing them from page-local business logic.

Context Graph panel 通过同一边界消费 workspace Context Graph SDK/API payload。Presenter 负责 layout coordinate、紧凑 graph fact 与 relationship view model；screen 使用 `StackTable` 与 `StatusPill` 渲染这些 relationship，不再通过页面局部业务逻辑重建 relationship。

The Context Graph inspector is a read-only client interaction adapter. It keeps transient node selection local, calls the presenter selection helper for direct incoming and outgoing relationship view models, and renders semantic node buttons with `aria-pressed`, visible keyboard focus, a labeled selector for graph nodes beyond the bounded visual layout, and responsive detail tables. It does not own graph traversal, API calls, or mutation rules.

Context Graph 检查器是只读的客户端交互适配器。它把短暂的节点选择状态留在本地，通过 presenter 的选择辅助函数获取直接入边和出边 view model，并用语义化节点按钮、`aria-pressed`、可见的键盘焦点、面向超出受限视觉布局节点的带标签选择器和响应式详情表格渲染。它不负责图遍历、API 调用或修改规则。

## Dense Data Display Primitives / 高密度数据展示 Primitive

`CodeChip`, `DefinitionGrid`, `StatGrid`, and `StackTable` package the repeated inspection patterns used by ContextLab's operational workspaces. They keep hashes, metadata, metrics, and table-like discovery lists visually consistent while preserving server-safe composition inside `apps/web`.

`CodeChip`、`DefinitionGrid`、`StatGrid` 与 `StackTable` 将 ContextLab 工作台里反复出现的检查模式收敛为共享 primitive。它们让 hash、metadata、metrics 与类表格发现列表保持统一视觉节奏，同时继续把 server-safe 的组合职责留在 `apps/web`。

- `CodeChip` renders mono identifiers, fingerprints, model names, and short hashes.
- `CodeChip` 用于渲染等宽 identifier、fingerprint、model name 与短 hash。
- `DefinitionGrid` handles stable key-value inspection for metadata, routes, and persisted metrics.
- `DefinitionGrid` 负责 metadata、route 与持久化 metric 的稳定 key-value 检查布局。
- `StatGrid` summarizes compact operational facts without introducing dashboard-specific business logic.
- `StatGrid` 用于汇总紧凑的 operation fact，而不会引入 dashboard 专属业务逻辑。
- `StackTable` provides token-driven responsive row layouts for dense lists that must collapse cleanly on mobile.
- `StackTable` 为需要在移动端自然折叠的高密度列表提供 token 驱动的响应式 row 布局。

Graph relationship inspection rows use the same `StackTable` rhythm as operational lists: source entity, relationship kind, and target entity stay visible as explicit graph contract data rather than decorative graph labels.

Graph relationship inspection row 复用 operational list 的 `StackTable` 节奏：source entity、relationship kind 与 target entity 作为显式 graph contract data 保持可见，而不是只作为装饰性图谱标签。

Component inventory rows reuse the same operational table rhythm as evaluation runs: token-driven borders, compact spacing, mono content hashes, and responsive single-column behavior on mobile.

Component inventory row 复用 evaluation run 的操作表格节奏：使用 token 驱动的边框、紧凑间距、等宽 content hash，并在移动端切换为单列布局。

The selected component detail block is a metadata-first inspection pattern. It uses shared status pills, mono fingerprint chips, stable key-value grids, and explicit bilingual copy that body content is not part of the current component detail contract.

Selected component detail block 是 metadata-first 的检查模式。它使用共享 status pill、等宽 fingerprint chip、稳定的 key-value grid，并通过中英双语文案明确说明当前 component detail contract 不包含正文内容。

The local Context lifecycle editor composes `Input`, `Textarea`, `Select`, `Button`, `StatusPill`, `CodeChip`, and `DefinitionGrid` without placing lifecycle rules in the screen. Its data and presenter modules own request shaping, structured errors, and state transitions; the client component owns only in-memory form state and interaction. It is deliberately marked local-only and does not turn the general Web workspace into a public mutation surface.

本地 Context 生命周期编辑器组合 `Input`、`Textarea`、`Select`、`Button`、`StatusPill`、`CodeChip` 与 `DefinitionGrid`，不会把 lifecycle rule 放进 screen。其 data 与 presenter module 负责请求构造、结构化错误与状态转换；client component 只管理内存表单状态与交互。它被明确标记为仅本地，不会把通用 Web workspace 变为 public mutation surface。

The selected evaluation run detail block follows the same dense inspection pattern for raw persisted metrics. It keeps benchmark identity, model configuration, timestamps, and metric key/value rows visible without deriving scorecards, pass/fail states, or regression conclusions inside the UI layer.

Selected evaluation run detail block 沿用同一套高信息密度检查模式，用于查看原始持久化 metrics。它展示 benchmark identity、model configuration、timestamp 与 metric key/value row，但不会在 UI 层推导 scorecard、pass/fail state 或 regression conclusion。

The selected scorecard block uses the same operational table rhythm for aggregated metric averages. It is scoped to the selected evaluation run's suite and model filters so the UI inspects one benchmark/model slice without deriving policy thresholds.

Selected scorecard block 复用同一套 operational table 节奏展示聚合 metric average。它按选中 evaluation run 的 suite 与 model filter 收敛，使 UI 只检查一个 benchmark/model 切片，而不在界面层推导 policy threshold。
