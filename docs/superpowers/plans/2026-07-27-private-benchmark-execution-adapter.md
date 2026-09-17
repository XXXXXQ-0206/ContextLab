# Private Benchmark Execution Adapter / 私有 Benchmark 执行适配器

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
advances Criterion 3 by connecting the exact version-bound benchmark definition to a usable local
evaluation workflow. It also preserves the charter's Context-first and replayable-history rules:
execution requests must name one immutable Context commit and one immutable definition binding.

本增量直接推进条件 3：将精确版本绑定的 benchmark definition 连接到可实际使用的本地评测 workflow。
同时保持宪章中的 Context-first 与可回放历史规则：execution request 必须明确一条不可变 Context commit 与一条
不可变 definition binding。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The protected
application boundary now exists, but its idempotency header is not yet connected to the reusable
storage contract: the adapter drops `Idempotency-Key`, and a fresh `Utc::now()` makes an HTTP retry
fail the service's exact replay check. The exact binding scope, `ContextPermission::Write` policy,
authentication-before-quota ordering, and evaluator-unavailable fail-closed behavior need focused
integration evidence before a local execution control is usable.

 Rust storage 现已组装并测试 provider-free `BenchmarkDefinitionBindingExecutionSelection`，受保护的
 application boundary 也已存在；但 `Idempotency-Key` 尚未接入可复用 storage contract：adapter 丢弃了 header，
 且每次新建 `Utc::now()` 会使 HTTP retry 无法通过 service 的精确 replay 校验。exact binding scope、
 `ContextPermission::Write` policy、authentication-before-quota 顺序与 evaluator unavailable fail-closed
 行为仍需聚焦集成证据，之后才能视为可用的 local execution control。

**Why now / 为什么现在优先:** The exact binding inspection and selection prerequisites are
freshly green, and the execution service has deterministic replay and projection behavior. A protected
local adapter is the nearest remaining Criterion 3 gap and makes the existing core reachable without
adding public write methods or relying on external deployment evidence.

**为什么现在优先：** exact binding inspection 与 selection 前置已新鲜通过，execution service 也具备确定性的
replay 与 projection 语义。protected local adapter 是条件 3 最近的剩余缺口，可以在不增加 public write method、
也不依赖外部部署证据的前提下让既有 core 可被使用。

**Minimal affected boundary / 最小受影响边界:** First add the protected local application request
and response contract in `server/api`, reusing the existing binding repository, execution service,
authentication/RBAC/audit/rate-limit/idempotency boundaries. Add only the non-public local SDK/BFF/Web
adapter after the server contract is green. The default router, checked-in OpenAPI, and public SDK stay
unchanged. The evaluator remains an injected port; an unavailable evaluator fails closed.

第一步在 `server/api` 增加 protected local application request/response contract，复用既有 binding repository、
execution service、authentication/RBAC/audit/rate-limit/idempotency boundary。只有 server contract 变绿后，才增加
非公开 local SDK/BFF/Web adapter。default router、已检入 OpenAPI 与 public SDK 保持不变。evaluator 仍是注入式 port；
evaluator 不可用时 fail closed。

**Explicit non-goals / 明确非目标:** No public REST/OpenAPI/public SDK method, public or production
provider access, secret reading, raw case/input/output response, client-owned policy/diff calculation,
operator transport, release claim, production readiness, or second graph-diff calculator.
`GraphDiff::between` remains the sole graph-diff calculator.

不新增 public REST/OpenAPI/public SDK method，不访问 public 或 production provider，不读取 secret，不返回 raw
case/input/output，不允许 client 自行计算 policy/diff，不增加 operator transport、release 声明、production readiness
或第二个 graph-diff calculator。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Fresh verification before implementation / 实现前的新鲜验证:** Re-read auth/RBAC/rate-limit,
idempotency, benchmark binding, and API route contracts; the existing private/public exclusion and
evaluator-unavailable tests are green, while storage-backed replay and changed-key conflict remain
red/unobserved. Implementation is limited to the storage idempotency mapping, replay timestamp reuse,
and focused API/storage tests.

**实现前的新鲜验证：** 已重新阅读 auth/RBAC/rate-limit、idempotency、benchmark binding 与 API route contract；
现有 private/public exclusion 与 evaluator-unavailable 测试通过，但 storage-backed replay 与 changed-key conflict
仍为 red/unobserved。实现范围仅限 storage idempotency mapping、replay timestamp reuse 以及聚焦 API/storage test。

## Implementation Tasks / 实施任务

- [x] Add the protected local request/response contract and public-surface/fail-closed tests in the API boundary.
- [x] Connect `Idempotency-Key` and request digest to the existing execution service and prove exact replay/conflict.
- [x] Reuse stored evidence timestamp during HTTP replay and prove exact binding/scope behavior.
- [x] Add local SDK/BFF/Web `data -> presenter -> screen` only after the server contract is green.
- [x] Mount the default-off execution inspector in the existing Context workspace Benchmark region behind `localLifecycleEnabled`.
- [x] Record fresh local verification and select the next increment; keep the long-term goal active.

## Integration closure / 集成收束

The local SDK, same-origin BFF, Web data/presenter/screen, and default-off inspector now form one
exact project/Context/commit workflow. The inspector is mounted in the existing Benchmark region,
holds the Bearer token only in request memory, generates one request-scoped idempotency key, and
delegates request validation and response parsing to the shared local SDK contract. The UI exposes
only redacted server-owned receipt metadata and never calculates evaluation policy or Diff.

local SDK、同源 BFF、Web data/presenter/screen 与默认关闭的 inspector 现已组成一条精确
project/Context/commit workflow。inspector 已挂载到现有 Benchmark 区域；Bearer token 只保存在请求内存中，
每次生成 request-scoped idempotency key，并将 request validation 与 response parsing 委托给共享 local SDK contract。
UI 只呈现服务端控制的脱敏 receipt metadata，不计算 evaluation policy 或 Diff。
