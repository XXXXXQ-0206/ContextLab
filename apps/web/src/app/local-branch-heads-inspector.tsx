"use client";

import { Button, Input, StatusPill } from "@contextlab/ui";
import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  createInitialLocalBranchHeadsResource,
  loadLocalBranchHeads,
  LocalBranchHeadsProxyError,
  selectedLocalBranchHeadTarget,
  type LocalBranchHeadTarget,
  type LocalBranchHeadsResource,
  type LocalBranchHeadsTarget
} from "./local-branch-heads-data";
import { presentLocalBranchHeads } from "./local-branch-heads-presenter";
import { LocalBranchHeadsScreen } from "./local-branch-heads-screen";

export type LocalBranchHeadsInspectorProps = Readonly<{
  contextId: string;
  onSelectHeadTarget?: (target: LocalBranchHeadTarget | null) => void;
  onSelectHeadCommit?: (headCommitId: string | null) => void;
}>;

export function LocalBranchHeadsInspector({
  contextId,
  onSelectHeadCommit,
  onSelectHeadTarget
}: LocalBranchHeadsInspectorProps) {
  const target = useMemo(() => createTarget(contextId), [contextId]);
  const [bearerToken, setBearerToken] = useState("");
  const [resource, setResource] = useState<LocalBranchHeadsResource>(() =>
    createInitialLocalBranchHeadsResource(target)
  );
  const [selectedBranch, setSelectedBranch] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const requestGeneration = useRef(0);
  const view = presentLocalBranchHeads(resource, selectedBranch);
  const canInspect = bearerToken.trim().length > 0 && !isLoading;

  useEffect(() => {
    requestGeneration.current += 1;
    setResource(createInitialLocalBranchHeadsResource(target));
    setSelectedBranch("");
    setNotice(null);
    setIsLoading(false);
    selectHeadTarget(null);
  }, [target]);

  async function inspectBranchHeads() {
    if (!canInspect) return;

    const generation = requestGeneration.current + 1;
    requestGeneration.current = generation;
    setIsLoading(true);
    setNotice(null);
    setResource({ kind: "loading", target });
    selectHeadTarget(null);

    try {
      const summary = await loadLocalBranchHeads(target, bearerToken.trim());
      if (requestGeneration.current !== generation) return;
      setResource({ kind: "ready", target, summary });
      const firstBranch = summary.branches[0];
      setSelectedBranch(firstBranch?.branch_name ?? "");
      selectHeadTarget(firstBranch ? selectedLocalBranchHeadTarget({ kind: "ready", target, summary }, firstBranch.branch_name) : null);
    } catch (error) {
      if (requestGeneration.current !== generation) return;
      setResource({ kind: "error", target });
      setNotice(presentBranchHeadsError(error));
    } finally {
      if (requestGeneration.current === generation) setIsLoading(false);
    }
  }

  function selectBranch(branchName: string) {
    setSelectedBranch(branchName);
    selectHeadTarget(selectedLocalBranchHeadTarget(resource, branchName));
  }

  function selectHeadTarget(nextTarget: LocalBranchHeadTarget | null) {
    onSelectHeadTarget?.(nextTarget);
    onSelectHeadCommit?.(nextTarget?.head_commit_id ?? null);
  }

  return (
    <section aria-labelledby="local-branch-heads-inspector-heading" className="context-benchmark-evidence">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Branch Heads / 分支 Head</span>
          <h3 id="local-branch-heads-inspector-heading">Context branch discovery / Context 分支发现</h3>
          <p>Protected local read-only inspection / 受保护的本地只读检查。No credential is persisted / 不会持久化凭据。</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          label="Bearer token / 访问令牌"
          name="local-branch-heads-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Button disabled={!canInspect} tone="muted" type="button" onClick={inspectBranchHeads}>
          {isLoading ? "Inspecting... / 正在检查..." : "Inspect branches / 检查分支"}
        </Button>
      </div>
      <p className="context-benchmark-evidence__scope">
        Context / 上下文: <code>{contextId}</code>
      </p>
      {notice ? (
        <p className="context-benchmark-evidence__notice" role="alert">
          {notice}
        </p>
      ) : null}
      <LocalBranchHeadsScreen onSelectBranch={selectBranch} view={view} />
    </section>
  );
}

function createTarget(contextId: string): LocalBranchHeadsTarget {
  return Object.freeze({
    contextId,
    capability: Object.freeze({
      en: "Local branch heads",
      zh: "本地分支 head"
    })
  });
}

function presentBranchHeadsError(error: unknown): string {
  if (error instanceof LocalBranchHeadsProxyError) {
    if (error.status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
    if (error.status === 403) return "You do not have permission for this Context / 你没有此 Context 的权限。";
    if (error.status === 429) return "The local rate limit is active / 本地速率限制已生效。";
    if (error.status === 503) return "Branch-head inspection is unavailable / 分支 head 检查暂不可用。";
  }
  if (error instanceof TypeError) return "Branch-head response was rejected / 分支 head 响应已被拒绝。";
  return "Unable to reach branch-head inspection / 无法连接分支 head 检查。";
}
