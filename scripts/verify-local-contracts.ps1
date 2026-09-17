[CmdletBinding()]
param(
    [string]$Root,
    [string]$DiffPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-ReparsePoint {
    param([Parameter(Mandatory = $true)] [System.IO.FileSystemInfo]$Item)

    return ($Item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0
}

function Get-SafeItem {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [string]$Label,
        [switch]$Directory
    )

    $pathType = "Leaf"
    if ($Directory) {
        $pathType = "Container"
    }
    if (-not (Test-Path -LiteralPath $Path -PathType $pathType)) {
        throw "missing-${Label}:$Path"
    }

    $item = Get-Item -LiteralPath $Path -Force
    if (Test-ReparsePoint $item) {
        throw "unsafe-reparse-point-${Label}:$Path"
    }

    return $item
}

function Get-SafeSourceFiles {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [string]$Label
    )

    $rootItem = Get-SafeItem -Path $Path -Label $Label -Directory
    $pending = [System.Collections.Generic.Stack[string]]::new()
    $files = [System.Collections.Generic.List[System.IO.FileInfo]]::new()
    $pending.Push($rootItem.FullName)

    while ($pending.Count -gt 0) {
        $directory = $pending.Pop()
        foreach ($child in Get-ChildItem -LiteralPath $directory -Force) {
            if (Test-ReparsePoint $child) {
                throw "unsafe-reparse-point-${Label}:$($child.FullName)"
            }
            if ($child.PSIsContainer) {
                $pending.Push($child.FullName)
            }
            elseif (
                ($child.Extension -eq ".ts" -or $child.Extension -eq ".tsx") -and
                $child.Name -notmatch "\.(?:test|spec)\.(?:ts|tsx)$"
            ) {
                $files.Add($child)
            }
        }
    }

    return @($files.ToArray())
}

function Get-StructBody {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$StructName
    )

    $match = [regex]::Match(
        $Source,
        "(?ms)^\s*(?:pub(?:\([^\)]*\))?\s+)?struct\s+$([regex]::Escape($StructName))\s*\{(?<body>.*?)^\s*\}"
    )
    if (-not $match.Success) {
        throw "missing-safe-dto-declaration:$StructName"
    }

    return $match.Groups["body"].Value
}

function Get-StructFieldNames {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$StructName
    )

    $body = Get-StructBody -Source $Source -StructName $StructName
    return @(
        [regex]::Matches(
            $body,
            "(?m)^\s*(?:pub(?:\([^\)]*\))?\s+)?(?<field>[A-Za-z_][A-Za-z0-9_]*)\s*:"
        ) | ForEach-Object { $_.Groups["field"].Value }
    )
}

function Get-RustTopLevelBody {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$DeclarationPattern,
        [Parameter(Mandatory = $true)] [string]$Label
    )

    $match = [regex]::Match(
        $Source,
        "(?ms)^$DeclarationPattern[^\r\n\{]*\{(?<body>.*?)^\}"
    )
    if (-not $match.Success) {
        throw "missing-rust-declaration:$Label"
    }

    return $match.Groups["body"].Value
}

function Get-TypeScriptAsyncMethodBody {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$MethodName
    )

    $match = [regex]::Match(
        $Source,
        "(?ms)^  async\s+$([regex]::Escape($MethodName))\s*\((?<body>.*?)^  \}"
    )
    if (-not $match.Success) {
        throw "missing-typescript-async-method:$MethodName"
    }

    return $match.Groups["body"].Value
}

function Get-SafeDtoNames {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$Pattern
    )

    return @(
        [regex]::Matches($Source, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Multiline) |
            ForEach-Object { $_.Groups["name"].Value } |
            Sort-Object -Unique
    )
}

function Find-ForbiddenSafeDtoFields {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string[]]$StructNames
    )

    $forbiddenFieldPattern = "(?i)(raw|payload|content|document|chunk|embedding|(?:^|_)(?:input|expected|output)(?:_|$)|^(?:case|cases)$|measurement|provider|model_config|definition)"
    $violations = [System.Collections.Generic.List[string]]::new()

    foreach ($structName in $StructNames) {
        foreach ($fieldName in Get-StructFieldNames -Source $Source -StructName $structName) {
            if ($fieldName -match $forbiddenFieldPattern) {
                $violations.Add("$structName.$fieldName")
            }
        }
    }

    return @($violations)
}

