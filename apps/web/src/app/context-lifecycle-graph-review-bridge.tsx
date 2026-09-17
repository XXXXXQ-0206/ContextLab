"use client";

import React, { useCallback, useState } from "react";
import { ContextLifecycleEditor } from "./context-lifecycle-editor";
import { LocalBranchHeadsGraphReview } from "./local-branch-heads-graph-review";
import {
  composeBranchHeadGraphReview,
  composeCommittedGraphReview
} from "./local-branch-heads-graph-review";
import type { LocalBranchHeadTarget } from "./local-branch-heads-data";
import type { ContextLifecycleCommitReviewPair } from "./context-lifecycle-editor";
import type {
  CommitGraphReviewViewModel,
  CommitGraphReviewOption
} from "./context-workspace-presenter";

export type ContextLifecycleGraphReviewBridgeProps = Readonly<{
  contextId: string;
  candidates: ReadonlyArray<CommitGraphReviewOption>;
  localLifecycleEnabled: boolean;
  review: CommitGraphReviewViewModel;
}>;

type ScopedCommittedPair = Readonly<{
  scopeKey: string;
  pair: Parameters<typeof composeCommittedGraphReview>[1];
}>;

type ScopedSelectedHeadTarget = Readonly<{
  scopeKey: string;
  target: LocalBranchHeadTarget | null;
}>;

export type ContextLifecycleBranchHeadBinding = Readonly<{
  lifecycleTarget: LocalBranchHeadTarget | null;
  review: CommitGraphReviewViewModel;
}>;

export function createContextLifecycleScopeKey(
  contextId: string,
  candidates: ReadonlyArray<CommitGraphReviewOption>,
  selectedHeadTarget: LocalBranchHeadTarget | null = null
): string {
  return JSON.stringify([
    contextId,
    candidates.map((candidate) => [candidate.id, candidate.label]),
    selectedHeadTarget
      ? [selectedHeadTarget.branch_name, selectedHeadTarget.head_commit_id]
      : null
  ]);
}

export function composeContextLifecycleBranchHeadBinding(
  review: CommitGraphReviewViewModel,
  selectedHeadTarget: LocalBranchHeadTarget | null
): ContextLifecycleBranchHeadBinding {
  return {
    lifecycleTarget: selectedHeadTarget,
    review: composeBranchHeadGraphReview(review, selectedHeadTarget)
  };
}

export function composeContextLifecycleGraphReview(
  review: CommitGraphReviewViewModel,
  selectedHeadTarget: LocalBranchHeadTarget | null,
  committedPair: ContextLifecycleCommitReviewPair | null,
  committedPairScopeKey: string | null,
  currentScopeKey: string
): CommitGraphReviewViewModel {
  const branchHeadReview = composeContextLifecycleBranchHeadBinding(review, selectedHeadTarget).review;
  const scopedCommittedPair = committedPairScopeKey === currentScopeKey ? committedPair : null;
  return composeCommittedGraphReview(branchHeadReview, scopedCommittedPair);
}

export function ContextLifecycleGraphReviewBridge({
  candidates,
  contextId,
  localLifecycleEnabled,
  review
}: ContextLifecycleGraphReviewBridgeProps) {
  const scopeKey = createContextLifecycleScopeKey(contextId, candidates);
  const [scopedCommittedPair, setScopedCommittedPair] = useState<ScopedCommittedPair | null>(null);
  const [scopedSelectedHeadTarget, setScopedSelectedHeadTarget] = useState<ScopedSelectedHeadTarget | null>(null);
  const selectedHeadTarget = scopedSelectedHeadTarget?.scopeKey === scopeKey
    ? scopedSelectedHeadTarget.target
    : null;
  const targetScopeKey = createContextLifecycleScopeKey(contextId, candidates, selectedHeadTarget);
  const committedPair = scopedCommittedPair?.scopeKey === targetScopeKey ? scopedCommittedPair.pair : null;
  const composedReview = composeContextLifecycleGraphReview(
    review,
    selectedHeadTarget,
    committedPair,
    scopedCommittedPair?.scopeKey ?? null,
    targetScopeKey
  );
  const selectedTargetKey = selectedHeadTarget
    ? `${selectedHeadTarget.branch_name}:${selectedHeadTarget.head_commit_id}`
    : "none";
  const selectHeadTarget = useCallback(
    (target: LocalBranchHeadTarget | null) => {
      setScopedSelectedHeadTarget({ scopeKey, target });
      setScopedCommittedPair(null);
    },
    [scopeKey]
  );

  return (
    <>
      {localLifecycleEnabled ? (
        <ContextLifecycleEditor
          key={`lifecycle-editor:${scopeKey}:${selectedTargetKey}`}
          candidates={candidates.map((candidate) => ({ id: candidate.id, label: candidate.label }))}
          contextId={contextId}
          onCommitSuccess={(pair) => setScopedCommittedPair({ scopeKey: targetScopeKey, pair })}
          selectedBranchHeadTarget={selectedHeadTarget}
        />
      ) : null}
      <LocalBranchHeadsGraphReview
        key={`graph-review:${scopeKey}:${committedPair?.originalCommitId ?? "none"}:${committedPair?.revisedCommitId ?? "none"}`}
        onSelectHeadTarget={selectHeadTarget}
        review={composedReview}
        selectedHeadTarget={selectedHeadTarget}
      />
    </>
  );
}
