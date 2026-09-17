import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalKnowledgeMemoryProjectionClient,
  ContextLabLocalKnowledgeMemoryProjectionError,
  KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1,
  LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1,
  parseLocalKnowledgeMemoryProjection
} from "./knowledge-memory-projection";

const scope = {
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
};

const projection = {
  schema_version: LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1,
  project_id: scope.projectId,
  context_id: scope.contextId,
  commit_id: scope.commitId,
  source_project_id: scope.projectId,
  source_commit_id: scope.commitId,
  projection: {
    id: "b3e95ec7-a9ea-5cd7-91b9-cae06341965c",
    schema_version: KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1,
    citation_projection_id: "22222222-2222-5222-8222-222222222222",
    citation_projection_schema_version: "knowledge-local-citation-projection-v1",
    knowledge_scope: "e67154bb-d629-52c3-aa5f-44f362ab879e",
    retrieval_id: "33333333-3333-5333-8333-333333333333",
    retrieval_version: "retrieval-v1",
    citations: [
      citation("44444444-4444-5444-8444-444444444444"),
      citation("55555555-5555-5555-8555-555555555555")
    ],
    memory_id: "66666666-6666-5666-8666-666666666666",
    memory_scope: "e67154bb-d629-52c3-aa5f-44f362ab879e",
    memory_timeline_version: 4,
    memory_capability_schema_version: "memory-retention-capability-v1",
    retention_policy_version: "retention-v1",
    retention_decision: "retained_important",
    replay_state: "active"
  }
} as const;

test("parses a frozen, exact redacted Knowledge/Memory projection", () => {
  const parsed = parseLocalKnowledgeMemoryProjection(projection);

  assert.deepEqual(parsed, projection);
  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.projection), true);
  assert.equal(Object.isFrozen(parsed.projection.citations), true);
  assert.equal(Object.isFrozen(parsed.projection.citations[0]), true);
  assert.equal("content" in parsed.projection, false);
  assert.equal("memory_content" in parsed.projection, false);
  assert.equal("embedding" in parsed.projection, false);
});

test("rejects schema drift, raw fields, invalid scope IDs, and unstable citations", () => {
  for (const mutate of [
    (value: Record<string, unknown>) => { value.schema_version = "contextlab.local-knowledge-memory-projection.v2"; },
    (value: Record<string, unknown>) => { value.raw_content = "private"; },
    (value: Record<string, unknown>) => { value.project_id = "not-a-uuid"; },
    (value: Record<string, unknown>) => {
      const nested = value.projection as Record<string, unknown>;
      nested.memory_capability_schema_version = "memory-retention-capability-v2";
    },
    (value: Record<string, unknown>) => {
      const nested = value.projection as Record<string, unknown>;
      nested.citations = [...(nested.citations as unknown[])].reverse();
    },
    (value: Record<string, unknown>) => {
      const nested = value.projection as Record<string, unknown>;
      nested.retention_decision = "expired_forgotten";
    },
    (value: Record<string, unknown>) => { delete value.source_project_id; },
    (value: Record<string, unknown>) => { value.source_project_id = scope.commitId; }
  ]) {
    const copy = structuredClone(projection) as unknown as Record<string, unknown>;
    mutate(copy);
    assert.throws(() => parseLocalKnowledgeMemoryProjection(copy), TypeError);
  }
});

test("rejects Context-derived scope drift and deterministic replay ID drift", () => {
  const scopeDrift = structuredClone(projection) as Record<string, unknown>;
  (scopeDrift.projection as Record<string, unknown>).knowledge_scope = scope.contextId;
  (scopeDrift.projection as Record<string, unknown>).memory_scope = scope.contextId;
  assert.throws(() => parseLocalKnowledgeMemoryProjection(scopeDrift), TypeError);

  const replayIdDrift = structuredClone(projection) as Record<string, unknown>;
  (replayIdDrift.projection as Record<string, unknown>).id =
    "11111111-1111-5111-8111-111111111111";
  assert.throws(() => parseLocalKnowledgeMemoryProjection(replayIdDrift), TypeError);
});

