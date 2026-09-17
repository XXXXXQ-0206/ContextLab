"use client";

import { Button, Input, StatusPill } from "@contextlab/ui";
import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  createLocalKnowledgeMemoryProjectionResource,
  loadLocalKnowledgeMemoryProjection,
  LocalKnowledgeMemoryProjectionProxyError,
  type LocalKnowledgeMemoryProjectionResource,
  type LocalKnowledgeMemoryProjectionTarget
} from "./local-knowledge-memory-projection-data";
import { LocalKnowledgeMemoryProjectionScreen } from "./local-knowledge-memory-projection-screen";
import { presentLocalKnowledgeMemoryProjection } from "./local-knowledge-memory-projection-presenter";

export type LocalKnowledgeMemoryProjectionInspectorProps = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
}>;

export function LocalKnowledgeMemoryProjectionInspector({ projectId, contextId, commitId }: LocalKnowledgeMemoryProjectionInspectorProps) {
  const [bearerToken, setBearerToken] = useState("");
  const target = useMemo(() => createTarget(projectId, contextId, commitId), [projectId, contextId, commitId]);
  const [resource, setResource] = useState<LocalKnowledgeMemoryProjectionResource>(() => ({ kind: "empty", target }));
  const [notice, setNotice] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const requestId = useRef(0);

  useEffect(() => {
    requestId.current += 1;
    setResource({ kind: "empty", target });
    setNotice(null);
    setIsLoading(false);
  }, [target]);

  async function inspectProjection() {
    if (!bearerToken.trim() || isLoading) return;
    const currentRequestId = ++requestId.current;
    setIsLoading(true);
    setNotice(null);
    setResource({ kind: "loading", target });
    try {
      const summary = await loadLocalKnowledgeMemoryProjection(target, bearerToken);
      if (currentRequestId !== requestId.current) return;
      setResource(createLocalKnowledgeMemoryProjectionResource({ kind: "ready", target, summary }));
    } catch (error) {
      if (currentRequestId !== requestId.current) return;
      setResource({ kind: errorState(error), target });
      setNotice(presentProjectionError(error));
    } finally {
      if (currentRequestId === requestId.current) setIsLoading(false);
    }
  }

  const view = presentLocalKnowledgeMemoryProjection(resource);
  return (
    <section aria-labelledby="local-knowledge-memory-projection-inspector-heading" className="context-benchmark-evidence">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Knowledge and Memory / Knowledge 与 Memory</span>
          <h3 id="local-knowledge-memory-projection-inspector-heading">Private Context projection / 私有 Context 投影</h3>
          <p>Request-memory Bearer only; no credential is persisted. / 仅在请求内存中使用 Bearer，不会持久化凭据。</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          disabled={isLoading}
          label="Bearer token / 访问令牌"
          name="knowledge-memory-projection-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Button disabled={!bearerToken.trim() || isLoading} tone="muted" type="button" onClick={inspectProjection}>
          {isLoading ? "Inspecting... / 正在审阅..." : "Inspect projection / 审阅投影"}
        </Button>
      </div>
      <p className="context-benchmark-evidence__scope">
        Project / 项目: <code>{projectId}</code> · Context / 上下文: <code>{contextId}</code> · Commit / 提交: <code>{commitId}</code>
      </p>
      {notice ? <p className="context-benchmark-evidence__notice">{notice}</p> : null}
      <LocalKnowledgeMemoryProjectionScreen view={view} />
    </section>
  );
}

function createTarget(projectId: string, contextId: string, commitId: string): LocalKnowledgeMemoryProjectionTarget {
  return Object.freeze({
    project_id: projectId,
    context_id: contextId,
    commit_id: commitId,
    capability: Object.freeze({ en: "Knowledge and memory projection", zh: "Knowledge 与 Memory 投影" })
  });
}

function errorState(error: unknown): "error" | "empty" | "unavailable" {
  if (error instanceof LocalKnowledgeMemoryProjectionProxyError) {
    if (error.status === 404) return "empty";
    if (error.status === 503) return "unavailable";
  }
  return "error";
}

function presentProjectionError(error: unknown): string {
  if (error instanceof LocalKnowledgeMemoryProjectionProxyError) {
    if (error.status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
    if (error.status === 403) return "You do not have permission for this Context commit / 你没有此 Context 提交的权限。";
    if (error.status === 404) return "No projection was found for this commit / 未找到此提交的投影。";
    if (error.status === 429) return "The local rate limit is active / 本地速率限制已生效。";
    if (error.status === 503) return "Knowledge/Memory projection is unavailable / Knowledge/Memory 投影暂不可用。";
  }
  if (error instanceof TypeError) return "Projection response was rejected / 投影响应已被拒绝。";
  return "Unable to reach Knowledge/Memory projection / 无法连接 Knowledge/Memory 投影。";
}
