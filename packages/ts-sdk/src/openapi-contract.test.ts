import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { ContextLabClient } from "./client";

type OpenApiParameter = {
  $ref?: string;
  name?: string;
  in?: string;
  required?: boolean;
};

type OpenApiSchema = {
  $ref?: string;
  type?: string;
  items?: OpenApiSchema;
  properties?: Record<string, OpenApiSchema>;
  required?: string[];
  additionalProperties?: boolean | OpenApiSchema;
};

type OpenApiResponse = {
  content?: {
    "application/json"?: {
      schema?: OpenApiSchema;
    };
  };
};

type OpenApiOperation = {
  operationId?: string;
  parameters?: OpenApiParameter[];
  requestBody?: {
    required?: boolean;
    content?: {
      "application/json"?: {
        schema?: OpenApiSchema;
      };
    };
  };
  responses?: Record<string, OpenApiResponse>;
};

type OpenApiPathItem = {
  get?: OpenApiOperation;
  post?: OpenApiOperation;
};

type OpenApiDocument = {
  openapi?: string;
  paths?: Record<string, OpenApiPathItem>;
  components?: {
    parameters?: Record<string, OpenApiParameter>;
    schemas?: Record<string, OpenApiSchema>;
  };
};

type GetOperationExpectation = {
  methodName: keyof ContextLabClient;
  path: string;
  pathParameters: string[];
  queryParameters: string[];
};

const getOperations: GetOperationExpectation[] = [
  {
    methodName: "healthz",
    path: "/healthz",
    pathParameters: [],
    queryParameters: []
  },
  {
    methodName: "getMeta",
    path: "/api/v1/meta",
    pathParameters: [],
    queryParameters: []
  },
  {
    methodName: "getOpenApiDocument",
    path: "/api/v1/openapi.json",
    pathParameters: [],
    queryParameters: []
  },
  {
    methodName: "listProviders",
    path: "/api/v1/providers",
    pathParameters: [],
    queryParameters: []
  },
  {
    methodName: "listWorkspaces",
    path: "/api/v1/workspaces",
    pathParameters: [],
    queryParameters: ["page", "per_page", "search", "sort"]
  },
  {
    methodName: "listProjects",
    path: "/api/v1/workspaces/{workspace_id}/projects",
    pathParameters: ["workspace_id"],
    queryParameters: ["page", "per_page", "search", "sort"]
  },
  {
    methodName: "listExperiments",
    path: "/api/v1/projects/{project_id}/experiments",
    pathParameters: ["project_id"],
    queryParameters: ["page", "per_page", "search", "sort"]
  },
  {
    methodName: "listContexts",
    path: "/api/v1/projects/{project_id}/contexts",
    pathParameters: ["project_id"],
    queryParameters: ["page", "per_page", "search", "sort", "experiment_id"]
  },
  {
    methodName: "listCommits",
    path: "/api/v1/contexts/{context_id}/commits",
    pathParameters: ["context_id"],
    queryParameters: ["page", "per_page", "search", "sort", "branch_name"]
  },
  {
    methodName: "getCommit",
    path: "/api/v1/contexts/{context_id}/commits/{commit_id}",
    pathParameters: ["context_id", "commit_id"],
    queryParameters: []
  },
  {
    methodName: "listComponents",
    path: "/api/v1/contexts/{context_id}/components",
    pathParameters: ["context_id"],
    queryParameters: ["page", "per_page", "search", "sort", "kind"]
  },
  {
    methodName: "getComponent",
    path: "/api/v1/contexts/{context_id}/components/{component_id}",
    pathParameters: ["context_id", "component_id"],
    queryParameters: []
  },
  {
    methodName: "listEvaluationRuns",
    path: "/api/v1/contexts/{context_id}/evaluation-runs",
    pathParameters: ["context_id"],
    queryParameters: ["page", "per_page", "search", "sort", "suite_name", "model_version"]
  },
  {
    methodName: "getEvaluationScorecard",
    path: "/api/v1/contexts/{context_id}/evaluation-scorecard",
    pathParameters: ["context_id"],
    queryParameters: ["search", "suite_name", "model_version"]
  },
  {
    methodName: "getEvaluationRun",
    path: "/api/v1/contexts/{context_id}/evaluation-runs/{run_id}",
    pathParameters: ["context_id", "run_id"],
    queryParameters: []
  },
  {
    methodName: "getContextGraphPreview",
    path: "/api/v1/context-graph/preview",
    pathParameters: [],
    queryParameters: []
  },
  {
    methodName: "getWorkspaceContextGraph",
    path: "/api/v1/workspaces/{workspace_id}/context-graph",
    pathParameters: ["workspace_id"],
    queryParameters: []
  }
];

test("OpenAPI document is versioned as OpenAPI 3.x", () => {
  const document = readOpenApiDocument();

  assert.match(document.openapi ?? "", /^3\./);
});

