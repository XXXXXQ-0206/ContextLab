import assert from "node:assert/strict";
import test from "node:test";
import { loadContextWorkspace } from "./context-workspace-data";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("loadContextWorkspace returns bundled preview data without an API base URL", async () => {
  delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  const requests: string[] = [];
  globalThis.fetch = (async (input: string | URL | Request) => {
    requests.push(String(input));
    throw new Error("preview mode should not fetch live data");
  }) as typeof fetch;

  try {
    const data = await loadContextWorkspace();

    assert.equal(data.source, "preview");
    assert.equal(data.selectedCommitDetail.id, "support-resolution-agent-initial");
    assert.equal(data.commitGraphDiff, null);
    assert.equal(data.commitGraphDiffUnavailableReason, "authenticated-local-read-required");
    assert.equal(data.workspaceContextGraph.graph.nodes["workspace:default"]?.label, "Default Workspace");
    assert.deepEqual(requests, []);
  } finally {
    restoreEnvironment();
  }
});

test("loadContextWorkspace fetches live commit detail without substituting preview details", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://contextlab.test";
  const requests: string[] = [];
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(String(input));
    requests.push(`${url.pathname}${url.search}`);

    if (url.pathname === "/api/v1/contexts/live-context/evaluation-runs/live-run") {
      return jsonResponse(
        {
          error: "evaluation_run_not_found",
          message: "evaluation run detail unavailable"
        },
        { status: 404 }
      );
    }

    if (url.pathname === "/api/v1/contexts/live-context/evaluation-scorecard") {
      if (
        url.searchParams.get("suite_name") === "Live Regression" &&
        url.searchParams.get("model_version") === "deepseek-chat"
      ) {
        return jsonResponse({
          context_id: "live-context",
          run_count: 1,
          metrics: [
            {
              name: "accuracy",
              average: 0.97,
              sample_count: 1
            }
          ]
        });
      }

      return jsonResponse(
        {
          error: "scorecard_filters_required",
          message: "scorecard must be scoped to the selected evaluation run"
        },
        { status: 400 }
      );
    }

    const body = liveResponse(url.pathname);

    if (!body) {
      return jsonResponse(
        {
          error: "unexpected_route",
          message: `unexpected route ${url.pathname}`
        },
        { status: 500 }
      );
    }

    return jsonResponse(body);
  }) as typeof fetch;

  try {
    const data = await loadContextWorkspace();

    assert.equal(data.source, "live");
    assert.equal(data.selectedCommitDetail.id, "live-commit");
    assert.equal(data.commitGraphDiff, null);
    assert.equal(data.commitGraphDiffUnavailableReason, "authenticated-local-read-required");
    assert.deepEqual(data.selectedCommitDetail.changes, [
      {
        component_id: "live-component",
        operation: "update",
        summary: "Live commit detail from API"
      }
    ]);
    assert.equal(data.selectedComponentDetail.id, "live-component");
    assert.equal(data.selectedEvaluationRunDetail, null);
    assert.equal(data.workspaceContextGraph.graph.nodes["workspace:live-workspace"]?.label, "Live Workspace Graph");
    assert.deepEqual(data.workspaceContextGraph.graph.edges, [
      {
        source: "workspace:live-workspace",
        target: "context:live-context",
        kind: "owns"
      }
    ]);
    assert.deepEqual(data.evaluationScorecard, {
      context_id: "live-context",
      run_count: 1,
      metrics: [
        {
          name: "accuracy",
          average: 0.97,
          sample_count: 1
        }
      ]
    });
    assert.equal(
      requests.includes(
        "/api/v1/contexts/live-context/evaluation-scorecard?suite_name=Live+Regression&model_version=deepseek-chat"
      ),
      true
    );
    assert.equal(requests.includes("/api/v1/workspaces/live-workspace/context-graph"), true);
    assert.equal(
      requests.includes("/api/v1/contexts/live-context/commits/live-commit"),
      true
    );
    assert.equal(
      requests.some((request) => request.includes("/graph-diff")),
      false,
      "SSR must not substitute a server credential for the protected local graph-diff read"
    );
  } finally {
    restoreEnvironment();
  }
});

test("loadContextWorkspace does not issue a protected graph diff request during SSR", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://contextlab.test";
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(String(input));

    if (url.pathname === "/api/v1/contexts/live-context/evaluation-runs/live-run") {
      return jsonResponse({ error: "not_found", message: "not found" }, { status: 404 });
    }

    if (url.pathname === "/api/v1/contexts/live-context/evaluation-scorecard") {
      return jsonResponse({ context_id: "live-context", run_count: 0, metrics: [] });
    }

    const body = liveResponse(url.pathname);
    return body
      ? jsonResponse(body)
      : jsonResponse({ error: "unexpected_route", message: `unexpected route ${url.pathname}` }, { status: 500 });
  }) as typeof fetch;

  try {
    const data = await loadContextWorkspace();

    assert.equal(data.source, "live");
    assert.equal(data.commitGraphDiff, null);
    assert.equal(data.commitGraphDiffUnavailableReason, "authenticated-local-read-required");
  } finally {
    restoreEnvironment();
  }
});

