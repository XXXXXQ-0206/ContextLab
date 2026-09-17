Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$verifier = Join-Path $repositoryRoot "scripts/verify-local-contracts.ps1"

if (-not (Test-Path -LiteralPath $verifier -PathType Leaf)) {
    throw "Expected local contract verifier at $verifier"
}

$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("contextlab-local-contracts-" + [guid]::NewGuid())

function Write-TextFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    $parent = Split-Path -Parent $Path
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
    [System.IO.File]::WriteAllText($Path, $Content, [System.Text.UTF8Encoding]::new($false))
}

function Assert-Contains {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Actual,
        [Parameter(Mandatory = $true)]
        [string]$Expected
    )

    if ($Actual.IndexOf($Expected, [System.StringComparison]::Ordinal) -lt 0) {
        throw "Expected output to contain '$Expected', received: $Actual"
    }
}

function Invoke-Verifier {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FixtureRoot,
        [string]$DiffPath
    )

    $arguments = @("-NoProfile", "-File", $verifier, "-Root", $FixtureRoot)
    if ($null -ne $DiffPath) {
        $arguments += @("-DiffPath", $DiffPath)
    }

    $output = & powershell @arguments 2>&1
    return [pscustomobject]@{
        ExitCode = $LASTEXITCODE
        Output = [string]::Join("`n", @($output))
    }
}

try {
    $fixtureRoot = Join-Path $temporaryRoot "fixture"
    $benchmarkRoutePath = "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}"
    $benchmarkDecisionRoutePath = "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}/workspace"
    $safeApplicationSource = @'
fn compare_graphs() {
    let _diff = GraphDiff::between(original.graph(), revised.graph());
}
'@
    $safeBenchmarkSource = @'
pub struct BenchmarkWorkspaceProjectionV1 {
    summary: String,
}

pub struct BenchmarkRunWorkspaceSummaryV1 {
    case_id: String,
}
'@
    $safeBenchmarkDefinitionAuthoringSource = @'
pub const BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION: u16 = 1;

pub struct BenchmarkDefinitionBindingCommand {
    schema_version: u16,
}

impl BenchmarkDefinitionBindingCommand {
    pub fn try_new(schema_version: u16) -> Result<Self, ()> {
        if schema_version != BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION {
            return Err(());
        }

        Ok(Self { schema_version })
    }
}
'@
    $safeArchitectureSource = @'
### Private Benchmark Definition Authoring / 私有 Benchmark 定义创作

Private benchmark authoring now has a reusable Rust command and storage port. The command validates
schema version 1, complete suite membership, deterministic dataset ordering, an exact immutable
project/context/commit source, an equal expected branch head, and replay metadata.
This is a private core/storage contract; public REST/OpenAPI/public SDK
writes, Web mutation, provider calls, Context commit mutation, and release readiness remain out of
scope. GraphDiff::between remains the sole graph-diff calculator.

私有 benchmark authoring 现已具备可复用 Rust command 与 storage port。command 校验 schema version 1、
完整 suite membership、确定性 dataset ordering、精确不可变 project/context/commit source、
相等的 expected branch head 与 replay metadata。
本增量是 private core/storage contract；public REST/OpenAPI/public SDK write、Web mutation、
provider call、Context commit mutation 与 release readiness 均不在范围内。GraphDiff::between
仍是唯一 graph-diff calculator。
'@
    $safeBenchmarkDefinitionIntegrationSource = @'
### Private Benchmark Definition Binding / 私有 Benchmark 定义绑定

BenchmarkDefinitionBindingCommand is a private Rust/storage contract for authoring one complete
suite and its datasets at an exact Context commit. It carries schema version 1, stable binding
identity, project/Context/commit scope, branch and expected-head guard, issuer-scoped principal,
idempotency key, request digest, capture time, and immutable definitions. The suite dataset IDs
must exactly equal the supplied dataset IDs in stable order. Memory and PostgreSQL expose the same
writer disposition (created or replayed) and exact read/list ports. PostgreSQL verifies write
authorization and branch head inside one transaction, persists definitions and binding atomically,
and uses microsecond capture-time normalization for replay parity. The binding route is not yet
public or transport-complete; no public OpenAPI/public SDK write is added.

BenchmarkDefinitionBindingCommand 是私有 Rust/storage contract，用于在精确 Context commit 上创作
一条完整 suite 及其 dataset。它携带 schema version 1、稳定 binding identity、project/Context/
commit scope、branch 与 expected-head guard、issuer-scoped principal、idempotency key、request
digest、capture time 与不可变 definition。suite dataset ID 必须与传入 dataset ID 按稳定顺序
完全一致。Memory 与 PostgreSQL 暴露相同的 writer disposition（created 或 replayed）及 exact
read/list port。PostgreSQL 在一个 transaction 内复核 write authorization 和 branch head，原子
持久化 definitions 与 binding，并通过微秒级 capture-time normalization 保持 replay parity。
binding route 还未成为 public 或完整 transport；没有新增 public OpenAPI/public SDK write。
'@
    $safeKnowledgeSource = @'
pub struct KnowledgeLocalCitationProjectionV1 {
    query_fingerprint: String,
    citations: Vec<String>,
}
'@
    $safeReplaySource = @'
pub struct KnowledgeMemoryReplayProjection {
    citation_projection_id: String,
    memory_id: String,
}
'@
    $safeServerLibSource = @'
enum PublicGetRoute {
    Health,
}

impl PublicGetRoute {
    const fn path(self) -> &'static str {
        "/healthz"
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        router.route(self.path(), axum::routing::get(routes::health))
    }
}

