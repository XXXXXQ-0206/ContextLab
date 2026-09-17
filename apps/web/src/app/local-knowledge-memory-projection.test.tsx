import assert from "node:assert/strict";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import test from "node:test";
import {
  LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1,
  knowledgeMemoryReplayProjectionId,
  knowledgeMemoryScopeForContext,
  loadLocalKnowledgeMemoryProjection,
  parseLocalKnowledgeMemoryProjectionV1,
  type LocalKnowledgeMemoryProjectionResource,
  type LocalKnowledgeMemoryProjectionV1
} from "./local-knowledge-memory-projection-data";
import { presentLocalKnowledgeMemoryProjection } from "./local-knowledge-memory-projection-presenter";
import { LocalKnowledgeMemoryProjectionScreen } from "./local-knowledge-memory-projection-screen";

const projectId = "00000000-0000-4000-8000-000000000001";
const contextId = "00000000-0000-4000-8000-000000000002";
const commitId = "00000000-0000-4000-8000-000000000003";

test("parses a frozen Context-scoped redacted Knowledge/Memory projection through local SDK", () => {
  const parsed = parseLocalKnowledgeMemoryProjectionV1(projectionPayload());
  assert.equal(parsed.schema_version, LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1);
  assert.equal(parsed.context_id, contextId);
  assert.equal(parsed.projection.knowledge_scope, knowledgeMemoryScopeForContext(contextId));
  assert.equal(parsed.projection.citations[0]?.chunk_id, "00000000-0000-4000-8000-000000000006");
  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.projection.citations), true);
  assert.equal("content" in (parsed.projection.citations[0] as object), false);
});

test("rejects scope drift, raw fields, unsupported schemas, and noncanonical citations", () => {
  const cases: Array<[string, (value: Record<string, unknown>) => void]> = [
    ["knowledge/memory scope", (value) => {
      const projection = value.projection as Record<string, unknown>;
      projection.knowledge_scope = "00000000-0000-4000-8000-000000000099";
    }],
    ["raw body", (value) => {
      const projection = value.projection as Record<string, unknown>;
      const citation = (projection.citations as Array<Record<string, unknown>>)[0]!;
      citation.content = "private source body";
    }],
    ["schema", (value) => { value.schema_version = "contextlab.local-knowledge-memory-projection.v2"; }],
    ["citation order", (value) => {
      const projection = value.projection as Record<string, unknown>;
      const citations = projection.citations as Array<Record<string, unknown>>;
      citations.push({ ...citations[0], chunk_id: "00000000-0000-4000-8000-000000000007" });
      citations.reverse();
    }]
  ];

  for (const [name, mutate] of cases) {
    const value = structuredClone(projectionPayload()) as Record<string, unknown>;
    mutate(value);
    assert.throws(() => parseLocalKnowledgeMemoryProjectionV1(value), TypeError, name);
  }
});

test("loads the exact private project/Context commit with request-scoped Bearer and no cookies", async () => {
  const originalFetch = globalThis.fetch;
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(projectionPayload());
  }) as typeof fetch;

  try {
    const result = await loadLocalKnowledgeMemoryProjection(
      { project_id: projectId, context_id: contextId, commit_id: commitId, capability: { en: "Knowledge and memory", zh: "知识与记忆" } },
      "request-token"
    );
    assert.equal(result.commit_id, commitId);
    assert.equal(requests[0]?.input, `/api/local/projects/${projectId}/contexts/${contextId}/commits/${commitId}/knowledge-memory-projection`);
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("presents every capability state with accessible bilingual status semantics", () => {
  const states: LocalKnowledgeMemoryProjectionResource["kind"][] = ["loading", "error", "empty", "unavailable", "ready"];
  for (const state of states) {
    const resource: LocalKnowledgeMemoryProjectionResource = state === "ready"
      ? { kind: "ready", target: target(), summary: projectionPayload() }
      : { kind: state, target: target() };
    const view = presentLocalKnowledgeMemoryProjection(resource);
    const markup = renderToStaticMarkup(<LocalKnowledgeMemoryProjectionScreen view={view} />);
    assert.match(markup, /aria-labelledby="local-knowledge-memory-projection-heading"/);
    assert.match(markup, /aria-busy=/);
    assert.match(markup, /aria-live=/);
    assert.match(markup, /Knowledge and memory projection \/ Knowledge 与 Memory 投影/);
    if (state === "error") {
      assert.equal((markup.match(/role="alert"/g) ?? []).length, 1);
      assert.match(markup, /aria-live="assertive"/);
    }
    if (state === "empty" || state === "unavailable") {
      assert.equal((markup.match(/role="alert"/g) ?? []).length, 0);
      assert.match(markup, /role="status"/);
      assert.match(markup, /aria-live="polite"/);
    }
    if (state === "ready") {
      assert.match(markup, /Redacted Knowledge citations/);
      assert.match(markup, /Retention decision/);
      assert.doesNotMatch(markup, /private source body/);
    }
  }
});

function target() {
  return { project_id: projectId, context_id: contextId, commit_id: commitId, capability: { en: "Knowledge and memory", zh: "知识与记忆" } } as const;
}

function projectionPayload(): LocalKnowledgeMemoryProjectionV1 {
  const projectionFacts = {
    schema_version: "knowledge-memory-local-replay-v2" as const,
    citation_projection_id: "00000000-0000-4000-8000-000000000005",
    citation_projection_schema_version: "knowledge-local-citation-projection-v1" as const,
    knowledge_scope: knowledgeMemoryScopeForContext(contextId),
    retrieval_id: "00000000-0000-4000-8000-000000000008",
    retrieval_version: "retrieval-v1",
    citations: [{
      document_id: "00000000-0000-4000-8000-000000000009",
      document_revision_id: "00000000-0000-4000-8000-000000000010",
      chunk_id: "00000000-0000-4000-8000-000000000006",
      source_version: "source-v1",
      chunking_version: "chunking-v1",
      ordinal: 0,
      range: { start_byte: 0, end_byte: 12 },
      content_fingerprint: "sha256:chunk-1"
    }],
    memory_id: "00000000-0000-4000-8000-000000000011",
    memory_scope: knowledgeMemoryScopeForContext(contextId),
    memory_timeline_version: 2,
    memory_capability_schema_version: "memory-retention-capability-v1" as const,
    retention_policy_version: "retention-v1",
    retention_decision: "retained_fresh" as const,
    replay_state: "active" as const
  };
  return {
    schema_version: LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1,
    project_id: projectId,
    context_id: contextId,
    commit_id: commitId,
    source_project_id: projectId,
    source_commit_id: commitId,
    projection: {
      id: knowledgeMemoryReplayProjectionId(projectionFacts),
      ...projectionFacts
    }
  };
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" } });
}