test("preserves every allowlisted Knowledge/Memory server error code", async () => {
  const codes = [
    "knowledge_memory_projection_not_found",
    "knowledge_memory_projection_scope_conflict",
    "knowledge_memory_projection_unavailable",
    "knowledge_memory_projection_invalid",
    "context_read_forbidden",
    "authentication_required",
    "authentication_failed",
    "authorization_unavailable",
    "rate_limit_exceeded",
    "unauthorized",
    "forbidden"
  ];

  for (const code of codes) {
    const client = new ContextLabLocalKnowledgeMemoryProjectionClient({
      fetch: async () => new Response(JSON.stringify({
        error: code,
        message: "private server details"
      }), { status: 503 })
    });

    await assert.rejects(
      () => client.getKnowledgeMemoryProjection(scope.projectId, scope.contextId, scope.commitId, {
        bearerToken: "request-token"
      }),
      (error: unknown) => error instanceof ContextLabLocalKnowledgeMemoryProjectionError
        && error.code === code
        && !error.message.includes("private server details")
    );
  }
});

test("rejects a source commit identity that disagrees with the outer envelope", () => {
  const copy = structuredClone(projection) as Record<string, unknown>;
  copy.source_commit_id = scope.projectId;
  assert.throws(() => parseLocalKnowledgeMemoryProjection(copy), TypeError);
});

test("reads one exact private projection with bearer-only no-store transport", async () => {
  const calls: Array<{ url: string; init: RequestInit | undefined }> = [];
  const client = new ContextLabLocalKnowledgeMemoryProjectionClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: async (input, init) => {
      calls.push({ url: String(input), init });
      return new Response(JSON.stringify(projection), {
        headers: { "content-type": "application/json" }
      });
    }
  });

  const result = await client.getKnowledgeMemoryProjection(
    scope.projectId,
    scope.contextId,
    scope.commitId,
    { bearerToken: "request-token" }
  );

  assert.deepEqual(result, projection);
  assert.equal(
    calls[0]?.url,
    `http://127.0.0.1:3100/api/v1/local/projects/${scope.projectId}/contexts/${scope.contextId}/commits/${scope.commitId}/knowledge-memory-projection`
  );
  const headers = new Headers(calls[0]?.init?.headers);
  assert.equal(headers.get("authorization"), "Bearer request-token");
  assert.equal(headers.get("cookie"), null);
  assert.equal(calls[0]?.init?.method, "GET");
  assert.equal(calls[0]?.init?.credentials, "omit");
  assert.equal(calls[0]?.init?.cache, "no-store");
});

test("rejects response scope drift before returning projection", async () => {
  const client = new ContextLabLocalKnowledgeMemoryProjectionClient({
    fetch: async () => new Response(JSON.stringify({ ...projection, commit_id: scope.projectId }), {
      headers: { "content-type": "application/json" }
    })
  });

  await assert.rejects(
    () => client.getKnowledgeMemoryProjection(scope.projectId, scope.contextId, scope.commitId, {
      bearerToken: "request-token"
    }),
    /projection source scope|requested Context scope/
  );
});

test("redacts upstream error messages and preserves a typed error", async () => {
  const client = new ContextLabLocalKnowledgeMemoryProjectionClient({
    fetch: async () => new Response(JSON.stringify({
      error: "knowledge_memory_projection_unavailable",
      message: "private provider credential and diagnostic details"
    }), { status: 503 })
  });

  await assert.rejects(
    () => client.getKnowledgeMemoryProjection(scope.projectId, scope.contextId, scope.commitId, {
      bearerToken: "request-token"
    }),
    (error: unknown) => error instanceof ContextLabLocalKnowledgeMemoryProjectionError
      && error.code === "knowledge_memory_projection_unavailable"
      && !error.message.includes("credential")
      && !error.message.includes("diagnostic")
  );
});

function citation(chunkId: string) {
  return {
    document_id: "88888888-8888-5888-8888-888888888888",
    document_revision_id: "99999999-9999-5999-8999-999999999999",
    chunk_id: chunkId,
    source_version: "source-v1",
    chunking_version: "chunking-v1",
    ordinal: 0,
    range: { start_byte: 0, end_byte: 16 },
    content_fingerprint: "f".repeat(64)
  };
}