function Test-ExactContractVocabulary {
    param(
        [Parameter(Mandatory = $true)] [string]$Source,
        [Parameter(Mandatory = $true)] [string]$RequiredText
    )

    $normalizedSource = $Source -replace "\s+", " "
    $normalizedRequiredText = $RequiredText -replace "\s+", " "
    return $normalizedSource.IndexOf($normalizedRequiredText, [System.StringComparison]::Ordinal) -ge 0
}

function ConvertFrom-Utf8Base64 {
    param([Parameter(Mandatory = $true)] [string]$Value)

    return [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($Value))
}

if ([string]::IsNullOrWhiteSpace($Root)) {
    $Root = Split-Path -Parent $PSScriptRoot
}

$localStatus = "passed"
$graphStatus = "passed"
$dtoStatus = "passed"
$benchmarkRouteStatus = "passed"
$benchmarkPublicSurfaceStatus = "passed"
$benchmarkLocalSdkStatus = "passed"
$benchmarkDefinitionSchemaStatus = "passed"
$benchmarkDefinitionGraphDiffStatus = "passed"
$benchmarkDefinitionPublicBoundaryStatus = "passed"
$benchmarkDefinitionVocabularyStatus = "passed"
$publicWriteStatus = "unobserved"
$details = [System.Collections.Generic.List[string]]::new()

try {
    $rootItem = Get-SafeItem -Path $Root -Label "repository-root" -Directory
    $rootPath = $rootItem.FullName

    $applicationPath = Join-Path $rootPath "crates/diff-engine/src/application.rs"
    $benchmarkPath = Join-Path $rootPath "crates/evaluation/src/benchmark_workspace.rs"
    $benchmarkDefinitionAuthoringPath = Join-Path $rootPath "crates/storage/src/benchmark_definition_authoring.rs"
    $knowledgePath = Join-Path $rootPath "crates/knowledge/src/lib.rs"
    $replayPath = Join-Path $rootPath "crates/knowledge/src/memory_replay.rs"
    $serverLibPath = Join-Path $rootPath "server/api/src/lib.rs"
    $serverRoutesPath = Join-Path $rootPath "server/api/src/routes.rs"
    $openApiPath = Join-Path $rootPath "docs/api/openapi.json"
    $localSdkPackagePath = Join-Path $rootPath "packages/local-sdk/package.json"
    $localSdkIndexPath = Join-Path $rootPath "packages/local-sdk/src/index.ts"
    $localSdkBenchmarkPath = Join-Path $rootPath "packages/local-sdk/src/benchmark-workspace.ts"
    $localSdkClientPath = Join-Path $rootPath "packages/local-sdk/src/client.ts"
    $publicSdkPackagePath = Join-Path $rootPath "packages/ts-sdk/package.json"
    $publicSdkSourcePath = Join-Path $rootPath "packages/ts-sdk/src"
    $architecturePath = Join-Path $rootPath "ARCHITECTURE.md"
    $benchmarkDefinitionIntegrationPath = Join-Path $rootPath "docs/api/wave-1-integration-contracts.md"

    $applicationSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $applicationPath -Label "graph-application").FullName)
    $benchmarkSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $benchmarkPath -Label "benchmark-safe-dtos").FullName)
    $benchmarkDefinitionAuthoringSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $benchmarkDefinitionAuthoringPath -Label "benchmark-definition-authoring").FullName)
    $knowledgeSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $knowledgePath -Label "knowledge-safe-dtos").FullName)
    $replaySource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $replayPath -Label "replay-safe-dtos").FullName)
    $serverLibSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $serverLibPath -Label "benchmark-route-catalog").FullName)
    $serverRoutesSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $serverRoutesPath -Label "benchmark-route-dtos").FullName)
    $openApiSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $openApiPath -Label "public-openapi").FullName)
    $localSdkPackageSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $localSdkPackagePath -Label "local-sdk-package").FullName)
    $localSdkIndexSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $localSdkIndexPath -Label "local-sdk-index").FullName)
    $localSdkBenchmarkSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $localSdkBenchmarkPath -Label "local-sdk-benchmark-parser").FullName)
    $localSdkClientSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $localSdkClientPath -Label "local-sdk-client").FullName)
    $publicSdkPackageSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $publicSdkPackagePath -Label "public-sdk-package").FullName)
    $publicSdkSourceFiles = @(Get-SafeSourceFiles -Path $publicSdkSourcePath -Label "public-sdk-source")
    $architectureSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $architecturePath -Label "architecture-vocabulary").FullName)
    $benchmarkDefinitionIntegrationSource = [System.IO.File]::ReadAllText((Get-SafeItem -Path $benchmarkDefinitionIntegrationPath -Label "benchmark-definition-integration-vocabulary").FullName)
}
catch {
    $localStatus = "blocked"
    $graphStatus = "blocked"
    $dtoStatus = "blocked"
    $benchmarkRouteStatus = "blocked"
    $benchmarkPublicSurfaceStatus = "blocked"
    $benchmarkLocalSdkStatus = "blocked"
    $benchmarkDefinitionSchemaStatus = "blocked"
    $benchmarkDefinitionGraphDiffStatus = "blocked"
    $benchmarkDefinitionPublicBoundaryStatus = "blocked"
    $benchmarkDefinitionVocabularyStatus = "blocked"
    $details.Add($_.Exception.Message)
    $applicationSource = ""
    $benchmarkSource = ""
    $benchmarkDefinitionAuthoringSource = ""
    $knowledgeSource = ""
    $replaySource = ""
    $serverLibSource = ""
    $serverRoutesSource = ""
    $openApiSource = ""
    $localSdkPackageSource = ""
    $localSdkIndexSource = ""
    $localSdkBenchmarkSource = ""
    $localSdkClientSource = ""
    $publicSdkPackageSource = ""
    $publicSdkSourceFiles = @()
    $architectureSource = ""
    $benchmarkDefinitionIntegrationSource = ""
}