enum ProtectedBenchmarkWorkspaceGetRoute {
    Workspace,
    DecisionWorkspace,
}

const PROTECTED_BENCHMARK_WORKSPACE_GET_ROUTES: &[ProtectedBenchmarkWorkspaceGetRoute] =
    &[
        ProtectedBenchmarkWorkspaceGetRoute::Workspace,
        ProtectedBenchmarkWorkspaceGetRoute::DecisionWorkspace,
    ];

impl ProtectedBenchmarkWorkspaceGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::Workspace => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}"
            }
            Self::DecisionWorkspace => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}/workspace"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::Workspace => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_workspace),
            ),
            Self::DecisionWorkspace => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_workspace_by_decision),
            ),
        }
    }
}

pub fn build_protected_router_with_state(state: AppState) -> Router {
    let mut protected_benchmark_workspace_read_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_WORKSPACE_GET_ROUTES {
        protected_benchmark_workspace_read_router =
            route.install(protected_benchmark_workspace_read_router);
    }
    let protected_benchmark_workspace_read_router = protected_benchmark_workspace_read_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_workspace_read_request,
        ));

    public_router()
        .merge(protected_benchmark_workspace_read_router)
        .with_state(state)
}

fn public_router() -> Router<AppState> {
    Router::<AppState>::new()
}
'@
    $safeServerRoutesSource = @'
pub struct LocalBenchmarkWorkspaceResponse {
    schema_version: &'static str,
    project_id: String,
    context_id: String,
    revised: LocalBenchmarkWorkspaceScopeResponse,
    baseline: Option<LocalBenchmarkWorkspaceScopeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_pair_witness: Option<LocalBenchmarkDecisionPairWitnessResponse>,
    projection: BenchmarkWorkspaceProjectionV1,
}

struct LocalBenchmarkWorkspaceScopeResponse {
    commit_id: String,
    cohort_id: String,
}

struct LocalBenchmarkDecisionScopeResponse {
    commit_id: String,
    decision_id: String,
}

struct LocalBenchmarkDecisionPairWitnessResponse {
    schema_version: u16,
    project_id: String,
    context_id: String,
    baseline: LocalBenchmarkDecisionScopeResponse,
    revised: LocalBenchmarkDecisionScopeResponse,
}

pub async fn local_benchmark_workspace() {}
pub async fn local_benchmark_workspace_by_decision() {}
'@
    $safeOpenApiSource = @'
{
  "openapi": "3.1.0",
  "paths": {
    "/healthz": {
      "get": {}
    }
  }
}
'@
    $safeLocalSdkPackageSource = @'
{
  "name": "@contextlab/local-sdk",
  "exports": {
    ".": "./src/index.ts"
  }
}
'@
    $safePublicSdkPackageSource = @'
{
  "name": "@contextlab/ts-sdk",
  "exports": {
    ".": "./src/index.ts"
  }
}
'@
    $safeLocalSdkIndexSource = @'
export { ContextLabLocalClient } from "./client";
export { parseLocalBenchmarkWorkspace } from "./benchmark-workspace";
'@
    $safeLocalSdkBenchmarkSource = @'
