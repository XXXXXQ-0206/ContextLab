# Context Graph Inspector Design

**Status:** Implemented and verified / 已实现并验证

## Goal / 目标

Add an interactive, read-only Context Graph inspector to the Web workspace. A user can select a graph node and inspect its identity, kind, direct incoming relationships, and direct outgoing relationships without changing the graph contract or adding page-local graph reconstruction.

为 Web workspace 增加交互式、只读的 Context Graph 检查器。用户可以选择图节点，查看其标识、类型、直接入边和直接出边；实现不得改变图契约，也不得在页面局部重建图数据。

## Product Boundary / 产品边界

Included:

- Select any rendered graph node with pointer or keyboard interaction, and select every graph-contract node through an accessible selector when the visual preview is layout-limited.
- Show one stable detail region for the selected node, including bilingual empty states.
- Derive direct relationships from the existing presenter view model.
- Preserve the existing relationship table as a complete graph-wide inspection view.
- Keep preview, preview-fallback, and live SDK data paths behaviorally identical at the screen boundary.

包含范围：

- 支持通过指针或键盘选择任意已渲染的图节点；当视觉预览受布局限制时，可通过无障碍选择器选择图契约中的全部节点。
- 为选中节点显示一个稳定的详情区域，并提供中英双语空状态。
- 只从现有 presenter view model 推导直接关系。
- 保留现有 relationship table，继续作为全图关系检查视图。
- 让 preview、preview-fallback 与 live SDK 数据在 screen 边界保持相同行为。

Excluded:

- Graph mutation, relationship creation or deletion, persistence changes, and new API or SDK operations.
- Graph diffing, semantic diffing, graph layout algorithms, and canvas or React Flow migration.
- Scorecard thresholds, regression decisions, dashboard policy, or evaluation write flows.

不包含范围：

- 图修改、关系增删、持久化变更，以及新的 API 或 SDK 操作。
- 图 Diff、语义 Diff、图布局算法，以及迁移到 canvas 或 React Flow。
- Scorecard 阈值、回归结论、dashboard 策略或评测写入流程。

## Architecture / 架构

The existing `data -> presenter -> screen` boundary remains intact. `context-workspace-data.ts` continues to obtain the workspace graph through the current SDK and preview fallback. `context-workspace-presenter.ts` becomes the single place that converts graph node and relationship records into an inspector-ready, deterministic view model. The screen owns only transient selection state and composes the provided view model with shared UI primitives.

现有的 `data -> presenter -> screen` 边界保持不变。`context-workspace-data.ts` 继续通过当前 SDK 与 preview fallback 获取 workspace graph。`context-workspace-presenter.ts` 仍是唯一负责把图节点和关系记录转换为确定性检查器 view model 的位置。Screen 只拥有短暂的选择状态，并使用共享 UI primitive 组合 presenter 提供的 view model。

The presenter will expose a node identifier, kind, label, compact detail, and relationship identifiers for every graph node. It will provide a bounded visual-node list for layout, a complete inspector-node list for accessible selection, relationship rows once, and pure selection helpers that return the selected node and its incoming and outgoing rows. This keeps relationship matching testable without coupling the Web interaction to the raw SDK DTO shape.

Presenter 将为每个图节点提供 node identifier、类型、标签、紧凑说明和关系标识；它为布局提供受限的视觉节点列表，为无障碍选择提供完整的检查节点列表，关系行只生成一次，再通过纯选择辅助函数返回选中节点及其入边、出边。这样既能独立测试关系匹配，又不会让 Web 交互依赖原始 SDK DTO 结构。

The graph panel will be isolated as a client-side interaction adapter beneath the screen composition boundary. It will render semantic buttons for nodes, maintain the selected node identifier, expose selection through `aria-pressed`, and render the inspector through `Panel`, `PanelHeader`, `DefinitionGrid`, `StackTable`, and `StatusPill`. No business rules, API calls, or raw graph traversal may move into the component.

图面板将作为 screen 组合边界下的客户端交互适配器独立出来。它用语义化 button 渲染节点，维护选中 node identifier，通过 `aria-pressed` 暴露选择状态，并用 `Panel`、`PanelHeader`、`DefinitionGrid`、`StackTable` 与 `StatusPill` 渲染检查器。组件中不得加入业务规则、API 调用或原始图遍历。

## Interaction and States / 交互与状态

- Initial selection is the first presenter node, so the inspector is informative on first render.
- Selecting another node updates its detail, direct incoming relationships, and direct outgoing relationships without affecting the complete relationship table.
- A node with no direct incoming or outgoing relationships shows a compact `No incoming relationships / 暂无入边` or `No outgoing relationships / 暂无出边` state.
- A graph with no nodes keeps the existing graph empty state and does not render a misleading selected-node inspector.
- The visual treatment uses existing tokens: selected nodes have a clear non-color-only state, node buttons retain a stable size, and the inspector remains readable in narrow layouts.

- 初始选择 presenter 中的第一个节点，让检查器首次渲染就提供信息。
- 选择其他节点后，只更新该节点的详情、直接入边和直接出边；全图 relationship table 不受影响。
- 没有直接入边或出边的节点分别显示紧凑的 `No incoming relationships / 暂无入边` 或 `No outgoing relationships / 暂无出边` 状态。
- 图中没有节点时，保留现有空状态，不渲染误导性的已选节点检查器。
- 视觉层继续使用既有 token：选中节点必须提供不只依赖颜色的状态，节点按钮保持稳定尺寸，检查器在窄屏下仍可阅读。

## Testing and Verification / 测试与验证

- Extend presenter tests with selected-node identity and direct relationship assertions for preview and sparse live graph data.
- Add focused component tests for initial selection, keyboard-accessible selection, incoming/outgoing relationship rendering, and empty relationship states.
- Extend the workspace visual verifier to assert the inspector's bilingual labels and selected preview node content on desktop and mobile.
- Run `pnpm --filter @contextlab/web test`, `pnpm --filter @contextlab/web lint`, `pnpm --filter @contextlab/web build`, and `pnpm check:web` after implementation.

- 扩展 presenter 测试，覆盖 preview 与稀疏 live graph 的选中节点标识和直接关系断言。
- 增加聚焦的组件测试，覆盖初始选择、可通过键盘访问的选择、入边/出边渲染和空关系状态。
- 扩展 workspace visual verifier，检查桌面端和移动端的检查器中英双语标签与选中 preview 节点内容。
- 实现后运行 `pnpm --filter @contextlab/web test`、`pnpm --filter @contextlab/web lint`、`pnpm --filter @contextlab/web build` 和 `pnpm check:web`。

## Completion Criteria / 本周期完成判定

This cycle is complete only when the interactive inspector works against the existing graph contract in all three data modes, its state is accessible and responsive, the specified verification passes, and the route map and long-term completion audit remain honest about the remaining graph diffing and editing work.

本周期只有在交互式检查器能基于既有图契约运行于全部三种数据模式、状态具备可访问性且适配窄屏、指定验证全部通过，并且路线图与长期完成度审计仍如实说明图 Diff 和图编辑尚未完成时，才算完成。