function restoreEnvironment() {
  if (originalApiBaseUrl === undefined) {
    delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  } else {
    process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  }

  globalThis.fetch = originalFetch;
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: {
      "content-type": "application/json"
    },
    ...init
  });
}

function liveResponse(pathname: string) {
  if (pathname === "/api/v1/workspaces") {
    return listResponse([
      {
        id: "live-workspace",
        name: "Live Workspace",
        slug: "live",
        created_at: "2026-07-10T00:00:00Z"
      }
    ]);
  }

  if (pathname === "/api/v1/workspaces/live-workspace/projects") {
    return listResponse([
      {
        id: "live-project",
        workspace_id: "live-workspace",
        name: "Live Project",
        slug: "live-project",
        created_at: "2026-07-10T00:00:00Z"
      }
    ]);
  }

  if (pathname === "/api/v1/workspaces/live-workspace/context-graph") {
    return {
      graph: {
        nodes: {
          "workspace:live-workspace": {
            id: "workspace:live-workspace",
            kind: "workspace",
            label: "Live Workspace Graph"
          },
          "context:live-context": {
            id: "context:live-context",
            kind: "context",
            label: "Live Context Graph"
          }
        },
        edges: [
          {
            source: "workspace:live-workspace",
            target: "context:live-context",
            kind: "owns"
          }
        ]
      }
    };
  }

  if (pathname === "/api/v1/projects/live-project/contexts") {
    return listResponse([
      {
        id: "live-context",
        project_id: "live-project",
        experiment_id: null,
        name: "Live Context",
        description: "Live context returned by the API.",
        created_at: "2026-07-10T00:00:00Z"
      }
    ]);
  }

  if (pathname === "/api/v1/contexts/live-context/commits") {
    return listResponse([
      {
        id: "live-commit",
        context_id: "live-context",
        branch_name: "main",
        message: "Update live context",
        parent_commit_ids: ["parent-commit"],
        change_count: 1,
        authored_at: "2026-07-10T00:00:00Z",
        created_at: "2026-07-10T00:00:00Z"
      },
      {
        id: "live-baseline",
        context_id: "live-context",
        branch_name: "main",
        message: "Create live context",
        parent_commit_ids: [],
        change_count: 1,
        authored_at: "2026-07-09T00:00:00Z",
        created_at: "2026-07-09T00:00:00Z"
      }
    ]);
  }

  if (pathname === "/api/v1/contexts/live-context/commits/live-commit") {
    return {
      id: "live-commit",
      context_id: "live-context",
      branch_name: "main",
      message: "Update live context",
      parent_commit_ids: ["parent-commit"],
      changes: [
        {
          component_id: "live-component",
          operation: "update",
          summary: "Live commit detail from API"
        }
      ],
      change_count: 1,
      authored_at: "2026-07-10T00:00:00Z",
      created_at: "2026-07-10T00:00:00Z"
    };
  }

  if (pathname === "/api/v1/contexts/live-context/components") {
    return listResponse([
      {
        id: "live-component",
        context_id: "live-context",
        kind: "knowledge",
        name: "Live Knowledge",
        content_hash: "sha256:live-knowledge",
        created_at: "2026-07-10T00:00:00Z"
      }
    ]);
  }

  if (pathname === "/api/v1/contexts/live-context/components/live-component") {
    return {
      id: "live-component",
      context_id: "live-context",
      kind: "knowledge",
      name: "Live Knowledge",
      content_hash: "sha256:live-knowledge",
      metadata: {
        source: "live"
      },
      created_at: "2026-07-10T00:00:00Z",
      updated_at: "2026-07-10T00:00:00Z"
    };
  }

  if (pathname === "/api/v1/contexts/live-context/evaluation-runs") {
    return listResponse([
      {
        id: "live-run",
        context_id: "live-context",
        suite_name: "Live Regression",
        model_version: "deepseek-chat",
        temperature: 0.2,
        metric_count: 2,
        executed_at: "2026-07-10T00:00:00Z",
        created_at: "2026-07-10T00:00:00Z"
      }
    ]);
  }

  return null;
}

function listResponse<TItem>(items: TItem[]) {
  return {
    items,
    pagination: {
      page: 1,
      per_page: 20,
      total: items.length
    }
  };
}