test("OpenAPI GET operations cover every SDK client method", () => {
  const document = readOpenApiDocument();
  const expectedOperationIds = getOperations.map((operation) => operation.methodName).sort();
  const actualOperationIds = Object.values(document.paths ?? {})
    .flatMap((pathItem) => (pathItem.get?.operationId ? [pathItem.get.operationId] : []))
    .sort();

  assert.deepEqual(actualOperationIds, expectedOperationIds);

  for (const expectation of getOperations) {
    assert.equal(typeof ContextLabClient.prototype[expectation.methodName], "function");

    const operation = document.paths?.[expectation.path]?.get;
    assert.ok(operation, `missing GET ${expectation.path}`);
    assert.equal(operation.operationId, expectation.methodName);
  }
});

test("OpenAPI GET operations expose SDK path and query parameters", () => {
  const document = readOpenApiDocument();

  for (const expectation of getOperations) {
    const operation = document.paths?.[expectation.path]?.get;
    assert.ok(operation, `missing GET ${expectation.path}`);
    const parameters = resolveParameters(document, operation.parameters ?? []);

    assert.deepEqual(parameterNames(parameters, "path"), [...expectation.pathParameters].sort());
    assert.deepEqual(parameterNames(parameters, "query"), [...expectation.queryParameters].sort());
  }
});

test("OpenAPI exposes the graph diff POST SDK contract", () => {
  const document = readOpenApiDocument();
  const operation = document.paths?.["/api/v1/graph-diffs"]?.post;

  assert.ok(operation, "missing POST graph diff operation");
  assert.equal(operation.operationId, "compareGraphs");
  assert.equal(
    typeof (ContextLabClient.prototype as unknown as Record<string, unknown>).compareGraphs,
    "function"
  );
  assert.equal(operation.requestBody?.required, true);
  assert.equal(
    operation.requestBody?.content?.["application/json"]?.schema?.$ref,
    "#/components/schemas/GraphDiffRequest"
  );
  assert.equal(
    operation.responses?.["200"]?.content?.["application/json"]?.schema?.$ref,
    "#/components/schemas/GraphDiffResponse"
  );

  const response = document.components?.schemas?.GraphDiffResponse;
  assert.ok(response, "missing GraphDiffResponse schema");
  assert.deepEqual([...(response.required ?? [])].sort(), [
    "added_edges",
    "added_nodes",
    "modified_nodes",
    "removed_edges",
    "removed_nodes"
  ]);
});

test("OpenAPI and public SDK omit protected Context lifecycle writes while preserving graph diff POST", () => {
  const document = readOpenApiDocument();
  const clientPrototype = ContextLabClient.prototype as unknown as Record<string, unknown>;

  assert.equal(
    document.paths?.["/api/v1/contexts/{context_id}/commits"]?.post,
    undefined,
    "POST context commit creation must remain absent from the public contract"
  );
  assert.equal(
    clientPrototype.createContextCommit,
    undefined,
    "the public SDK must not expose context commit creation"
  );
  assert.equal(
    document.paths?.["/api/v1/local/contexts/{context_id}/component-lifecycle-commits"],
    undefined,
    "local lifecycle commit transport must remain absent from the public contract"
  );
  assert.equal(
    document.paths?.["/api/v1/local/contexts/{context_id}/commits/{commit_id}/lifecycle-state"],
    undefined,
    "local lifecycle state transport must remain absent from the public contract"
  );
  assert.equal(
    document.paths?.[
      "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions"
    ],
    undefined,
    "local benchmark decision collection transport must remain absent from the public contract"
  );
  assert.equal(
    document.paths?.[
      "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}"
    ],
    undefined,
    "local benchmark decision transport must remain absent from the public contract"
  );
  assert.equal(
    clientPrototype.createLocalComponentLifecycleCommit,
    undefined,
    "the public SDK must not expose local lifecycle writes"
  );
  assert.equal(
    clientPrototype.getLocalContextLifecycleState,
    undefined,
    "the public SDK must not expose local lifecycle state reads"
  );
  assert.equal(
    clientPrototype.listBenchmarkDecisions,
    undefined,
    "the public SDK must not expose local benchmark decision discovery"
  );
  assert.equal(
    clientPrototype.getLocalBenchmarkDecision,
    undefined,
    "the public SDK must not expose local benchmark decision reads"
  );

  const graphDiffOperation = document.paths?.["/api/v1/graph-diffs"]?.post;
  assert.ok(graphDiffOperation, "missing public POST graph diff operation");
  assert.equal(graphDiffOperation.operationId, "compareGraphs");
});