export type LocalBenchmarkWorkspace = {
  schema_version: "contextlab.local-benchmark-workspace.v1";
};

export function parseLocalBenchmarkWorkspace(value: unknown): LocalBenchmarkWorkspace {
  return value as LocalBenchmarkWorkspace;
}
'@
    $safeLocalSdkClientSource = @'
import type { LocalBenchmarkWorkspace } from "./benchmark-workspace";
import { parseLocalBenchmarkWorkspace } from "./benchmark-workspace";

export class ContextLabLocalClient {
  async getBenchmarkWorkspace(
    projectId: string,
    contextId: string,
    commitId: string,
    cohortId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalBenchmarkWorkspace> {
    const response = await this.request<unknown>(
      `/api/v1/local/projects/${projectId}/contexts/${contextId}/commits/${commitId}/benchmark-workspace/${cohortId}`,
      credentials
    );
    return parseLocalBenchmarkWorkspace(response);
  }
}
'@
    $safePublicSdkSource = @'
export class ContextLabClient {}
'@

    Write-TextFile (Join-Path $fixtureRoot "crates/diff-engine/src/application.rs") $safeApplicationSource
    Write-TextFile (Join-Path $fixtureRoot "crates/evaluation/src/benchmark_workspace.rs") $safeBenchmarkSource
    Write-TextFile (Join-Path $fixtureRoot "crates/storage/src/benchmark_definition_authoring.rs") $safeBenchmarkDefinitionAuthoringSource
    Write-TextFile (Join-Path $fixtureRoot "crates/knowledge/src/lib.rs") $safeKnowledgeSource
    Write-TextFile (Join-Path $fixtureRoot "crates/knowledge/src/memory_replay.rs") $safeReplaySource
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/routes.rs") $safeServerRoutesSource
    Write-TextFile (Join-Path $fixtureRoot "docs/api/openapi.json") $safeOpenApiSource
    Write-TextFile (Join-Path $fixtureRoot "packages/local-sdk/package.json") $safeLocalSdkPackageSource
    Write-TextFile (Join-Path $fixtureRoot "packages/local-sdk/src/index.ts") $safeLocalSdkIndexSource
    Write-TextFile (Join-Path $fixtureRoot "packages/local-sdk/src/benchmark-workspace.ts") $safeLocalSdkBenchmarkSource
    Write-TextFile (Join-Path $fixtureRoot "packages/local-sdk/src/client.ts") $safeLocalSdkClientSource
    Write-TextFile (Join-Path $fixtureRoot "packages/ts-sdk/package.json") $safePublicSdkPackageSource
    Write-TextFile (Join-Path $fixtureRoot "packages/ts-sdk/src/index.ts") $safePublicSdkSource
    Write-TextFile (Join-Path $fixtureRoot "ARCHITECTURE.md") $safeArchitectureSource
    Write-TextFile (Join-Path $fixtureRoot "docs/api/wave-1-integration-contracts.md") $safeBenchmarkDefinitionIntegrationSource

    $safeDiff = Join-Path $temporaryRoot "safe.diff"
    Write-TextFile $safeDiff @'
--- a/crates/evaluation/src/benchmark_workspace.rs
+++ b/crates/evaluation/src/benchmark_workspace.rs
@@
+    regression_status: String,
'@
    $safe = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($safe.ExitCode -ne 0) {
        throw "Expected safe fixture to pass, received exit $($safe.ExitCode): $($safe.Output)"
    }
    Assert-Contains $safe.Output "graph_diff_application=passed count=1"
    Assert-Contains $safe.Output "safe_local_dto_fields=passed"
    Assert-Contains $safe.Output "benchmark_workspace_route=passed"
    Assert-Contains $safe.Output "benchmark_workspace_public_surface=passed"
    Assert-Contains $safe.Output "benchmark_workspace_local_sdk=passed"
    Assert-Contains $safe.Output "benchmark_definition_schema=passed"
    Assert-Contains $safe.Output "benchmark_definition_graph_diff=passed count=0"
    Assert-Contains $safe.Output "benchmark_definition_public_boundary=passed"
    Assert-Contains $safe.Output "benchmark_definition_vocabulary=passed"
    Assert-Contains $safe.Output "public_write_additions=passed"
    Assert-Contains $safe.Output "overall=passed"