$graphDiffCount = @([regex]::Matches($applicationSource, "GraphDiff::between\s*\(")).Count
if ($localStatus -eq "passed" -and $graphDiffCount -ne 1) {
    $graphStatus = "blocked"
    $details.Add("graph-diff-calculator-count:$graphDiffCount")
}

$benchmarkDefinitionGraphDiffCount = @([regex]::Matches($benchmarkDefinitionAuthoringSource, "GraphDiff::between\s*\(")).Count
if ($localStatus -eq "passed" -and $benchmarkDefinitionGraphDiffCount -ne 0) {
    $benchmarkDefinitionGraphDiffStatus = "blocked"
    $details.Add("benchmark-definition-graph-diff-calculator-count:$benchmarkDefinitionGraphDiffCount")
}

if ($localStatus -eq "passed") {
    $schemaVersionPattern = "(?m)^\s*pub\s+const\s+BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION\s*:\s*u16\s*=\s*1\s*;"
    if (@([regex]::Matches($benchmarkDefinitionAuthoringSource, $schemaVersionPattern)).Count -ne 1) {
        $benchmarkDefinitionSchemaStatus = "blocked"
        $details.Add("benchmark-definition-schema-version")
    }
    if ($benchmarkDefinitionAuthoringSource -notmatch "if\s+schema_version\s*!=\s*BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION") {
        $benchmarkDefinitionSchemaStatus = "blocked"
        $details.Add("benchmark-definition-schema-validation")
    }
}

