import assert from "node:assert/strict";
import test from "node:test";
import {
  KNOWLEDGE_MEMORY_REPLAY_NAMESPACE_V5,
  KNOWLEDGE_SCOPE_NAMESPACE_V5,
  deriveKnowledgeMemoryReplayProjectionId,
  deriveKnowledgeScope,
  uuidV5
} from "./knowledge-memory-identity";

const contextId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const knowledgeScope = "e67154bb-d629-52c3-aa5f-44f362ab879e";

test("implements the RFC 4122 UUID-v5 SHA-1 test vector", () => {
  assert.equal(
    uuidV5("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "www.widgets.com"),
    "21f7f8de-8051-5b89-8680-0195ef798b6a"
  );
});

test("derives the Rust Knowledge scope from the Context UUID", () => {
  assert.equal(KNOWLEDGE_SCOPE_NAMESPACE_V5, "1c2f6eb7-8c58-5e48-b815-30a1365c7418");
  assert.equal(deriveKnowledgeScope(contextId), knowledgeScope);
});

test("matches the Rust length-prefixed replay projection identity", () => {
  assert.equal(
    KNOWLEDGE_MEMORY_REPLAY_NAMESPACE_V5,
    "325a50f1-9b23-55c8-8566-2a4b84a3cfd0"
  );
  assert.equal(
    deriveKnowledgeMemoryReplayProjectionId({
      schema_version: "knowledge-memory-local-replay-v2",
      citation_projection_id: "22222222-2222-5222-8222-222222222222",
      citation_projection_schema_version: "knowledge-local-citation-projection-v1",
      knowledge_scope: knowledgeScope,
      retrieval_id: "33333333-3333-5333-8333-333333333333",
      retrieval_version: "retrieval-v1",
      citations: [
        citation("44444444-4444-5444-8444-444444444444"),
        citation("55555555-5555-5555-8555-555555555555")
      ],
      memory_id: "66666666-6666-5666-8666-666666666666",
      memory_scope: knowledgeScope,
      memory_timeline_version: 4,
      memory_capability_schema_version: "memory-retention-capability-v1",
      retention_policy_version: "retention-v1",
      retention_decision: "retained_important",
      replay_state: "active"
    }),
    "b3e95ec7-a9ea-5cd7-91b9-cae06341965c"
  );
});

test("changes replay identity when a length-prefixed source fact changes", () => {
  const base = {
    schema_version: "knowledge-memory-local-replay-v2",
    citation_projection_id: "22222222-2222-5222-8222-222222222222",
    citation_projection_schema_version: "knowledge-local-citation-projection-v1",
    knowledge_scope: knowledgeScope,
    retrieval_id: "33333333-3333-5333-8333-333333333333",
    retrieval_version: "retrieval-v1",
    citations: [citation("44444444-4444-5444-8444-444444444444")],
    memory_id: "66666666-6666-5666-8666-666666666666",
    memory_scope: knowledgeScope,
    memory_timeline_version: 4,
    memory_capability_schema_version: "memory-retention-capability-v1",
    retention_policy_version: "retention-v1",
    retention_decision: "retained_important",
    replay_state: "active"
  } as const;

  assert.notEqual(
    deriveKnowledgeMemoryReplayProjectionId(base),
    deriveKnowledgeMemoryReplayProjectionId({ ...base, retrieval_version: "retrieval-v2" })
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