    $unknownWitnessSource = $safeServerRoutesSource.Replace(
        "    revised: LocalBenchmarkDecisionScopeResponse,`n}",
        "    revised: LocalBenchmarkDecisionScopeResponse,`n    unknown: String,`n}"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/routes.rs") $unknownWitnessSource
    $unknownWitness = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($unknownWitness.ExitCode -eq 0) {
        throw "Expected unknown decision-pair witness field to be blocked: $($unknownWitness.Output)"
    }
    Assert-Contains $unknownWitness.Output "safe_local_dto_fields=blocked"
    Assert-Contains $unknownWitness.Output "invalid-safe-dto-fields:LocalBenchmarkDecisionPairWitnessResponse"
    Assert-Contains $unknownWitness.Output "overall=blocked"

    $rawWitnessSource = $safeServerRoutesSource.Replace(
        "    revised: LocalBenchmarkDecisionScopeResponse,`n}",
        "    revised: LocalBenchmarkDecisionScopeResponse,`n    payload: String,`n}"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/routes.rs") $rawWitnessSource
    $rawWitness = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($rawWitness.ExitCode -eq 0) {
        throw "Expected raw decision-pair witness field to be blocked: $($rawWitness.Output)"
    }
    Assert-Contains $rawWitness.Output "safe_local_dto_fields=blocked"
    Assert-Contains $rawWitness.Output "raw-safe-dto-field:LocalBenchmarkDecisionPairWitnessResponse.payload"
    Assert-Contains $rawWitness.Output "overall=blocked"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/routes.rs") $safeServerRoutesSource

    $openApiLeakSource = @"
{
  "openapi": "3.1.0",
  "paths": {
    "$benchmarkRoutePath": {
      "get": {}
    }
  }
}
"@
    Write-TextFile (Join-Path $fixtureRoot "docs/api/openapi.json") $openApiLeakSource
    $openApiLeak = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($openApiLeak.ExitCode -eq 0) {
        throw "Expected OpenAPI benchmark workspace leak to be blocked: $($openApiLeak.Output)"
    }
    Assert-Contains $openApiLeak.Output "benchmark_workspace_public_surface=blocked"
    Assert-Contains $openApiLeak.Output "benchmark-workspace-openapi-path:$benchmarkRoutePath"
    Write-TextFile (Join-Path $fixtureRoot "docs/api/openapi.json") $safeOpenApiSource

    Write-TextFile (Join-Path $fixtureRoot "packages/ts-sdk/src/index.ts") @'
export type BenchmarkWorkspace = {};
'@
    $publicSdkLeak = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($publicSdkLeak.ExitCode -eq 0) {
        throw "Expected public SDK benchmark workspace leak to be blocked: $($publicSdkLeak.Output)"
    }
    Assert-Contains $publicSdkLeak.Output "benchmark_workspace_public_surface=blocked"
    Assert-Contains $publicSdkLeak.Output "benchmark-workspace-public-sdk:packages/ts-sdk/src/index.ts"
    Write-TextFile (Join-Path $fixtureRoot "packages/ts-sdk/src/index.ts") $safePublicSdkSource

    Write-TextFile (Join-Path $fixtureRoot "docs/api/openapi.json") @'
{
  "openapi": "3.1.0",
  "paths": {
    "/api/v1/benchmark-definitions": {
      "post": {}
    }
  }
}
'@
    $benchmarkDefinitionOpenApiLeak = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($benchmarkDefinitionOpenApiLeak.ExitCode -eq 0) {
        throw "Expected benchmark definition OpenAPI leak to be blocked: $($benchmarkDefinitionOpenApiLeak.Output)"
    }
    Assert-Contains $benchmarkDefinitionOpenApiLeak.Output "benchmark_definition_public_boundary=blocked"
    Assert-Contains $benchmarkDefinitionOpenApiLeak.Output "benchmark-definition-public-openapi:/api/v1/benchmark-definitions"
    Write-TextFile (Join-Path $fixtureRoot "docs/api/openapi.json") $safeOpenApiSource

    Write-TextFile (Join-Path $fixtureRoot "packages/ts-sdk/src/index.ts") @'
export type BenchmarkDefinitionBinding = {};
'@
    $benchmarkDefinitionPublicSdkLeak = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($benchmarkDefinitionPublicSdkLeak.ExitCode -eq 0) {
        throw "Expected benchmark definition public SDK leak to be blocked: $($benchmarkDefinitionPublicSdkLeak.Output)"
    }
    Assert-Contains $benchmarkDefinitionPublicSdkLeak.Output "benchmark_definition_public_boundary=blocked"
    Assert-Contains $benchmarkDefinitionPublicSdkLeak.Output "benchmark-definition-public-sdk:packages/ts-sdk/src/index.ts"
    Write-TextFile (Join-Path $fixtureRoot "packages/ts-sdk/src/index.ts") $safePublicSdkSource

    $schemaVersionDriftSource = $safeBenchmarkDefinitionAuthoringSource.Replace(
        "pub const BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION: u16 = 1;",
        "pub const BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION: u16 = 2;"
    )
    Write-TextFile (Join-Path $fixtureRoot "crates/storage/src/benchmark_definition_authoring.rs") $schemaVersionDriftSource
    $schemaVersionDrift = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($schemaVersionDrift.ExitCode -eq 0) {
        throw "Expected benchmark definition schema version drift to be blocked: $($schemaVersionDrift.Output)"
    }
    Assert-Contains $schemaVersionDrift.Output "benchmark_definition_schema=blocked"
    Assert-Contains $schemaVersionDrift.Output "benchmark-definition-schema-version"
    Write-TextFile (Join-Path $fixtureRoot "crates/storage/src/benchmark_definition_authoring.rs") $safeBenchmarkDefinitionAuthoringSource

    Write-TextFile (Join-Path $fixtureRoot "crates/storage/src/benchmark_definition_authoring.rs") ($safeBenchmarkDefinitionAuthoringSource + @'

fn calculate_definition_diff() {
    let _diff = GraphDiff::between(original.graph(), revised.graph());
}
'@)
    $benchmarkDefinitionGraphDiff = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($benchmarkDefinitionGraphDiff.ExitCode -eq 0) {
        throw "Expected benchmark definition graph diff calculator to be blocked: $($benchmarkDefinitionGraphDiff.Output)"
    }
    Assert-Contains $benchmarkDefinitionGraphDiff.Output "benchmark_definition_graph_diff=blocked count=1"
    Assert-Contains $benchmarkDefinitionGraphDiff.Output "benchmark-definition-graph-diff-calculator-count:1"
    Write-TextFile (Join-Path $fixtureRoot "crates/storage/src/benchmark_definition_authoring.rs") $safeBenchmarkDefinitionAuthoringSource

    $vocabularyDriftSource = $safeArchitectureSource.Replace(
        "### Private Benchmark Definition Authoring / 私有 Benchmark 定义创作",
        "### Private Benchmark Definition Authoring"
    )
    Write-TextFile (Join-Path $fixtureRoot "ARCHITECTURE.md") $vocabularyDriftSource
    $vocabularyDrift = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($vocabularyDrift.ExitCode -eq 0) {
        throw "Expected benchmark definition vocabulary drift to be blocked: $($vocabularyDrift.Output)"
    }
    Assert-Contains $vocabularyDrift.Output "benchmark_definition_vocabulary=blocked"
    Assert-Contains $vocabularyDrift.Output "benchmark-definition-vocabulary:architecture-heading"
    Write-TextFile (Join-Path $fixtureRoot "ARCHITECTURE.md") $safeArchitectureSource

    $writeRouteSource = $safeServerLibSource.Replace(
        "axum::routing::get(routes::local_benchmark_workspace)",
        "axum::routing::post(routes::local_benchmark_workspace)"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $writeRouteSource
    $writeRoute = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($writeRoute.ExitCode -eq 0) {
        throw "Expected non-GET benchmark workspace route to be blocked: $($writeRoute.Output)"
    }
    Assert-Contains $writeRoute.Output "benchmark_workspace_route=blocked"
    Assert-Contains $writeRoute.Output "benchmark-workspace-route-method:post"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $duplicateHandlerSource = $safeServerLibSource.Replace(
        "routes::local_benchmark_workspace_by_decision",
        "routes::local_benchmark_workspace"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $duplicateHandlerSource
    $duplicateHandler = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($duplicateHandler.ExitCode -eq 0) {
        throw "Expected duplicate benchmark workspace handler to be blocked: $($duplicateHandler.Output)"
    }
    Assert-Contains $duplicateHandler.Output "benchmark_workspace_route=blocked"
    Assert-Contains $duplicateHandler.Output "benchmark-workspace-handler-registration-count:local_benchmark_workspace:2"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $missingHandlerSource = $safeServerLibSource.Replace(
        "routes::local_benchmark_workspace_by_decision",
        "routes::missing_benchmark_workspace_by_decision"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $missingHandlerSource
    $missingHandler = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($missingHandler.ExitCode -eq 0) {
        throw "Expected missing benchmark workspace handler to be blocked: $($missingHandler.Output)"
    }
    Assert-Contains $missingHandler.Output "benchmark_workspace_route=blocked"
    Assert-Contains $missingHandler.Output "benchmark-workspace-handler-registration-count:local_benchmark_workspace_by_decision:0"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $nonLocalRouteSource = $safeServerLibSource.Replace(
        $benchmarkRoutePath,
        $benchmarkRoutePath.Replace("/api/v1/local/", "/api/v1/")
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $nonLocalRouteSource
    $nonLocalRoute = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($nonLocalRoute.ExitCode -eq 0) {
        throw "Expected non-local benchmark workspace route to be blocked: $($nonLocalRoute.Output)"
    }
    Assert-Contains $nonLocalRoute.Output "benchmark_workspace_route=blocked"
    Assert-Contains $nonLocalRoute.Output "benchmark-workspace-route-not-local"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $duplicateRouteSource = $safeServerLibSource.Replace(
        "ProtectedBenchmarkWorkspaceGetRoute::DecisionWorkspace,",
        "ProtectedBenchmarkWorkspaceGetRoute::Workspace,"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $duplicateRouteSource
    $duplicateRoute = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($duplicateRoute.ExitCode -eq 0) {
        throw "Expected duplicate benchmark workspace route to be blocked: $($duplicateRoute.Output)"
    }
    Assert-Contains $duplicateRoute.Output "benchmark_workspace_route=blocked"
    Assert-Contains $duplicateRoute.Output "benchmark-workspace-protected-catalog"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $missingCatalogSource = $safeServerLibSource.Replace(
        "        ProtectedBenchmarkWorkspaceGetRoute::DecisionWorkspace,`n",
        ""
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $missingCatalogSource
    $missingCatalog = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($missingCatalog.ExitCode -eq 0) {
        throw "Expected missing benchmark workspace catalog entry to be blocked: $($missingCatalog.Output)"
    }
    Assert-Contains $missingCatalog.Output "benchmark_workspace_route=blocked"
    Assert-Contains $missingCatalog.Output "benchmark-workspace-protected-catalog"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $publicRouterSource = $safeServerLibSource.Replace(
        "fn public_router() -> Router<AppState> {`n    Router::<AppState>::new()`n}",
        "fn public_router() -> Router<AppState> {`n    Router::<AppState>::new().merge(protected_benchmark_workspace_read_router)`n}"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $publicRouterSource
    $publicRouter = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($publicRouter.ExitCode -eq 0) {
        throw "Expected public router benchmark workspace registration to be blocked: $($publicRouter.Output)"
    }
    Assert-Contains $publicRouter.Output "benchmark_workspace_public_surface=blocked"
    Assert-Contains $publicRouter.Output "benchmark-workspace-public-router"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/lib.rs") $safeServerLibSource

    $unparsedClientSource = $safeLocalSdkClientSource.Replace(
        "return parseLocalBenchmarkWorkspace(response);",
        "return response as LocalBenchmarkWorkspace;"
    )
    Write-TextFile (Join-Path $fixtureRoot "packages/local-sdk/src/client.ts") $unparsedClientSource
    $unparsedClient = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($unparsedClient.ExitCode -eq 0) {
        throw "Expected unparsed local SDK benchmark workspace response to be blocked: $($unparsedClient.Output)"
    }
    Assert-Contains $unparsedClient.Output "benchmark_workspace_local_sdk=blocked"
    Assert-Contains $unparsedClient.Output "benchmark-workspace-client-parser-count:0"
    Write-TextFile (Join-Path $fixtureRoot "packages/local-sdk/src/client.ts") $safeLocalSdkClientSource

    $writeDiff = Join-Path $temporaryRoot "public-write.diff"
    Write-TextFile $writeDiff @'
--- a/server/api/src/routes.rs
+++ b/server/api/src/routes.rs
@@
+router.route("/api/v1/contexts", post(create_context));
'@
    $publicWrite = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $writeDiff
    if ($publicWrite.ExitCode -eq 0) {
        throw "Expected public write fixture to be blocked: $($publicWrite.Output)"
    }
    Assert-Contains $publicWrite.Output "public_write_additions=blocked"
    Assert-Contains $publicWrite.Output "overall=blocked"

    $openApiWriteDiff = Join-Path $temporaryRoot "benchmark-definition-openapi-write.diff"
    Write-TextFile $openApiWriteDiff @'
--- a/docs/api/openapi.json
+++ b/docs/api/openapi.json
@@
+    "post": { "operationId": "createBenchmarkDefinition" }
'@
    $openApiWrite = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $openApiWriteDiff
    if ($openApiWrite.ExitCode -eq 0) {
        throw "Expected benchmark definition OpenAPI mutation addition to be blocked: $($openApiWrite.Output)"
    }
    Assert-Contains $openApiWrite.Output "public_write_additions=blocked"
    Assert-Contains $openApiWrite.Output "public-write-addition:docs/api/openapi.json"

    $publicSdkWriteDiff = Join-Path $temporaryRoot "benchmark-definition-public-sdk-write.diff"
    Write-TextFile $publicSdkWriteDiff @'
--- a/packages/ts-sdk/src/client.ts
+++ b/packages/ts-sdk/src/client.ts
@@
+  async createBenchmarkDefinition(): Promise<void> {}
'@
    $publicSdkWrite = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $publicSdkWriteDiff
    if ($publicSdkWrite.ExitCode -eq 0) {
        throw "Expected benchmark definition public SDK mutation addition to be blocked: $($publicSdkWrite.Output)"
    }
    Assert-Contains $publicSdkWrite.Output "public_write_additions=blocked"
    Assert-Contains $publicSdkWrite.Output "public-write-addition:packages/ts-sdk/src/client.ts"

    $rawResponseSource = $safeServerRoutesSource.Replace(
        "    projection: BenchmarkWorkspaceProjectionV1,",
        "    projection: BenchmarkWorkspaceProjectionV1,`n    case: String,`n    input: String,`n    expected_output: String,`n    model_output: String,"
    )
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/routes.rs") $rawResponseSource
    $rawResponse = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($rawResponse.ExitCode -eq 0) {
        throw "Expected raw benchmark workspace response fixture to be blocked: $($rawResponse.Output)"
    }
    Assert-Contains $rawResponse.Output "safe_local_dto_fields=blocked"
    Assert-Contains $rawResponse.Output "raw-safe-dto-field:LocalBenchmarkWorkspaceResponse.case"
    Assert-Contains $rawResponse.Output "raw-safe-dto-field:LocalBenchmarkWorkspaceResponse.input"
    Assert-Contains $rawResponse.Output "raw-safe-dto-field:LocalBenchmarkWorkspaceResponse.expected_output"
    Assert-Contains $rawResponse.Output "raw-safe-dto-field:LocalBenchmarkWorkspaceResponse.model_output"
    Assert-Contains $rawResponse.Output "overall=blocked"
    Write-TextFile (Join-Path $fixtureRoot "server/api/src/routes.rs") $safeServerRoutesSource

    Write-TextFile (Join-Path $fixtureRoot "crates/diff-engine/src/application.rs") @'
fn compare_graphs() {}
'@
    $missingGraphDiff = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($missingGraphDiff.ExitCode -eq 0) {
        throw "Expected missing graph diff fixture to be blocked: $($missingGraphDiff.Output)"
    }
    Assert-Contains $missingGraphDiff.Output "graph_diff_application=blocked count=0"
    Assert-Contains $missingGraphDiff.Output "overall=blocked"

    Write-TextFile (Join-Path $fixtureRoot "crates/diff-engine/src/application.rs") @'
fn compare_graphs() {
    let _first = GraphDiff::between(original.graph(), revised.graph());
    let _second = GraphDiff::between(original.graph(), revised.graph());
}
'@
    $duplicateGraphDiff = Invoke-Verifier -FixtureRoot $fixtureRoot -DiffPath $safeDiff
    if ($duplicateGraphDiff.ExitCode -eq 0) {
        throw "Expected duplicate graph diff fixture to be blocked: $($duplicateGraphDiff.Output)"
    }
    Assert-Contains $duplicateGraphDiff.Output "graph_diff_application=blocked count=2"
    Assert-Contains $duplicateGraphDiff.Output "overall=blocked"

    Write-Output "verify-local-contracts fixture tests passed"
}
finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}
