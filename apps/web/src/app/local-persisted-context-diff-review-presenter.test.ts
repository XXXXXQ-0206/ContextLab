import assert from "node:assert/strict";
import test from "node:test";
import { createLocalPersistedContextDiffReviewResource } from "./local-persisted-context-diff-review-data";
import { presentLocalPersistedContextDiffReview } from "./local-persisted-context-diff-review-presenter";
import type { LocalContextMetadata, LocalSemanticDiffV1 } from "@contextlab/local-sdk";

type LocalContextMetadataChangeV1 = NonNullable<LocalSemanticDiffV1["metadata_change"]>;

const target = {
  project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  source_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  target_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("presenter preserves Rust-owned diff sections and produces bilingual counts", () => {
  const resource = createLocalPersistedContextDiffReviewResource({ kind: "ready", target, review: {
    schema_version: "v1",
    source_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.source_commit_id },
    target_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.target_commit_id },
    diff: {
      contract_version: "v1",
      semantic: { graph_diff: { added_nodes: [{ id: "node:a", kind: "memory", label: "Memory" }], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] }, document_changes: [] },
      behavior: { case_changes: [] },
      evaluation: { comparability_fingerprint: "fp", metric_changes: [] }
    }
  } });
  const view = presentLocalPersistedContextDiffReview(resource);
  assert.equal(view.state, "available");
  assert.equal(view.stats[1]?.value, "1");
  assert.equal(view.stats[2]?.value, "0");
  assert.match(view.title, /持久化 Context Diff/);
  assert.equal(view.sections[0]?.rows.length, 0);
  assert.equal(view.sections.find((section) => section.id === "semantic-metadata")?.rows.length, 0);
});

test("presenter keeps unavailable and loading states explicit", () => {
  const view = presentLocalPersistedContextDiffReview({ kind: "unavailable", target, message: "not ready" });
  assert.equal(view.state, "unavailable");
  assert.equal(view.message, "not ready");
  assert.deepEqual(view.stats, []);
});

test("presenter renders an SDK-adapted metadata modification without recalculating the diff", () => {
  const original: LocalContextMetadata = {
    created_at: "2025-07-07T00:00:00Z",
    updated_at: "2025-07-07T00:00:00Z",
    labels: { owner: "support", locale: "en-US" }
  };
  const revised: LocalContextMetadata = {
    created_at: original.created_at,
    updated_at: "2025-07-07T00:00:10Z",
    labels: { owner: "platform", locale: "en-US" }
  };
  const view = presentLocalPersistedContextDiffReview(readyResourceWithMetadataChange({ kind: "modified", original, revised }));
  const metadataSection = view.sections.find((section) => section.id === "semantic-metadata");
  assert.equal(view.stats[2]?.value, "1");
  assert.equal(metadataSection?.rows[0]?.cells[0], "modified / 修改");
  assert.match(metadataSection?.rows[0]?.cells[1] ?? "", /owner=support/);
  assert.match(metadataSection?.rows[0]?.cells[1] ?? "", /owner=platform/);
});

test("presenter renders added metadata using only the SDK union revised value", () => {
  const view = presentLocalPersistedContextDiffReview(readyResourceWithMetadataChange({
    kind: "added",
    revised: metadata("platform", "2025-07-07T00:00:10Z")
  }));
  const row = view.sections.find((section) => section.id === "semantic-metadata")?.rows[0];
  assert.deepEqual(row?.cells[0], "added / 新增");
  assert.match(row?.cells[1] ?? "", /revised \/ 修订/);
  assert.match(row?.cells[1] ?? "", /owner=platform/);
  assert.doesNotMatch(row?.cells[1] ?? "", /original \/ 原始/);
});

test("presenter renders removed metadata using only the SDK union original value", () => {
  const view = presentLocalPersistedContextDiffReview(readyResourceWithMetadataChange({
    kind: "removed",
    original: metadata("support", "2025-07-07T00:00:00Z")
  }));
  const row = view.sections.find((section) => section.id === "semantic-metadata")?.rows[0];
  assert.deepEqual(row?.cells[0], "removed / 移除");
  assert.match(row?.cells[1] ?? "", /original \/ 原始/);
  assert.match(row?.cells[1] ?? "", /owner=support/);
  assert.doesNotMatch(row?.cells[1] ?? "", /revised \/ 修订/);
});

function metadata(owner: string, updatedAt: string): LocalContextMetadata {
  return {
    created_at: "2025-07-07T00:00:00Z",
    updated_at: updatedAt,
    labels: { owner }
  };
}

function readyResourceWithMetadataChange(metadataChange: LocalContextMetadataChangeV1) {
  return createLocalPersistedContextDiffReviewResource({
    kind: "ready",
    target,
    review: {
      schema_version: "v1",
      source_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.source_commit_id },
      target_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.target_commit_id },
      diff: {
        contract_version: "v1",
        semantic: {
          graph_diff: { added_nodes: [], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] },
          document_changes: [],
          metadata_change: metadataChange
        },
        behavior: { case_changes: [] },
        evaluation: { comparability_fingerprint: "fp", metric_changes: [] }
      }
    }
  });
}
