import assert from "node:assert/strict";
import test from "node:test";
import {
  composeBranchHeadGraphReview,
  composeCommittedGraphReview
} from "./local-branch-heads-graph-review";
import {
  composeContextLifecycleBranchHeadBinding,
  composeContextLifecycleGraphReview,
  createContextLifecycleScopeKey
} from "./context-lifecycle-graph-review-bridge";
import type { CommitGraphReviewViewModel } from "./context-workspace-presenter";

const review: CommitGraphReviewViewModel = {
  title: "Commit Graph Review / 提交图谱审查",
  contextId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  candidates: [
    { id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", label: "Baseline" },
    { id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", label: "Current" }
  ],
  defaultOriginalCommitId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  defaultRevisedCommitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  isInteractive: true,
  initialResult: null,
  unavailableMessage: null
};

test("selected branch head becomes the exact revised graph-review candidate", () => {
  const selected = {
    branch_name: "feature/read-only",
    head_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
  } as const;
  const composed = composeBranchHeadGraphReview(review, selected);

  assert.equal(composed.defaultOriginalCommitId, review.defaultOriginalCommitId);
  assert.equal(composed.defaultRevisedCommitId, selected.head_commit_id);
  assert.deepEqual(composed.candidates.at(-1), {
    id: selected.head_commit_id,
    label: "Selected branch head / 选中分支 head"
  });
  assert.equal(review.defaultRevisedCommitId, "cccccccc-cccc-4ccc-8ccc-cccccccccccc");
});

test("existing candidates are not duplicated and unborn heads preserve the review defaults", () => {
  const existing = composeBranchHeadGraphReview(
    review,
    { branch_name: "main", head_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc" }
  );
  assert.equal(existing.candidates.length, review.candidates.length);
  assert.equal(existing.defaultRevisedCommitId, "cccccccc-cccc-4ccc-8ccc-cccccccccccc");

  const unborn = composeBranchHeadGraphReview(review, null);
  assert.strictEqual(unborn, review);
});

test("a committed lifecycle pair becomes the exact graph-review selection", () => {
  const composed = composeCommittedGraphReview(review, {
    originalCommitId: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
    revisedCommitId: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"
  });

  assert.equal(composed.defaultOriginalCommitId, "dddddddd-dddd-4ddd-8ddd-dddddddddddd");
  assert.equal(composed.defaultRevisedCommitId, "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee");
  assert.deepEqual(composed.candidates.slice(-2), [
    { id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd", label: "Previous head / 之前 head" },
    { id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee", label: "Committed revision / 已提交修订" }
  ]);
  assert.equal(composed.initialResult, null);
  assert.match(composed.unavailableMessage ?? "", /request-memory Bearer/);
});

test("a missing lifecycle pair preserves the existing graph-review state", () => {
  assert.strictEqual(composeCommittedGraphReview(review, null), review);
});

test("lifecycle review scope changes when Context or materialized candidates change", () => {
  const base = createContextLifecycleScopeKey("context-a", [
    { id: "commit-a", label: "A" }
  ]);
  const otherContext = createContextLifecycleScopeKey("context-b", [
    { id: "commit-a", label: "A" }
  ]);
  const otherCandidates = createContextLifecycleScopeKey("context-a", [
    { id: "commit-b", label: "B" }
  ]);

  assert.notEqual(base, otherContext);
  assert.notEqual(base, otherCandidates);
});

test("the selected server-owned target drives lifecycle editing and graph review together", () => {
  const selected = Object.freeze({
    branch_name: "feature/read-only",
    head_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
  });
  const binding = composeContextLifecycleBranchHeadBinding(review, selected);

  assert.strictEqual(binding.lifecycleTarget, selected);
  assert.equal(binding.review.defaultRevisedCommitId, selected.head_commit_id);
  assert.equal(
    binding.review.candidates.some((candidate) => candidate.id === selected.head_commit_id),
    true
  );
});

test("null branch-head targets clear lifecycle and graph-review selection", () => {
  const binding = composeContextLifecycleBranchHeadBinding(review, null);

  assert.equal(binding.lifecycleTarget, null);
  assert.strictEqual(binding.review, review);
});

test("a committed pair from the previous branch-head target cannot override the new target", () => {
  const previousTarget = {
    branch_name: "feature/previous",
    head_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
  } as const;
  const nextTarget = {
    branch_name: "feature/next",
    head_commit_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"
  } as const;
  const previousScope = createContextLifecycleScopeKey("context-a", review.candidates, previousTarget);
  const nextScope = createContextLifecycleScopeKey("context-a", review.candidates, nextTarget);
  const composed = composeContextLifecycleGraphReview(
    review,
    nextTarget,
    {
      originalCommitId: "ffffffff-ffff-4fff-8fff-ffffffffffff",
      revisedCommitId: previousTarget.head_commit_id
    },
    previousScope,
    nextScope
  );

  assert.equal(composed.defaultRevisedCommitId, nextTarget.head_commit_id);
  assert.equal(composed.defaultOriginalCommitId, review.defaultOriginalCommitId);
  assert.equal(composed.initialResult, review.initialResult);

  const clearedScope = createContextLifecycleScopeKey("context-a", review.candidates, null);
  const cleared = composeContextLifecycleGraphReview(
    review,
    null,
    {
      originalCommitId: "ffffffff-ffff-4fff-8fff-ffffffffffff",
      revisedCommitId: previousTarget.head_commit_id
    },
    previousScope,
    clearedScope
  );
  assert.strictEqual(cleared, review);
});
