"use client";

import {
  Button,
  CodeChip,
  DefinitionGrid,
  Input,
  StackTable,
  StatusPill
} from "@contextlab/ui";
import type { LocalContextLifecycleState } from "@contextlab/local-sdk";
import { Eye } from "lucide-react";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import {
  loadLocalContextLifecycleState,
  LocalLifecycleProxyError
} from "./context-lifecycle-data";
import {
  lifecycleErrorMessage,
  presentContextLifecycleReadInspector,
  type ContextLifecycleReadResource,
  type ContextLifecycleReadTarget
} from "./context-lifecycle-presenter";

export type ContextLifecycleReadInspectorProps = Readonly<{
  contextId: string;
  commitId: string;
}>;

export type ContextLifecycleReadInspectorScreenProps = Readonly<{
  resource: ContextLifecycleReadResource;
}>;

export function ContextLifecycleReadInspector({ contextId, commitId }: ContextLifecycleReadInspectorProps) {
  const [bearerToken, setBearerToken] = useState("");
  const target = useMemo(() => createTarget(contextId, commitId), [contextId, commitId]);
  const [resource, setResource] = useState<ContextLifecycleReadResource>(() => emptyResource(target));
  const requestGeneration = useRef(0);
  const currentResource = sameTarget(resource.target, target) ? resource : emptyResource(target);
  const isLoading = currentResource.kind === "loading";
  const canInspect = Boolean(bearerToken.trim() && target.contextId && target.commitId && !isLoading);

  useEffect(() => {
    requestGeneration.current += 1;
    setResource(emptyResource(target));
  }, [target]);

  async function inspectLifecycleState() {
    if (!canInspect) {
      return;
    }

    const generation = requestGeneration.current + 1;
    requestGeneration.current = generation;
    setResource({ kind: "loading", target });

    try {
      const state = await loadLocalContextLifecycleState(
        target.contextId,
        target.commitId,
        bearerToken
      );
      if (requestGeneration.current !== generation) {
        return;
      }
      setResource({ kind: "available", target, state });
    } catch (error) {
      if (requestGeneration.current !== generation) {
        return;
      }
      setResource({
        kind: lifecycleReadState(error),
        target,
        ...(lifecycleReadMessage(error) ? { message: lifecycleReadMessage(error) } : {})
      });
    }
  }

  return (
    <section
      aria-labelledby="context-lifecycle-read-inspector-heading"
      className="context-benchmark-evidence"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Context Lifecycle Read / Context 生命周期读取</span>
          <h3 id="context-lifecycle-read-inspector-heading">Selected commit lifecycle / 选定提交生命周期</h3>
          <p>
            Protected local read-only inspection / 受保护的本地只读检查。No credential is persisted / 不会持久化凭据。
          </p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      <div className="context-benchmark-evidence__controls">
        <Input
          autoComplete="off"
          description="Memory only / 仅内存"
          label="Bearer token / 访问令牌"
          name="context-lifecycle-read-bearer-token"
          onChange={(event) => setBearerToken(event.target.value)}
          type="password"
          value={bearerToken}
        />
        <Button
          disabled={!canInspect}
          icon={<Eye aria-hidden="true" />}
          tone="muted"
          type="button"
          onClick={inspectLifecycleState}
        >
          {isLoading ? "Inspecting... / 正在审阅..." : "Inspect lifecycle state / 审阅生命周期状态"}
        </Button>
      </div>
      <ContextLifecycleReadInspectorScreen resource={currentResource} />
    </section>
  );
}