test("OpenAPI getCommit response matches the SDK CommitDetail contract", () => {
  const document = readOpenApiDocument();
  const operation = document.paths?.["/api/v1/contexts/{context_id}/commits/{commit_id}"]?.get;
  assert.ok(operation, "missing GET commit detail operation");
  assert.equal(operation.operationId, "getCommit");

  const responseSchema = operation.responses?.["200"]?.content?.["application/json"]?.schema;
  assert.equal(responseSchema?.$ref, "#/components/schemas/CommitDetail");

  const commitDetail = document.components?.schemas?.CommitDetail;
  assert.ok(commitDetail, "missing CommitDetail schema");
  assert.equal(commitDetail.type, "object");
  assert.deepEqual([...(commitDetail.required ?? [])].sort(), [
    "authored_at",
    "branch_name",
    "change_count",
    "changes",
    "context_id",
    "created_at",
    "id",
    "message",
    "parent_commit_ids"
  ]);
  assert.equal(commitDetail.properties?.changes?.type, "array");
  assert.equal(commitDetail.properties?.changes?.items?.type, "object");
  assert.equal(commitDetail.properties?.changes?.items?.additionalProperties, true);
});

test("OpenAPI and public SDK omit the retired version-backed commit graph diff read", () => {
  const document = readOpenApiDocument();
  assert.equal(
    document.paths?.["/api/v1/contexts/{context_id}/graph-diff"],
    undefined,
    "the retired public commit graph diff route must be absent"
  );
  assert.equal(
    (ContextLabClient.prototype as unknown as Record<string, unknown>).getCommitGraphDiff,
    undefined,
    "the public SDK must not expose the retired commit graph diff read"
  );
  assert.equal(
    document.components?.schemas?.CommitGraphDiffResponse,
    undefined,
    "the retired public response schema must be absent"
  );
  assert.equal(
    document.components?.schemas?.CommitGraphSnapshotReference,
    undefined,
    "the retired public snapshot reference schema must be absent"
  );
});

test("OpenAPI allowlists the sole public POST graph-diff computation", () => {
  const document = readOpenApiDocument();
  const publicPostOperations = Object.entries(document.paths ?? {})
    .flatMap(([path, pathItem]) =>
      pathItem.post?.operationId ? [{ path, operationId: pathItem.post.operationId }] : []
    )
    .sort((left, right) => left.path.localeCompare(right.path));

  assert.deepEqual(publicPostOperations, [
    {
      path: "/api/v1/graph-diffs",
      operationId: "compareGraphs"
    }
  ]);
});

test("OpenAPI getEvaluationScorecard response matches the SDK EvaluationScorecard contract", () => {
  const document = readOpenApiDocument();
  const operation =
    document.paths?.["/api/v1/contexts/{context_id}/evaluation-scorecard"]?.get;
  assert.ok(operation, "missing GET evaluation scorecard operation");
  assert.equal(operation.operationId, "getEvaluationScorecard");

  const responseSchema = operation.responses?.["200"]?.content?.["application/json"]?.schema;
  assert.equal(responseSchema?.$ref, "#/components/schemas/EvaluationScorecard");

  const scorecard = document.components?.schemas?.EvaluationScorecard;
  assert.ok(scorecard, "missing EvaluationScorecard schema");
  assert.equal(scorecard.type, "object");
  assert.deepEqual([...(scorecard.required ?? [])].sort(), [
    "context_id",
    "metrics",
    "run_count"
  ]);
  assert.equal(scorecard.properties?.context_id?.type, "string");
  assert.equal(scorecard.properties?.run_count?.type, "integer");
  assert.equal(scorecard.properties?.metrics?.type, "array");
  assert.equal(
    scorecard.properties?.metrics?.items?.$ref,
    "#/components/schemas/EvaluationScorecardMetric"
  );

  const metric = document.components?.schemas?.EvaluationScorecardMetric;
  assert.ok(metric, "missing EvaluationScorecardMetric schema");
  assert.equal(metric.type, "object");
  assert.deepEqual([...(metric.required ?? [])].sort(), ["average", "name", "sample_count"]);
  assert.equal(metric.properties?.name?.type, "string");
  assert.equal(metric.properties?.average?.type, "number");
  assert.equal(metric.properties?.sample_count?.type, "integer");
});

function parameterNames(parameters: OpenApiParameter[], location: string) {
  return parameters
    .filter((parameter) => parameter.in === location && parameter.name)
    .map((parameter) => parameter.name as string)
    .sort();
}

function resolveParameters(
  document: OpenApiDocument,
  parameters: OpenApiParameter[]
): OpenApiParameter[] {
  return parameters.map((parameter) => {
    if (!parameter.$ref) {
      return parameter;
    }

    const name = parameter.$ref.replace("#/components/parameters/", "");
    const resolved = document.components?.parameters?.[name];
    assert.ok(resolved, `missing parameter reference ${parameter.$ref}`);
    return resolved;
  });
}

function readOpenApiDocument(): OpenApiDocument {
  const source = readFileSync(openApiPath(), "utf8");
  return JSON.parse(source) as OpenApiDocument;
}

function openApiPath() {
  const directory = dirname(fileURLToPath(import.meta.url));
  return resolve(directory, "../../../docs/api/openapi.json");
}