if ($localStatus -eq "passed") {
    try {
        $benchmarkDtos = Get-SafeDtoNames -Source $benchmarkSource -Pattern "^\s*pub\s+struct\s+(?<name>[A-Za-z_][A-Za-z0-9_]*)\s*\{"
        $knowledgeDtos = @("KnowledgeLocalCitationProjectionV1")
        $replayDtos = @("KnowledgeMemoryReplayProjection")
        $routeDtos = @(
            "LocalBenchmarkWorkspaceResponse",
            "LocalBenchmarkDecisionPairWitnessResponse",
            "LocalBenchmarkDecisionScopeResponse"
        )
        if ($benchmarkDtos.Count -eq 0) {
            throw "missing-safe-dto-declarations:benchmark"
        }

        $responseBody = Get-StructBody -Source $serverRoutesSource -StructName "LocalBenchmarkWorkspaceResponse"
        $expectedResponseFields = @(
            "schema_version",
            "project_id",
            "context_id",
            "revised",
            "baseline",
            "decision_pair_witness",
            "projection"
        ) | Sort-Object
        $actualResponseFields = @(
            Get-StructFieldNames -Source $serverRoutesSource -StructName "LocalBenchmarkWorkspaceResponse"
        ) | Sort-Object
        $responseFieldDiff = @(
            Compare-Object -ReferenceObject $expectedResponseFields -DifferenceObject $actualResponseFields
        )
        if ($responseFieldDiff.Count -gt 0) {
            $dtoStatus = "blocked"
            $details.Add("invalid-safe-dto-fields:LocalBenchmarkWorkspaceResponse")
        }
        if ($responseBody -notmatch "(?m)^\s*projection\s*:\s*BenchmarkWorkspaceProjectionV1\s*,") {
            $dtoStatus = "blocked"
            $details.Add("invalid-safe-dto-projection:LocalBenchmarkWorkspaceResponse")
        }
        if ($responseBody -notmatch "(?m)^\s*decision_pair_witness\s*:\s*Option<LocalBenchmarkDecisionPairWitnessResponse>\s*,") {
            $dtoStatus = "blocked"
            $details.Add("invalid-safe-dto-witness:LocalBenchmarkWorkspaceResponse")
        }

        $witnessBody = Get-StructBody -Source $serverRoutesSource -StructName "LocalBenchmarkDecisionPairWitnessResponse"
        $expectedWitnessFields = @(
            "schema_version",
            "project_id",
            "context_id",
            "baseline",
            "revised"
        ) | Sort-Object
        $actualWitnessFields = @(
            Get-StructFieldNames -Source $serverRoutesSource -StructName "LocalBenchmarkDecisionPairWitnessResponse"
        ) | Sort-Object
        $witnessFieldDiff = @(
            Compare-Object -ReferenceObject $expectedWitnessFields -DifferenceObject $actualWitnessFields
        )
        if ($witnessFieldDiff.Count -gt 0) {
            $dtoStatus = "blocked"
            $details.Add("invalid-safe-dto-fields:LocalBenchmarkDecisionPairWitnessResponse")
        }

        $dtoViolations = @(
            Find-ForbiddenSafeDtoFields -Source $benchmarkSource -StructNames $benchmarkDtos
            Find-ForbiddenSafeDtoFields -Source $knowledgeSource -StructNames $knowledgeDtos
            Find-ForbiddenSafeDtoFields -Source $replaySource -StructNames $replayDtos
            Find-ForbiddenSafeDtoFields -Source $serverRoutesSource -StructNames $routeDtos
        )
        if ($dtoViolations.Count -gt 0) {
            $dtoStatus = "blocked"
            foreach ($violation in $dtoViolations) {
                $details.Add("raw-safe-dto-field:$violation")
            }
        }
    }
    catch {
        $dtoStatus = "blocked"
        $details.Add($_.Exception.Message)
    }
}

