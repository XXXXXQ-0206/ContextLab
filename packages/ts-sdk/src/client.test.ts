import assert from "node:assert/strict";
import test from "node:test";
import { ContextLabApiError, ContextLabClient, type FetchLike } from "./client";
import type { ListResponse, WorkspaceItem } from "./types";

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: {
      "content-type": "application/json"
    },
    ...init
  });
}

test("builds discovery routes with encoded path and query parameters", async () => {
  const requests: string[] = [];
  const workspaces: ListResponse<WorkspaceItem> = {
    items: [],
    pagination: {
      page: 2,
      per_page: 5,
      total: 0
    }
  };
  const fetchImpl: FetchLike = async (input) => {
    requests.push(String(input));
    return jsonResponse(workspaces);
  };
  const client = new ContextLabClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  await client.listProjects("workspace/id", {
    page: 2,
    per_page: 5,
    search: "support bot",
    sort: "-created_at"
  });
  await client.listComponents("context/id", {
    page: 1,
    per_page: 10,
    search: "policy hash",
    kind: "knowledge",
    sort: "kind"
  });
  await client.getCommit("context/id", "commit/id");
  await client.getComponent("context/id", "component/id");
  await client.getEvaluationScorecard("context/id", {
    search: "safety",
    suite_name: "Safety Regression Suite",
    model_version: "deepseek-chat"
  });
  await client.getEvaluationRun("context/id", "run/id");

  assert.deepEqual(requests, [
    "http://127.0.0.1:3100/api/v1/workspaces/workspace%2Fid/projects?page=2&per_page=5&search=support+bot&sort=-created_at",
    "http://127.0.0.1:3100/api/v1/contexts/context%2Fid/components?page=1&per_page=10&search=policy+hash&kind=knowledge&sort=kind",
    "http://127.0.0.1:3100/api/v1/contexts/context%2Fid/commits/commit%2Fid",
    "http://127.0.0.1:3100/api/v1/contexts/context%2Fid/components/component%2Fid",
    "http://127.0.0.1:3100/api/v1/contexts/context%2Fid/evaluation-scorecard?search=safety&suite_name=Safety+Regression+Suite&model_version=deepseek-chat",
    "http://127.0.0.1:3100/api/v1/contexts/context%2Fid/evaluation-runs/run%2Fid"
  ]);
});

test("builds platform, provider, graph, and OpenAPI routes", async () => {
  const requests: string[] = [];
  const fetchImpl: FetchLike = async (input) => {
    requests.push(String(input));
    return jsonResponse({});
  };
  const client = new ContextLabClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  await client.healthz();
  await client.getMeta();
  await client.listProviders();
  await client.getOpenApiDocument();
  await client.getContextGraphPreview();
  await client.getWorkspaceContextGraph("workspace/id");

  assert.deepEqual(requests, [
    "http://127.0.0.1:3100/healthz",
    "http://127.0.0.1:3100/api/v1/meta",
    "http://127.0.0.1:3100/api/v1/providers",
    "http://127.0.0.1:3100/api/v1/openapi.json",
    "http://127.0.0.1:3100/api/v1/context-graph/preview",
    "http://127.0.0.1:3100/api/v1/workspaces/workspace%2Fid/context-graph"
  ]);
});

test("builds a graph diff POST request with explicit snapshots", async () => {
  let requestUrl = "";
  let requestInit: RequestInit | undefined;
  const fetchImpl: FetchLike = async (input, init) => {
    requestUrl = String(input);
    requestInit = init;
    return jsonResponse({
      added_nodes: [],
      removed_nodes: [],
      modified_nodes: [],
      added_edges: [],
      removed_edges: []
    });
  };
  const client = new ContextLabClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });
  const compareGraphs = (client as unknown as {
    compareGraphs?: (request: unknown) => Promise<unknown>;
  }).compareGraphs;

  assert.equal(typeof compareGraphs, "function");
  if (!compareGraphs) {
    return;
  }

  await compareGraphs.call(client, {
    original: {
      nodes: [{ id: "context:agent", kind: "context", label: "Support Agent" }],
      edges: []
    },
    revised: {
      nodes: [{ id: "context:agent", kind: "context", label: "Support Agent v2" }],
      edges: []
    }
  });

  assert.equal(requestUrl, "http://127.0.0.1:3100/api/v1/graph-diffs");
  assert.equal(requestInit?.method, "POST");
  const headers = new Headers(requestInit?.headers);
  assert.equal(headers.get("accept"), "application/json");
  assert.equal(headers.get("content-type"), "application/json");
  assert.equal(
    requestInit?.body,
    JSON.stringify({
      original: {
        nodes: [{ id: "context:agent", kind: "context", label: "Support Agent" }],
        edges: []
      },
      revised: {
        nodes: [{ id: "context:agent", kind: "context", label: "Support Agent v2" }],
        edges: []
      }
    })
  );
});

test("does not expose the retired public version-backed commit graph diff read", () => {
  assert.equal(
    (ContextLabClient.prototype as unknown as Record<string, unknown>).getCommitGraphDiff,
    undefined
  );
});

test("throws structured API errors for non-2xx responses", async () => {
  const fetchImpl: FetchLike = async () =>
    jsonResponse(
      {
        error: "invalid_context_sort",
        message: "invalid sort parameter: latency"
      },
      {
        status: 400
      }
    );
  const client = new ContextLabClient({ fetch: fetchImpl });

  await assert.rejects(() => client.listContexts("support-ai", { sort: "created_at" }), {
    name: "ContextLabApiError",
    status: 400,
    code: "invalid_context_sort",
    message: "invalid sort parameter: latency"
  });
});

test("falls back to a generic API error when error response is not JSON", async () => {
  const fetchImpl: FetchLike = async () => new Response("unavailable", { status: 503 });
  const client = new ContextLabClient({ fetch: fetchImpl });

  await assert.rejects(
    async () => {
      await client.listWorkspaces();
    },
    (error: unknown) => {
      assert.ok(error instanceof ContextLabApiError);
      assert.equal(error.status, 503);
      assert.equal(error.code, "contextlab_api_error");
      assert.equal(error.message, "ContextLab API request failed with status 503");
      return true;
    }
  );
});
