"use client";

import { Button, Input, Select, StatusPill } from "@contextlab/ui";
import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  createLocalPersistedContextDiffReviewResource,
  loadLocalPersistedContextDiffReview,
  LocalPersistedContextDiffReviewProxyError,
  type LocalPersistedContextDiffReviewResource
} from "./local-persisted-context-diff-review-data";
import { presentLocalPersistedContextDiffReview } from "./local-persisted-context-diff-review-presenter";
import { LocalPersistedContextDiffReviewScreen } from "./local-persisted-context-diff-review-screen";

export type LocalPersistedContextDiffReviewInspectorProps = Readonly<{
  enabled?: boolean;
  projectId: string;
  contextId: string;
  candidates: ReadonlyArray<Readonly<{ id: string; label: string }>>;
  defaultSourceCommitId?: string | null;
  defaultTargetCommitId?: string | null;
}>;

export function LocalPersistedContextDiffReviewInspector({ enabled = false, projectId, contextId, candidates, defaultSourceCommitId, defaultTargetCommitId }: LocalPersistedContextDiffReviewInspectorProps) {
  const [sourceCommitId, setSourceCommitId] = useState(defaultSourceCommitId ?? candidates[0]?.id ?? "");
  const [targetCommitId, setTargetCommitId] = useState(defaultTargetCommitId ?? candidates[1]?.id ?? "");
  const target = useMemo(() => ({ project_id: projectId, context_id: contextId, source_commit_id: sourceCommitId, target_commit_id: targetCommitId }), [projectId, contextId, sourceCommitId, targetCommitId]);
  const [bearerToken, setBearerToken] = useState("");
  const [resource, setResource] = useState<LocalPersistedContextDiffReviewResource>(() => ({ kind: "empty", target }));
  const [notice, setNotice] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const requestId = useRef(0);

  useEffect(() => {
    requestId.current += 1;
    setResource({ kind: "empty", target });
    setNotice(null);
    setIsLoading(false);
  }, [target]);

  if (!enabled) return null;

  async function inspectReview() {
    if (!bearerToken.trim() || isLoading || !sourceCommitId || !targetCommitId || sourceCommitId === targetCommitId) return;
    const currentRequestId = ++requestId.current;
    setIsLoading(true);
    setNotice(null);
    setResource({ kind: "loading", target });
    try {
      const review = await loadLocalPersistedContextDiffReview(target, bearerToken);
      if (currentRequestId !== requestId.current) return;
      setResource(createLocalPersistedContextDiffReviewResource({ kind: "ready", target, review }));
    } catch (error) {
      if (currentRequestId !== requestId.current) return;
      const kind = error instanceof LocalPersistedContextDiffReviewProxyError && error.status === 404 ? "empty" : error instanceof LocalPersistedContextDiffReviewProxyError && error.status === 503 ? "unavailable" : "error";
      setResource({ kind, target });
      setNotice(presentError(error));
    } finally {
      if (currentRequestId === requestId.current) setIsLoading(false);
    }
  }

  return (
    <section aria-labelledby="local-persisted-context-diff-review-inspector-heading" className="context-benchmark-evidence">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Version history / 版本历史</span>
          <h3 id="local-persisted-context-diff-review-inspector-heading">Private persisted diff / 私有持久化 Diff</h3>
          <p>Request-memory Bearer only; no credential is persisted. / 仅在请求内存中使用 Bearer，不会持久化凭据。</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input autoComplete="off" description="Memory only / 仅内存" disabled={isLoading} label="Bearer token / 访问令牌" name="persisted-context-diff-review-bearer-token" onChange={(event) => setBearerToken(event.target.value)} type="password" value={bearerToken} />
        <Select aria-label="Source commit / 来源提交" disabled={isLoading} label="Source / 来源" onChange={(event) => setSourceCommitId(event.target.value)} value={sourceCommitId}>
          {candidates.map((candidate) => <option key={candidate.id} value={candidate.id}>{candidate.label}</option>)}
        </Select>
        <Select aria-label="Target commit / 目标提交" disabled={isLoading} label="Target / 目标" onChange={(event) => setTargetCommitId(event.target.value)} value={targetCommitId}>
          {candidates.map((candidate) => <option key={candidate.id} value={candidate.id}>{candidate.label}</option>)}
        </Select>
        <Button disabled={!bearerToken.trim() || sourceCommitId === targetCommitId || isLoading} tone="muted" type="button" onClick={inspectReview}>
          {isLoading ? "Loading... / 正在加载..." : "Review diff / 审阅 Diff"}
        </Button>
      </div>
      {notice ? <p className="context-benchmark-evidence__notice">{notice}</p> : null}
      <LocalPersistedContextDiffReviewScreen view={presentLocalPersistedContextDiffReview(resource)} />
    </section>
  );
}

function presentError(error: unknown): string {
  if (error instanceof LocalPersistedContextDiffReviewProxyError) {
    if (error.status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
    if (error.status === 403) return "You do not have permission for this Context / 你没有此 Context 的权限。";
    if (error.status === 404) return "No persisted review was found / 未找到持久化审阅。";
    if (error.status === 429) return "The local rate limit is active / 本地速率限制已生效。";
    if (error.status === 503) return "Persisted review is unavailable / 持久化审阅暂不可用。";
  }
  if (error instanceof TypeError) return "Review response was rejected / 审阅响应已被拒绝。";
  return "Unable to load persisted review / 无法加载持久化审阅。";
}