if ($localStatus -eq "passed") {
    try {
        $openApiDocument = $openApiSource | ConvertFrom-Json
        if ($null -eq $openApiDocument.paths) {
            throw "invalid-public-openapi:missing-paths"
        }
        foreach ($path in @($openApiDocument.paths.PSObject.Properties.Name)) {
            if ($path -match "(?i)(?:^|/)benchmark[-_]?definitions?(?:/|$)") {
                throw "benchmark-definition-public-openapi:$path"
            }
        }

        $publicSdkManifest = $publicSdkPackageSource | ConvertTo-Json -Depth 20 -Compress
        if ($publicSdkManifest -match "(?i)benchmark[-_]?definition(?:[-_]?binding)?") {
            throw "benchmark-definition-public-sdk:packages/ts-sdk/package.json"
        }
        foreach ($sourceFile in $publicSdkSourceFiles) {
            $source = [System.IO.File]::ReadAllText($sourceFile.FullName)
            if ($source -match "(?i)benchmark[-_]?definition(?:[-_]?binding)?") {
                $relativePath = $sourceFile.FullName.Substring($rootPath.Length + 1).Replace("\", "/")
                throw "benchmark-definition-public-sdk:$relativePath"
            }
        }
    }
    catch {
        $benchmarkDefinitionPublicBoundaryStatus = "blocked"
        $details.Add($_.Exception.Message)
    }
}

if ($localStatus -eq "passed") {
    $architectureVocabulary = @(
        @("architecture-heading", "### Private Benchmark Definition Authoring / $(ConvertFrom-Utf8Base64 '56eB5pyJIEJlbmNobWFyayDlrprkuYnliJvkvZw=')"),
        @("architecture-public-boundary-en", "This is a private core/storage contract; public REST/OpenAPI/public SDK writes, Web mutation, provider calls, Context commit mutation, and release readiness remain out of scope. GraphDiff::between remains the sole graph-diff calculator."),
        @("architecture-public-boundary-zh", (ConvertFrom-Utf8Base64 '5aKe6YeP5pivIHByaXZhdGUgY29yZS9zdG9yYWdlIGNvbnRyYWN077ybcHVibGljIFJFU1QvT3BlbkFQSS9wdWJsaWMgU0RLIHdyaXRl44CBV2ViIG11dGF0aW9u44CBIHByb3ZpZGVyIGNhbGzjgIFDb250ZXh0IGNvbW1pdCBtdXRhdGlvbiDkuI4gcmVsZWFzZSByZWFkaW5lc3Mg5Z2H5LiN5Zyo6IyD5Zu05YaF44CCR3JhcGhEaWZmOjpiZXR3ZWVuIOS7jeaYr+WUr+S4gCBncmFwaC1kaWZmIGNhbGN1bGF0b3LjgII='))
    )
    foreach ($requirement in $architectureVocabulary) {
        if (-not (Test-ExactContractVocabulary -Source $architectureSource -RequiredText $requirement[1])) {
            $benchmarkDefinitionVocabularyStatus = "blocked"
            $details.Add("benchmark-definition-vocabulary:$($requirement[0])")
        }
    }

    $integrationVocabulary = @(
        @("integration-heading", "### Private Benchmark Definition Binding / $(ConvertFrom-Utf8Base64 '56eB5pyJIEJlbmNobWFyayDlrprkuYnnu5Hlrpo=')"),
        @("integration-public-boundary-en", "The binding route is not yet public or transport-complete; no public OpenAPI/public SDK write is added."),
        @("integration-public-boundary-zh", (ConvertFrom-Utf8Base64 'YmluZGluZyByb3V0ZSDov5jmnKrmiJDkuLogcHVibGljIOaIluWujOaVtCB0cmFuc3BvcnTvvJvmsqHmnInmlrDlop4gcHVibGljIE9wZW5BUEkvcHVibGljIFNESyB3cml0ZeOAgg=='))
    )
    foreach ($requirement in $integrationVocabulary) {
        if (-not (Test-ExactContractVocabulary -Source $benchmarkDefinitionIntegrationSource -RequiredText $requirement[1])) {
            $benchmarkDefinitionVocabularyStatus = "blocked"
            $details.Add("benchmark-definition-vocabulary:$($requirement[0])")
        }
    }
}

if ($localStatus -eq "passed") {
    try {
        $routeImpl = Get-RustTopLevelBody `
            -Source $serverLibSource `
            -DeclarationPattern "impl\s+ProtectedBenchmarkWorkspaceGetRoute" `
            -Label "protected-benchmark-workspace-route"
        $expectedRoutePaths = @(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}",
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}/workspace"
        )
        $routePathMatches = @(
            [regex]::Matches(
                $routeImpl,
                '"(?<path>/api/v1/[^"\r\n]*)"'
            )
        )
        if ($routePathMatches.Count -ne $expectedRoutePaths.Count) {
            throw "benchmark-workspace-route-count:$($routePathMatches.Count)"
        }
        $actualRoutePaths = @($routePathMatches | ForEach-Object { $_.Groups["path"].Value } | Sort-Object)
        $expectedRoutePaths = @($expectedRoutePaths | Sort-Object)
        foreach ($routePath in $actualRoutePaths) {
            if (-not $routePath.StartsWith("/api/v1/local/", [System.StringComparison]::Ordinal)) {
                throw "benchmark-workspace-route-not-local"
            }
        }
        if (($actualRoutePaths -join "`n") -cne ($expectedRoutePaths -join "`n")) {
            throw "benchmark-workspace-route-catalog"
        }

        $routeMethodMatches = @(
            [regex]::Matches(
                $routeImpl,
                "axum::routing::(?<method>[A-Za-z_][A-Za-z0-9_]*)\s*\(\s*routes::(?<handler>[A-Za-z_][A-Za-z0-9_]*)\s*\)"
            )
        )
        if ($routeMethodMatches.Count -ne $expectedRoutePaths.Count) {
            throw "benchmark-workspace-route-method-count:$($routeMethodMatches.Count)"
        }
        foreach ($routeMethodMatch in $routeMethodMatches) {
            $routeMethod = $routeMethodMatch.Groups["method"].Value
            if ($routeMethod -cne "get") {
                throw "benchmark-workspace-route-method:$routeMethod"
            }
        }

        $expectedHandlers = @(
            "local_benchmark_workspace",
            "local_benchmark_workspace_by_decision"
        )
        $handlerReferenceCount = @(
            [regex]::Matches($routeImpl, "routes::(?<handler>[A-Za-z_][A-Za-z0-9_]*)\b")
        ).Count
        if ($handlerReferenceCount -ne $expectedHandlers.Count) {
            throw "benchmark-workspace-handler-registration-count:$handlerReferenceCount"
        }
        foreach ($handler in $expectedHandlers) {
            $handlerCount = @(
                [regex]::Matches($routeImpl, "routes::$([regex]::Escape($handler))\b")
            ).Count
            if ($handlerCount -ne 1) {
                throw "benchmark-workspace-handler-registration-count:$handler`:$handlerCount"
            }
        }
        $actualRouteRegistrations = @(
            $routeMethodMatches | ForEach-Object {
                "$($_.Groups["method"].Value):$($_.Groups["handler"].Value)"
            } | Sort-Object
        )
        $expectedRouteRegistrations = @(
            "get:local_benchmark_workspace",
            "get:local_benchmark_workspace_by_decision"
        ) | Sort-Object
        if (($actualRouteRegistrations -join "`n") -cne ($expectedRouteRegistrations -join "`n")) {
            throw "benchmark-workspace-route-catalog"
        }

        $catalogMatch = [regex]::Match(
            $serverLibSource,
            "(?ms)^\s*const\s+PROTECTED_BENCHMARK_WORKSPACE_GET_ROUTES\s*:[^=]+=(?<value>.*?);"
        )
        if (
            -not $catalogMatch.Success -or
            @([regex]::Matches($catalogMatch.Groups["value"].Value, "ProtectedBenchmarkWorkspaceGetRoute::(?:Workspace|DecisionWorkspace)\b")).Count -ne $expectedRoutePaths.Count -or
            @([regex]::Matches($catalogMatch.Groups["value"].Value, "ProtectedBenchmarkWorkspaceGetRoute::Workspace\b")).Count -ne 1 -or
            @([regex]::Matches($catalogMatch.Groups["value"].Value, "ProtectedBenchmarkWorkspaceGetRoute::DecisionWorkspace\b")).Count -ne 1
        ) {
            throw "benchmark-workspace-protected-catalog"
        }

        $protectedBuilder = Get-RustTopLevelBody `
            -Source $serverLibSource `
            -DeclarationPattern "pub\s+fn\s+build_protected_router_with_state" `
            -Label "protected-router-builder"
        foreach ($requiredSymbol in @(
            "PROTECTED_BENCHMARK_WORKSPACE_GET_ROUTES",
            "authenticate_benchmark_workspace_read_request",
            "merge(protected_benchmark_workspace_read_router)"
        )) {
            if ($protectedBuilder.IndexOf($requiredSymbol, [System.StringComparison]::Ordinal) -lt 0) {
                throw "benchmark-workspace-protected-wiring:$requiredSymbol"
            }
        }
    }
    catch {
        $benchmarkRouteStatus = "blocked"
        $details.Add($_.Exception.Message)
    }
}

if ($localStatus -eq "passed") {
    try {
        $publicRouteImpl = Get-RustTopLevelBody `
            -Source $serverLibSource `
            -DeclarationPattern "impl\s+PublicGetRoute" `
            -Label "public-get-route-catalog"
        $publicRouterBody = Get-RustTopLevelBody `
            -Source $serverLibSource `
            -DeclarationPattern "fn\s+public_router\s*\(\s*\)" `
            -Label "public-router"
        $publicRouterMarker = "(?i)(?:benchmark[_-]?workspace|local_benchmark_workspace|PROTECTED_BENCHMARK_WORKSPACE)"
        if ($publicRouteImpl -match $publicRouterMarker -or $publicRouterBody -match $publicRouterMarker) {
            throw "benchmark-workspace-public-router"
        }

        $openApiDocument = $openApiSource | ConvertFrom-Json
        if ($null -eq $openApiDocument.paths) {
            throw "invalid-public-openapi:missing-paths"
        }
        foreach ($path in @($openApiDocument.paths.PSObject.Properties.Name)) {
            if ($path -match "(?i)(?:^|/)benchmark-workspace(?:/|$)") {
                throw "benchmark-workspace-openapi-path:$path"
            }
        }

        $publicSdkPackage = $publicSdkPackageSource | ConvertFrom-Json
        if ($publicSdkPackage.name -cne "@contextlab/ts-sdk") {
            throw "invalid-public-sdk-package-name"
        }
        $publicSdkManifest = $publicSdkPackage | ConvertTo-Json -Depth 20 -Compress
        if ($publicSdkManifest -match "(?i)benchmark[-_]?workspace") {
            throw "benchmark-workspace-public-sdk:packages/ts-sdk/package.json"
        }
        foreach ($sourceFile in $publicSdkSourceFiles) {
            $source = [System.IO.File]::ReadAllText($sourceFile.FullName)
            if ($source -match "(?i)benchmark[-_]?workspace") {
                $relativePath = $sourceFile.FullName.Substring($rootPath.Length + 1).Replace("\", "/")
                throw "benchmark-workspace-public-sdk:$relativePath"
            }
        }
    }
    catch {
        $benchmarkPublicSurfaceStatus = "blocked"
        $details.Add($_.Exception.Message)
    }
}

if ($localStatus -eq "passed") {
    try {
        $localSdkPackage = $localSdkPackageSource | ConvertFrom-Json
        if ($localSdkPackage.name -cne "@contextlab/local-sdk") {
            throw "invalid-local-sdk-package-name"
        }

        $parserDefinitionCount = @(
            [regex]::Matches(
                $localSdkBenchmarkSource,
                "(?m)^\s*export\s+function\s+parseLocalBenchmarkWorkspace\s*\("
            )
        ).Count
        if ($parserDefinitionCount -ne 1) {
            throw "benchmark-workspace-parser-definition-count:$parserDefinitionCount"
        }
        if (
            $localSdkIndexSource -notmatch '(?ms)export\s*\{[^\}]*\bparseLocalBenchmarkWorkspace\b[^\}]*\}\s*from\s*["'']\./benchmark-workspace["'']' -or
            $localSdkIndexSource -notmatch '(?ms)export\s*\{[^\}]*\bContextLabLocalClient\b[^\}]*\}\s*from\s*["'']\./client["'']'
        ) {
            throw "benchmark-workspace-local-sdk-exports"
        }

        $clientMethodCount = @(
            [regex]::Matches($localSdkClientSource, "(?m)^  async\s+getBenchmarkWorkspace\s*\(")
        ).Count
        if ($clientMethodCount -ne 1) {
            throw "benchmark-workspace-client-method-count:$clientMethodCount"
        }
        $clientMethod = Get-TypeScriptAsyncMethodBody `
            -Source $localSdkClientSource `
            -MethodName "getBenchmarkWorkspace"
        $clientParserCount = @(
            [regex]::Matches($clientMethod, "\bparseLocalBenchmarkWorkspace\s*\(")
        ).Count
        if ($clientParserCount -ne 1) {
            throw "benchmark-workspace-client-parser-count:$clientParserCount"
        }
        if (
            $clientMethod -notmatch "(?i)/api/v1/local/" -or
            $clientMethod -notmatch "(?i)/benchmark-workspace/"
        ) {
            throw "benchmark-workspace-client-route-not-local"
        }
        if ($clientMethod -match "(?i)\bmethod\s*:") {
            throw "benchmark-workspace-client-not-get-only"
        }
    }
    catch {
        $benchmarkLocalSdkStatus = "blocked"
        $details.Add($_.Exception.Message)
    }
}

if (-not [string]::IsNullOrWhiteSpace($DiffPath)) {
    try {
        if ([System.IO.Path]::GetFileName($DiffPath) -match "(?i)^\.env") {
            throw "refusing-environment-file-input"
        }

        $diffItem = Get-SafeItem -Path $DiffPath -Label "diff-input"
        $currentPath = ""
        $sawDiffPath = $false
        $publicWriteMatches = [System.Collections.Generic.List[string]]::new()
        foreach ($line in [System.IO.File]::ReadLines($diffItem.FullName)) {
            if ($line -match '^\+\+\+ b/(?<path>.+)$') {
                $currentPath = $Matches.path.Replace("\", "/")
                $sawDiffPath = $true
                continue
            }
            if (-not $line.StartsWith("+") -or $line.StartsWith("+++")) {
                continue
            }

            $addedLine = $line.Substring(1)
            $isApiWrite = $currentPath -like "server/api/*" -and $addedLine -match "(?i)(?:routing::)?\b(?:post|put|patch|delete)\s*\("
            $isOpenApiWrite = ($currentPath -like "docs/api/*" -or $currentPath -match "(?i)openapi.*\.(json|ya?ml)$") -and $addedLine -match '(?i)"(?:post|put|patch|delete)"\s*:'
            $isSdkWrite = $currentPath -like "packages/*" -and $currentPath -match "/src/" -and $addedLine -match '(?i)(?:\basync\s+(?:create|update|delete)[A-Za-z0-9_]*\s*\(|\bmethod\s*:\s*(?:"(?:POST|PUT|PATCH|DELETE)"|HttpMethod\.(?:POST|PUT|PATCH|DELETE))|\.(?:post|put|patch|delete)\s*\()'
            if ($isApiWrite -or $isOpenApiWrite -or $isSdkWrite) {
                $publicWriteMatches.Add($currentPath)
            }
        }

        if (-not $sawDiffPath) {
            throw "malformed-unified-diff-input"
        }
        if ($publicWriteMatches.Count -gt 0) {
            $publicWriteStatus = "blocked"
            foreach ($path in ($publicWriteMatches | Sort-Object -Unique)) {
                $details.Add("public-write-addition:$path")
            }
        }
        else {
            $publicWriteStatus = "passed"
        }
    }
    catch {
        $publicWriteStatus = "blocked"
        $details.Add($_.Exception.Message)
    }
}

foreach ($detail in $details) {
    Write-Output "local_contract_detail=blocked reason=$detail"
}

Write-Output "local_contract_source=$localStatus"
Write-Output "graph_diff_application=$graphStatus count=$graphDiffCount"
Write-Output "safe_local_dto_fields=$dtoStatus"
Write-Output "benchmark_workspace_route=$benchmarkRouteStatus"
Write-Output "benchmark_workspace_public_surface=$benchmarkPublicSurfaceStatus"
Write-Output "benchmark_workspace_local_sdk=$benchmarkLocalSdkStatus"
Write-Output "benchmark_definition_schema=$benchmarkDefinitionSchemaStatus"
Write-Output "benchmark_definition_graph_diff=$benchmarkDefinitionGraphDiffStatus count=$benchmarkDefinitionGraphDiffCount"
Write-Output "benchmark_definition_public_boundary=$benchmarkDefinitionPublicBoundaryStatus"
Write-Output "benchmark_definition_vocabulary=$benchmarkDefinitionVocabularyStatus"
if ($publicWriteStatus -eq "unobserved") {
    Write-Output "public_write_additions=unobserved reason=no-unified-diff-input"
}
else {
    Write-Output "public_write_additions=$publicWriteStatus"
}
Write-Output "git_evidence=unobserved reason=not-read-by-static-verifier"
Write-Output "browser_evidence=unobserved reason=not-run-by-static-verifier"
Write-Output "production_evidence=unobserved reason=not-run-by-static-verifier"

if (
    $localStatus -eq "blocked" -or
    $graphStatus -eq "blocked" -or
    $dtoStatus -eq "blocked" -or
    $benchmarkRouteStatus -eq "blocked" -or
    $benchmarkPublicSurfaceStatus -eq "blocked" -or
    $benchmarkLocalSdkStatus -eq "blocked" -or
    $benchmarkDefinitionSchemaStatus -eq "blocked" -or
    $benchmarkDefinitionGraphDiffStatus -eq "blocked" -or
    $benchmarkDefinitionPublicBoundaryStatus -eq "blocked" -or
    $benchmarkDefinitionVocabularyStatus -eq "blocked" -or
    $publicWriteStatus -eq "blocked"
) {
    Write-Output "overall=blocked"
    exit 1
}
if ($publicWriteStatus -eq "unobserved") {
    Write-Output "overall=unobserved"
    exit 0
}

Write-Output "overall=passed"
