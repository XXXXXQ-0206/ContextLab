# Private ContextLab Browser Smoke Evidence / 私有 ContextLab 浏览器 Smoke 证据

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应条件与章程原则

This bounded evidence increment advances Criteria 1 and 8 by exercising the existing ContextLab
workspace as a rendered local Web application. The smoke must verify the server-rendered workspace,
the design-system controls, the mounted private read inspectors, and responsive layout at desktop and
mobile viewport sizes without inventing runtime data or changing product contracts.

本有界证据增量通过把现有 ContextLab workspace 作为实际本地 Web 应用渲染并检查，推进条件 1 与 8。smoke 必须验证 server-rendered workspace、
design-system controls、已挂载 private read inspector，以及 desktop/mobile viewport 下的响应式布局；不得虚构 runtime data，也不得改变产品 contract。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

Rust, API, SDK, BFF, and component-level Web tests are green, but no fresh browser receipt proves
that the integrated page renders without hydration/runtime errors, that the expected read-only
controls are reachable, or that the layout avoids horizontal overflow on desktop and mobile. This
gap can be tested against the preview-mode application without credentials, providers, database,
or mutation access.

Rust、API、SDK、BFF 与 component-level Web tests 均已通过，但尚无新鲜浏览器回执证明集成页面无 hydration/runtime error、预期 read-only
controls 可到达，或 desktop/mobile 布局没有 horizontal overflow。该缺口可在无需 credential、provider、database 或 mutation access 的 preview-mode 应用上测试。

### Why now / 为什么现在优先

The page already composes the required local read surfaces, and the latest increment explicitly
left browser/visual evidence unobserved. A black-box smoke is the smallest way to turn that
unobserved boundary into reproducible local evidence before adding another Context consumer.

页面已经组合所需的 local read surface，而最近增量明确保留 browser/visual evidence 为 unobserved。在增加其他 Context consumer 前，black-box
smoke 是把该未观测边界转成可复现本地证据的最小方式。

### Explicit non-goals / 明确非目标

- No production source, Rust, API, SDK, route, migration, provider, secret, database, write,
  polling, browser authentication, release, or production readiness change.
- No visual redesign, screenshot baseline, browser credential inspection, or user-data access.
- No second `GraphDiff` calculator; `GraphDiff::between` remains the sole graph-diff calculator.

- 不修改 production source、Rust、API、SDK、route、migration、provider、secret、database、write、polling、browser authentication、release 或 production readiness。
- 不进行 visual redesign、screenshot baseline、browser credential inspection 或用户数据访问。
- 不新增第二个 `GraphDiff` calculator；`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Smallest affected boundary and bilingual documentation / 最小边界与双语文档

The implementation boundary is a temporary Playwright smoke harness and the two bilingual evidence
receipts named below. It reads only rendered DOM, console/page errors, HTTP response statuses, and
viewport geometry. No application source file is changed by this increment.

实现边界是临时 Playwright smoke harness 与下述两份双语证据回执。它只读取 rendered DOM、console/page error、HTTP response status 与 viewport
geometry。本增量不修改 application source file。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

Run the smoke in preview mode at desktop and mobile viewports, capture the rendered control counts,
page/console errors, response failures, and horizontal-overflow result, then run `git diff --check`.
Authenticated browser-to-BFF-to-protected-Axum runtime, PostgreSQL/Docker, Git, remote CI,
operator rehearsal, release, and production remain separate unobserved or deferred boundaries.

在 preview mode 下以 desktop 与 mobile viewport 运行 smoke，记录 rendered control count、console/page error、HTTP response failure 与 horizontal-overflow 结果，
随后运行 `git diff --check`。authenticated browser-to-BFF-to-protected-Axum runtime、PostgreSQL/Docker、Git、remote CI、operator rehearsal、release 与 production
仍是独立的 unobserved 或 deferred 边界。

## Status / 状态

`completed / verified locally` for this bounded browser evidence increment; the long-term goal remains `active`. / 本有界浏览器证据增量为 `completed / verified locally`；长期目标保持 `active`。

## Completion Receipt / 完成回执

The reusable black-box harness in scripts/verify-contextlab-browser-smoke.py exercised the production Next server built from the current workspace at desktop (1440x1100) and mobile (390x844) viewports. Both renders had title ContextLab, all required workspace and private binding controls, no page errors, no unexpected console errors, no failed HTTP responses, and no horizontal overflow. The desktop path also filled the in-memory Bearer field and clicked the real Inspect bindings control; preview mode returned the expected 503 BFF-unavailable response with one local notice and no upstream diagnostic text.

可复用 black-box harness scripts/verify-contextlab-browser-smoke.py 使用当前 workspace 构建的 production Next server，在 desktop（1440x1100）与 mobile（390x844）viewport 实际运行。两种尺寸均得到 ContextLab 标题、完整 workspace 与 private binding controls，无 page error、无意外 console error、无失败 HTTP response，且没有 horizontal overflow。desktop 路径还填写了仅存于内存的 Bearer field 并点击真实的 Inspect bindings control；preview mode 返回预期的 503 BFF-unavailable response，只有一条本地 notice，不含 upstream diagnostic text。

The smoke captured screenshots at the temporary desktop and mobile paths printed by the harness; visual inspection confirmed the composed workspace renders in both sizes. The initial dev-server attempt exposed only a test-harness HMR WebSocket artifact; the final receipt uses the already-built production server on an independent port, and the server was stopped after the run.

本次 smoke 将截图保存到 harness 输出的临时 desktop 与 mobile 路径；视觉检查确认两种尺寸下 workspace 均完成渲染。初次 dev-server 尝试只暴露了测试 harness 的 HMR WebSocket artifact；最终回执使用已构建的 production server 与独立端口，运行后已停止 server。

No application source, Rust, API, SDK, route, migration, provider, secret, database, write, authentication, or GraphDiff behavior changed. Authenticated browser-to-BFF-to-protected-Axum runtime, PostgreSQL/Docker, Git, remote CI, operator rehearsal, release, production, and public-write readiness remain separate `unobserved` or `deferred` boundaries. This closes one local browser evidence increment only; Criteria 1 and 8 remain open and the long-term goal remains active.

未修改 application source、Rust、API、SDK、route、migration、provider、secret、database、write、authentication 或 GraphDiff behavior。authenticated browser-to-BFF-to-protected-Axum runtime、PostgreSQL/Docker、Git、remote CI、operator rehearsal、release、production 与 public-write readiness 继续是独立的 `unobserved` 或 `deferred` 边界。本回执只关闭一个本地浏览器证据增量；条件 1 与 8 仍开放，长期目标保持 active。