export function ContextLifecycleReadInspectorScreen({
  resource
}: ContextLifecycleReadInspectorScreenProps) {
  const view = presentContextLifecycleReadInspector(resource);

  return (
    <section
      aria-labelledby="context-lifecycle-read-inspector-state-heading"
      className="context-lifecycle-read-inspector"
      data-state={view.status.state}
      id="context-lifecycle-read-inspector"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <h4 id="context-lifecycle-read-inspector-state-heading">{view.title}</h4>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">exact scope / 精确范围</StatusPill>
      </div>

      <CapabilityStateScreen view={view.status} />

      <DefinitionGrid
        aria-label="Context lifecycle read scope / Context 生命周期读取范围"
        columns={2}
        compact
        items={view.scope.map((item) => ({
          id: item.id,
          label: item.label,
          value: <CodeChip>{item.value}</CodeChip>
        }))}
        surface="raised"
        valueTone="info"
      />

      {view.status.state === "available" ? (
        <>
          <div className="operation-block">
            <div className="block-heading">Context metadata / Context 元数据</div>
            {view.metadataText ? (
              <pre className="context-lifecycle-read-inspector__code">{view.metadataText}</pre>
            ) : (
              <p role="status">No Context metadata / 暂无 Context 元数据。</p>
            )}
          </div>

          <div className="operation-block">
            <div className="block-heading">Components / 组件</div>
            {view.components.length > 0 ? (
              <StackTable
                aria-label="Context lifecycle components / Context 生命周期组件"
                columnTemplate="minmax(12rem, 0.9fr) minmax(12rem, 1fr) minmax(10rem, 0.8fr) minmax(14rem, 1fr)"
                headers={["Component / 组件", "Content / 正文", "Metadata / 元数据", "Provenance / 来源"]}
                rows={view.components.map((component) => ({
                  id: component.id,
                  cells: [
                    <div key={`${component.id}-identity`}>
                      <StatusPill tone="info">{component.kindLabel}</StatusPill>
                      <strong>{component.name}</strong>
                      <CodeChip as="div">{component.componentId}</CodeChip>
                    </div>,
                    <pre className="context-lifecycle-read-inspector__code" key={`${component.id}-content`}>{component.content}</pre>,
                    <pre className="context-lifecycle-read-inspector__code" key={`${component.id}-metadata`}>{component.metadataText}</pre>,
                    <DefinitionGrid
                      aria-label={`Component provenance ${component.componentId}`}
                      columns={1}
                      compact
                      items={[
                        { id: "content-hash", label: "Content hash / 正文 hash", value: <CodeChip>{component.contentHash}</CodeChip> },
                        { id: "creation-commit", label: "Created / 创建提交", value: <CodeChip>{component.creationCommitId}</CodeChip> },
                        { id: "content-commit", label: "Content commit / 正文提交", value: <CodeChip>{component.contentCommitId}</CodeChip> }
                      ]}
                      surface="raised"
                      valueTone="info"
                      key={`${component.id}-provenance`}
                    />
                  ]
                }))}
              />
            ) : (
              <p role="status">No components in this exact commit / 此精确提交没有组件。</p>
            )}
          </div>

          <div className="operation-block">
            <div className="block-heading">Relationships / 关系</div>
            {view.relationships && view.relationships.rows.length > 0 ? (
              <StackTable
                aria-label="Context lifecycle relationships / Context 生命周期关系"
                headers={["Kind / 类型", "Source / 源", "Target / 目标"]}
                rows={view.relationships.rows.map((row) => ({
                  id: row.id,
                  cells: [row.kindLabel, <CodeChip key={`${row.id}-source`}>{row.sourceLabel}</CodeChip>, <CodeChip key={`${row.id}-target`}>{row.targetLabel}</CodeChip>]
                }))}
              />
            ) : (
              <p role="status">No relationships in this exact commit / 此精确提交没有关系。</p>
            )}
          </div>
        </>
      ) : null}
    </section>
  );
}

function createTarget(contextId: string, commitId: string): ContextLifecycleReadTarget {
  return Object.freeze({ contextId, commitId });
}

function emptyResource(target: ContextLifecycleReadTarget): ContextLifecycleReadResource {
  return { kind: "empty", target };
}

function sameTarget(left: ContextLifecycleReadTarget, right: ContextLifecycleReadTarget): boolean {
  return left.contextId === right.contextId && left.commitId === right.commitId;
}

function lifecycleReadState(error: unknown): "error" | "empty" | "unavailable" {
  if (error instanceof LocalLifecycleProxyError && error.status === 404) {
    return "empty";
  }
  if (error instanceof LocalLifecycleProxyError && error.status === 503) {
    return "unavailable";
  }
  return "error";
}

function lifecycleReadMessage(error: unknown): string | undefined {
  if (error instanceof LocalLifecycleProxyError) {
    return lifecycleErrorMessage(error.status, error.retryAfterMs, error.body.error);
  }
  if (error instanceof TypeError) {
    return "The lifecycle response scope was rejected / 生命周期响应范围已被拒绝。";
  }
  return "Unable to reach the local lifecycle read / 无法连接本地生命周期读取。";
}
