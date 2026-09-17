"use client";

import React, { useCallback, useMemo, useState } from "react";
import { CommitGraphReview } from "./context-graph-review";
import {
  LocalBranchHeadsInspector
} from "./local-branch-heads-inspector";
import type { LocalBranchHeadTarget } from "./local-branch-heads-data";
import type { ContextLifecycleCommitReviewPair } from "./context-lifecycle-editor";
import type { CommitGraphReviewViewModel } from "./context-workspace-presenter";

export type LocalBranchHeadsGraphReviewProps = Readonly<{
  review: CommitGraphReviewViewModel;
  selectedHeadTarget?: LocalBranchHeadTarget | null;
  onSelectHeadTarget?: (target: LocalBranchHeadTarget | null) => void;
}>;

export function LocalBranchHeadsGraphReview({
  onSelectHeadTarget,
  review,
  selectedHeadTarget
}: LocalBranchHeadsGraphReviewProps) {
  const [internalSelectedHeadTarget, setInternalSelectedHeadTarget] = useState<LocalBranchHeadTarget | null>(null);
  const isControlled = selectedHeadTarget !== undefined;
  const activeSelectedHeadTarget = !isControlled
    ? internalSelectedHeadTarget
    : selectedHeadTarget;
  const selectedReview = useMemo(
    () => composeBranchHeadGraphReview(review, activeSelectedHeadTarget),
    [activeSelectedHeadTarget, review]
  );

  const selectHeadTarget = useCallback((target: LocalBranchHeadTarget | null) => {
    if (!isControlled) {
      setInternalSelectedHeadTarget(target);
    }
    onSelectHeadTarget?.(target);
  }, [isControlled, onSelectHeadTarget]);

  return (
    <>
      <LocalBranchHeadsInspector
        contextId={review.contextId}
        onSelectHeadTarget={selectHeadTarget}
      />
      <CommitGraphReview
        key={`${review.contextId}:${activeSelectedHeadTarget?.head_commit_id ?? "unselected"}:${review.defaultOriginalCommitId ?? "none"}:${review.defaultRevisedCommitId ?? "none"}`}
        review={selectedReview}
      />
    </>
  );
}

export function composeBranchHeadGraphReview(
  review: CommitGraphReviewViewModel,
  selectedHeadTarget: LocalBranchHeadTarget | null
): CommitGraphReviewViewModel {
  if (!selectedHeadTarget) return review;

  const selectedHeadCommitId = selectedHeadTarget.head_commit_id;

  const hasCandidate = review.candidates.some((candidate) => candidate.id === selectedHeadCommitId);
  return {
    ...review,
    candidates: hasCandidate
      ? review.candidates
      : [
          ...review.candidates,
          {
            id: selectedHeadCommitId,
            label: "Selected branch head / 选中分支 head"
          }
        ],
    defaultRevisedCommitId: selectedHeadCommitId
  };
}

export function composeCommittedGraphReview(
  review: CommitGraphReviewViewModel,
  pair: ContextLifecycleCommitReviewPair | null
): CommitGraphReviewViewModel {
  if (!pair) {
    return review;
  }

  const candidates = appendCommitReviewCandidate(
    appendCommitReviewCandidate(review.candidates, pair.originalCommitId, "Previous head / 之前 head"),
    pair.revisedCommitId,
    "Committed revision / 已提交修订"
  );

  return {
    ...review,
    candidates,
    defaultOriginalCommitId: pair.originalCommitId,
    defaultRevisedCommitId: pair.revisedCommitId,
    initialResult: null,
    unavailableMessage: "Enter a request-memory Bearer token to review this commit / 输入请求内存中的 Bearer 令牌以审阅此提交。"
  };
}

function appendCommitReviewCandidate(
  candidates: CommitGraphReviewViewModel["candidates"],
  id: string,
  label: string
): CommitGraphReviewViewModel["candidates"] {
  if (candidates.some((candidate) => candidate.id === id)) {
    return candidates;
  }

  return [...candidates, { id, label }];
}
