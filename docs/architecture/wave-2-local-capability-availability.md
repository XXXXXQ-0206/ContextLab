# Wave 2 Local Capability Availability Contracts / Wave 2 本地能力可用性契约

> **Status / 状态:** Active local contract documentation. It records only the checked-in staging adapters and the focused local receipts named in [the Wave 2 evidence matrix](../verification/wave-2-local-capability-evidence-matrix.md). It does not register a shared integration, expose a public API, or establish any external runtime or release claim.
>
> **状态：** 活动的本地契约文档。本文只记录已检入的 staging adapter 与 [Wave 2 证据矩阵](../verification/wave-2-local-capability-evidence-matrix.md) 中列出的 focused 本地回执。它不注册共享集成、不公开 API，也不建立任何外部运行时或发布声明。

## Purpose / 目的

Wave 2 gives CLI, Desktop, and Web consumers an explicit, versioned answer when a shared core integration has not yet been registered. The answer is `unavailable`, not a substitute implementation and not evidence that the requested operation executed.

Wave 2 为 CLI、Desktop 与 Web consumer 在共享核心集成尚未注册时提供显式、带版本的答复。该答复为 `unavailable`，不是替代实现，也不表示请求的操作已经执行。

The source boundaries are the shared Rust adapter contract at `apps/cli/crates/contextlab-adapter-contract`, the CLI staging shell at `apps/cli`, the Tauri staging shell at `apps/desktop/src-tauri`, and the Web data/presentation adapters under `apps/web/src/app`. Root manifests, public REST/OpenAPI, public SDK transport, dynamic registration, and production wiring remain outside this document.

源边界包括 `apps/cli/crates/contextlab-adapter-contract` 中共享的 Rust adapter contract、`apps/cli` 中的 CLI staging shell、`apps/desktop/src-tauri` 中的 Tauri staging shell，以及 `apps/web/src/app` 下的 Web data/presentation adapter。根 manifest、public REST/OpenAPI、public SDK transport、动态注册与生产接线均不属于本文范围。

## Shared Adapter Transport / 共享 Adapter 传输

`LocalCapabilityAvailabilityV1` is the serializable DTO projected by the Rust staging adapter. Its literal schema version is `contextlab.local-capability-availability.v1`; consumers must reject a different version, operation identifier, integration mapping, availability state, or reason.

`LocalCapabilityAvailabilityV1` 是 Rust staging adapter 投影出的可序列化 DTO。其字面 schema version 为 `contextlab.local-capability-availability.v1`；consumer 必须拒绝不同的 version、operation identifier、integration mapping、availability state 或 reason。

| Field / 字段 | Current contract / 当前契约 | Meaning / 含义 |
| --- | --- | --- |
| `schema_version` | `contextlab.local-capability-availability.v1` | Versioned local wire shape / 带版本的本地传输形状。 |
| `operation_id` | Stable adapter operation identifier / 稳定的 adapter 操作标识 | Identifies the command class, never user-supplied request contents / 标识命令类别，绝不包含用户提供的请求内容。 |
| `integration` | Required shared integration identifier / 所需共享集成标识 | Names the integration that the Integration Lead must register / 指明必须由 Integration Lead 注册的集成。 |
| `availability` | `unavailable` | The staging adapter has no registered implementation / staging adapter 没有已注册的实现。 |
| `reason` | `shared_integration_not_registered` | Stable machine-readable reason / 稳定、机器可读的原因。 |

The current local adapter has one state only: `unavailable`. It must not be rendered as `available`, silently retried through an unrelated adapter, or translated into an assertion about core behavior.

当前本地 adapter 只有一种状态：`unavailable`。不得将其渲染为 `available`、通过无关 adapter 静默重试，或转换为关于核心行为的断言。

## Operation Mapping / 操作映射

| `operation_id` | Required integration / 所需集成 | CLI command path / CLI 命令路径 | Current local result / 当前本地结果 |
| --- | --- | --- | --- |
| `workspace-inspect` | `contextlab-context-core` | `workspace inspect <id>` | `unavailable` |
| `context-inspect` | `contextlab-context-core` | `context inspect <id>` | `unavailable` |
| `evaluation-run` | `contextlab-evaluation` | `evaluation run <context-id> <suite-id>` | `unavailable` |
| `diff-compare` | `contextlab-diff-engine` | `diff compare <base-revision> <comparison-revision>` | `unavailable` |
| `workflow-inspect` | `contextlab-workflow` | `workflow inspect <id>` | `unavailable` |

CLI and Desktop delegate to the same unavailable adapter contract. They preserve the operation-to-integration mapping and return a typed status with a versioned availability DTO; neither shell owns a second domain implementation.

CLI 与 Desktop 都委托给同一 unavailable adapter contract。它们保留 operation 到 integration 的映射，并返回带版本可用性 DTO 的类型化状态；两个 shell 都不拥有第二份领域实现。

## Web Presentation Resource / Web 展示资源

The Web layer also defines `LocalCapabilityAvailabilityV1`, a presentation resource with numeric `schema_version: 1`, a stable `capability_id`, bilingual capability text, `available` or `unavailable`, and optional bilingual summary/detail. It models loading, error, empty, and ready presentation resources through the shared capability-state presenter and accessible screen.

Web 层还定义了 `LocalCapabilityAvailabilityV1`：这是一个展示资源，使用数值 `schema_version: 1`、稳定的 `capability_id`、双语 capability 文案、`available` 或 `unavailable` 状态，以及可选的双语 summary/detail。它通过共享 capability-state presenter 与可访问 screen 表示 loading、error、empty 和 ready 展示资源。

Despite the similar V1 name, this Web resource is not interchangeable with the Rust wire DTO. The Rust DTO is strict and currently permits only the unregistered-integration projection; the Web resource is a local presentation model and may represent UI resource states. Any future cross-boundary transport requires a separately versioned contract and focused evidence.

尽管名称都带有 V1，Web 展示资源不能与 Rust wire DTO 互换。Rust DTO 是严格的，当前只允许未注册 integration 的投影；Web 资源是本地展示模型，可以表示 UI resource state。未来如需跨边界传输，必须另行定义带版本的契约并提供 focused 证据。

## Boundary and Evidence / 边界与证据

The focused receipts prove parsing, mapping, projection, and accessible static markup within the named local test scope. They do not prove registration, shared-core execution, provider access, data persistence, Docker or PostgreSQL runtime, browser visual rendering, authentication, remote CI, release packaging, or production behavior.

focused 回执证明了所列本地 test scope 内的解析、映射、投影与可访问静态 markup。它们不证明集成注册、共享核心执行、provider 访问、数据持久化、Docker 或 PostgreSQL runtime、browser visual rendering、认证、远端 CI、release packaging 或生产行为。

The evidence states used here follow the Wave 1 definitions: `passed` is limited to a named command and scope; `unobserved` means no reviewable receipt was produced; and `deferred` records an intentionally out-of-slice external activity. See [the Wave 2 evidence matrix](../verification/wave-2-local-capability-evidence-matrix.md) for the exact commands and excluded surfaces.

本文使用的证据状态沿用 Wave 1 定义：`passed` 仅限于指定 command 与 scope；`unobserved` 表示未产生可审阅回执；`deferred` 记录有意留在本切片之外的外部活动。精确 command 与排除的 surface 见 [Wave 2 证据矩阵](../verification/wave-2-local-capability-evidence-matrix.md)。
