"use client";

import { Button, Input, StatusPill } from "@contextlab/ui";
import React, { useState } from "react";
import {
  WORKFLOW_CAPABILITY_RESOURCE_FIXTURES,
  type FrozenWorkflowCapabilityResourceFixture
} from "./workflow-capability-data";
import {
  loadLocalWorkflowCapabilityStatus,
  LocalWorkflowCapabilityProxyError
} from "./local-workflow-capability-data";
import { WorkflowCapabilityInspector } from "./workflow-capability-screen";

export type LocalWorkflowCapabilityInspectorProps = {
  contextId: string;
};

export function LocalWorkflowCapabilityInspector({ contextId }: LocalWorkflowCapabilityInspectorProps) {
  const [bearerToken, setBearerToken] = useState("");
  const [fixture, setFixture] = useState<FrozenWorkflowCapabilityResourceFixture>(
    WORKFLOW_CAPABILITY_RESOURCE_FIXTURES.empty
  );
  const [notice, setNotice] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const canInspect = bearerToken.trim().length > 0 && !isLoading;

  async function inspectCapability() {
    if (!canInspect) {
      return;
    }

    setIsLoading(true);
    setNotice(null);
    setFixture(WORKFLOW_CAPABILITY_RESOURCE_FIXTURES.loading);
    try {
      const status = await loadLocalWorkflowCapabilityStatus(contextId, bearerToken);
      setFixture(
        status.enabled && status.availability === "available"
          ? WORKFLOW_CAPABILITY_RESOURCE_FIXTURES.available
          : WORKFLOW_CAPABILITY_RESOURCE_FIXTURES.unavailable
      );
    } catch (error) {
      setFixture(WORKFLOW_CAPABILITY_RESOURCE_FIXTURES.error);
      setNotice(presentWorkflowCapabilityError(error));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <section aria-labelledby="local-workflow-capability-heading" className="context-benchmark-evidence">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Local Workflow Capability / 本地工作流能力</span>
          <h3 id="local-workflow-capability-heading">Workflow capability status / 工作流能力状态</h3>
          <p>Protected local read-only inspection / 受保护的本地只读检查。No credential is persisted / 不会持久化凭据。</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          label="Bearer token / 访问令牌"
          name="workflow-capability-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Button disabled={!canInspect} tone="muted" type="button" onClick={inspectCapability}>
          {isLoading ? "Inspecting... / 正在审阅..." : "Inspect capability / 审阅能力"}
        </Button>
      </div>
      {notice ? <p className="context-benchmark-evidence__notice" role="alert">{notice}</p> : null}
      <WorkflowCapabilityInspector fixture={fixture} />
    </section>
  );
}

function presentWorkflowCapabilityError(error: unknown): string {
  if (error instanceof LocalWorkflowCapabilityProxyError) {
    if (error.status === 401) {
      return "Bearer authentication is required / 需要 Bearer 身份验证。";
    }
    if (error.status === 403) {
      return "You do not have permission for this Context / 你没有此 Context 的权限。";
    }
    if (error.status === 429) {
      return "The local rate limit is active / 本地速率限制已生效。";
    }
    if (error.status === 503) {
      return "Workflow capability inspection is unavailable / 工作流能力检查暂不可用。";
    }
    return `Unable to inspect workflow capability / 无法检查工作流能力。${error.body.message}`;
  }

  return "Unable to reach local workflow capability inspection / 无法连接本地工作流能力检查。";
}
