"use client";

import { Button, Input, Select, StatusPill } from "@contextlab/ui";
import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  createLocalContextMergeReviewResource,
  loadLocalContextMergeReview,
  LocalContextMergeReviewProxyError,
  type LocalContextMergeReviewResource
} from "./local-context-merge-review-data";
import { presentLocalContextMergeReview } from "./local-context-merge-review-presenter";
import { LocalContextMergeReviewScreen } from "./local-context-merge-review-screen";

export type LocalContextMergeReviewInspectorProps = Readonly<{
  enabled?: boolean;
  projectId: string;
  contextId: string;
  candidates: ReadonlyArray<Readonly<{ id: string; label: string }>>;
  defaultLeftCommitId?: string | null;
  defaultRightCommitId?: string | null;
}>;

export function LocalContextMergeReviewInspector({
  enabled = false,
  projectId,
  contextId,
  candidates,
  defaultLeftCommitId,
  defaultRightCommitId
}: LocalContextMergeReviewInspectorProps) {
  const [leftCommitId, setLeftCommitId] = useState(defaultLeftCommitId ?? candidates[0]?.id ?? "");
  const [rightCommitId, setRightCommitId] = useState(defaultRightCommitId ?? candidates[1]?.id ?? "");
  const candidateIdsKey = useMemo(() => candidates.map((candidate) => candidate.id).join("\u001f"), [candidates]);
  const target = useMemo(
    () => ({
      project_id: projectId,
      context_id: contextId,
      left_commit_id: leftCommitId,
      right_commit_id: rightCommitId
    }),
    [contextId, leftCommitId, projectId, rightCommitId]
  );
  const [bearerToken, setBearerToken] = useState("");
  const [resource, setResource] = useState<LocalContextMergeReviewResource>(() => ({ kind: "empty", target }));
  const [notice, setNotice] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const requestId = useRef(0);

  useEffect(() => {
    setLeftCommitId(defaultLeftCommitId ?? candidates[0]?.id ?? "");
    setRightCommitId(defaultRightCommitId ?? candidates[1]?.id ?? "");
  }, [candidateIdsKey, contextId, defaultLeftCommitId, defaultRightCommitId, projectId]);

  useEffect(() => {
    requestId.current += 1;
    setResource({ kind: "empty", target });
    setNotice(null);
    setIsLoading(false);
  }, [target]);

  if (!enabled) return null;

  async function inspectReview() {
    if (
      !bearerToken.trim()
      || isLoading
      || !leftCommitId
      || !rightCommitId
      || leftCommitId === rightCommitId
    ) return;

    const currentRequestId = ++requestId.current;
    setIsLoading(true);
    setNotice(null);
    setResource({ kind: "loading", target });
    try {
      const review = await loadLocalContextMergeReview(target, bearerToken);
      if (currentRequestId !== requestId.current) return;
      setResource(createLocalContextMergeReviewResource({ kind: "ready", target, review }));
    } catch (error) {
      if (currentRequestId !== requestId.current) return;
      const kind = error instanceof LocalContextMergeReviewProxyError && error.status === 404
        ? "empty"
        : error instanceof LocalContextMergeReviewProxyError && error.status === 503
          ? "unavailable"
          : "error";
      setResource({ kind, target });
      setNotice(presentError(error));
    } finally {
      if (currentRequestId === requestId.current) setIsLoading(false);
    }
  }

  return (
    <section
      aria-labelledby="local-context-merge-review-inspector-heading"
      className="context-benchmark-evidence"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Version history / 版本历史</span>
          <h3 id="local-context-merge-review-inspector-heading">Private Context merge review / 私有 Context Merge 审阅</h3>
          <p>Server-owned ancestry, read-only, request-memory Bearer only. / 服务端拥有 ancestry，只读，仅在请求内存使用 Bearer。</p>
        </div>
        <StatusPill tone="info">read-only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          disabled={isLoading}
          label="Bearer token / 访问令牌"
          name="context-merge-review-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Select
          aria-label="Left commit / 左支线提交"
          disabled={isLoading}
          label="Left / 左支线"
          onChange={(event) => setLeftCommitId(event.target.value)}
          value={leftCommitId}
        >
          {candidates.map((candidate) => <option key={candidate.id} value={candidate.id}>{candidate.label}</option>)}
        </Select>
        <Select
          aria-label="Right commit / 右支线提交"
          disabled={isLoading}
          label="Right / 右支线"
          onChange={(event) => setRightCommitId(event.target.value)}
          value={rightCommitId}
        >
          {candidates.map((candidate) => <option key={candidate.id} value={candidate.id}>{candidate.label}</option>)}
        </Select>
        <Button
          disabled={!bearerToken.trim() || leftCommitId === rightCommitId || isLoading}
          onClick={() => void inspectReview()}
          tone="muted"
          type="button"
        >
          {isLoading ? "Loading... / 正在加载..." : "Review merge / 审阅 Merge"}
        </Button>
      </div>
      {notice ? <p className="context-benchmark-evidence__notice">{notice}</p> : null}
      <LocalContextMergeReviewScreen view={presentLocalContextMergeReview(resource)} />
    </section>
  );
}

function presentError(error: unknown): string {
  if (error instanceof LocalContextMergeReviewProxyError) {
    if (error.status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
    if (error.status === 403) return "You do not have permission for this Context / 你没有此 Context 的权限。";
    if (error.status === 404) return "No merge review was found / 未找到 Merge 审阅。";
    if (error.status === 429) return "The local rate limit is active / 本地速率限制已生效。";
    if (error.status === 503) return "Merge review is unavailable / Merge 审阅暂不可用。";
  }
  if (error instanceof TypeError) return "Review response was rejected / 审阅响应已被拒绝。";
  return "Unable to load merge review / 无法加载 Merge 审阅。";
}
