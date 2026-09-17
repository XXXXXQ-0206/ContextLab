"use client";

import {
  Button,
  CodeChip,
  DefinitionGrid,
  Input,
  Select,
  StackTable,
  StatusPill,
  Textarea
} from "@contextlab/ui";
import type { ContextComponentKind, LocalContextLifecycleState } from "@contextlab/local-sdk";
import { useRouter } from "next/navigation";
import React, { useMemo, useRef, useState } from "react";
import {
  LocalLifecycleProxyError,
  loadLocalContextLifecycleState,
  readLocalLifecycleComponentMetadata,
  readLocalLifecycleContextMetadata,
  submitLocalComponentLifecycleCommit
} from "./context-lifecycle-data";
import {
  buildLocalLifecycleCommitRequest,
  canSubmitLocalLifecycle,
  deriveLocalUsesAddCandidates,
  deriveLocalUsesRemoveCandidates,
  lifecycleErrorMessage,
  lifecycleNoticeRole,
  presentContextLifecycleRelationships,
  presentLocalLifecycleMetadataDraft,
  type LifecycleUsesEdge,
  type ContextLifecycleRelationshipSectionModel,
  type LifecycleNoticeTone,
  type LifecycleEditorOperation
} from "./context-lifecycle-presenter";
import type { LocalBranchHeadTarget } from "./local-branch-heads-data";

export type ContextLifecycleEditorCandidate = {
  id: string;
  label: string;
};

export type ContextLifecycleEditorProps = {
  contextId: string;
  candidates: ContextLifecycleEditorCandidate[];
  selectedBranchHeadTarget?: LocalBranchHeadTarget | null;
  onCommitSuccess?: (pair: ContextLifecycleCommitReviewPair | null) => void;
};

export type ContextLifecycleCommitReviewPair = Readonly<{
  originalCommitId: string;
  revisedCommitId: string;
}>;

export type LifecycleIdempotencyKeyState = Readonly<{
  draftVersion: number;
  key: string;
}>;

export type ContextLifecycleEditorControlProps = ContextLifecycleEditorProps & {
  refreshWorkspace: () => void;
};

const componentKinds: Array<{ value: ContextComponentKind; label: string }> = [
  { value: "prompt", label: "Prompt / 提示词" },
  { value: "system_prompt", label: "System prompt / 系统提示词" },
  { value: "memory", label: "Memory / 记忆" },
  { value: "knowledge", label: "Knowledge / 知识" },
  { value: "tool", label: "Tool / 工具" },
  { value: "output_schema", label: "Output schema / 输出模式" },
  { value: "workflow", label: "Workflow / 工作流" }
];

const operationLabels: Record<LifecycleEditorOperation, string> = {
  initialize: "Initialize Context / 初始化 Context",
  update_metadata: "Update Context metadata / 更新 Context 元数据",
  create: "Create component / 创建组件",
  update: "Update content / 更新正文",
  update_descriptor: "Update descriptor / 更新描述符",
  remove: "Remove component / 移除组件",
  add_uses_relationship: "Add Uses relationship / 添加 Uses 关系",
  remove_uses_relationship: "Remove Uses relationship / 移除 Uses 关系"
};

type LifecycleNotice = {
  message: string;
  tone: LifecycleNoticeTone;
};

function ContextLifecycleRelationshipSection({
  model
}: {
  model: ContextLifecycleRelationshipSectionModel;
}) {
  return (
    <section aria-labelledby="context-lifecycle-relationships-heading" className="context-lifecycle-editor__relationships">
      <div className="context-lifecycle-editor__relationships-heading">
        <div>
          <span className="eyebrow">Read-only graph projection / 只读图谱投影</span>
          <h4 id="context-lifecycle-relationships-heading">Relationships / 关系</h4>
        </div>
        <StatusPill tone="info">Exact commit / 精确提交</StatusPill>
      </div>
      <DefinitionGrid
        aria-label="Exact lifecycle relationship commit / 精确生命周期关系提交"
        columns={1}
        compact
        items={[{ id: "relationship-commit", label: "Commit / 提交", value: model.commitId }]}
        surface="raised"
        valueTone="info"
      />
      {model.rows.length > 0 ? (
        <StackTable
          aria-label="Relationships in exact commit / 精确提交中的关系"
          columnTemplate="minmax(8rem, 0.8fr) minmax(12rem, 1.2fr) minmax(12rem, 1.2fr)"
          headers={["Relationship / 关系", "Source / 源", "Target / 目标"]}
          rows={model.rows.map((row) => ({
            id: row.id,
            cells: [
              row.kindLabel,
              `${row.sourceLabel} · ${row.sourceId}`,
              `${row.targetLabel} · ${row.targetId}`
            ]
          }))}
        />
      ) : (
        <p role="status">No relationships in this exact commit / 此精确提交没有关系。</p>
      )}
    </section>
  );
}

export function ContextLifecycleEditor({
  candidates,
  contextId,
  onCommitSuccess,
  selectedBranchHeadTarget
}: ContextLifecycleEditorProps) {
  const router = useRouter();

  return (
    <ContextLifecycleEditorControl
      candidates={candidates}
      contextId={contextId}
      onCommitSuccess={onCommitSuccess}
      refreshWorkspace={() => router.refresh()}
      selectedBranchHeadTarget={selectedBranchHeadTarget}
    />
  );
}

export function ContextLifecycleEditorControl({
  candidates,
  contextId,
  onCommitSuccess,
  refreshWorkspace,
  selectedBranchHeadTarget
}: ContextLifecycleEditorControlProps) {
  const initialCommitId = selectedBranchHeadTarget === undefined
    ? candidates[0]?.id ?? ""
    : selectedBranchHeadTarget?.head_commit_id ?? "";
  const [bearerToken, setBearerToken] = useState("");
  const [selectedCommitId, setSelectedCommitId] = useState(initialCommitId);
  const [branchName, setBranchName] = useState(selectedBranchHeadTarget?.branch_name ?? "main");
  const [message, setMessage] = useState("Create Context component");
  const [operation, setOperation] = useState<LifecycleEditorOperation>("create");
  const [componentKind, setComponentKind] = useState<ContextComponentKind>("prompt");
  const [componentId, setComponentId] = useState("");
  const [targetComponentId, setTargetComponentId] = useState("");
  const [name, setName] = useState("Instruction");
  const [metadataText, setMetadataText] = useState("{}");
  const [content, setContent] = useState("");
  const [removeConfirmed, setRemoveConfirmed] = useState(false);
  const [lifecycleState, setLifecycleState] = useState<LocalContextLifecycleState | null>(null);
  const [notice, setNotice] = useState<LifecycleNotice | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const draftVersionRef = useRef(0);
  const idempotencyKeyStateRef = useRef<LifecycleIdempotencyKeyState | null>(null);
  const selectedCommitIdRef = useRef(initialCommitId);
  const stateRequestSequenceRef = useRef(0);

  function setDraftValue<T>(setter: React.Dispatch<React.SetStateAction<T>>, value: T) {
    draftVersionRef.current += 1;
    setter(value);
  }

  const selectedComponent = useMemo(
    () => lifecycleState?.components.find((component) => component.component_id === componentId) ?? null,
    [componentId, lifecycleState]
  );
  const selectedTargetComponent = useMemo(
    () => lifecycleState?.components.find((component) => component.component_id === targetComponentId) ?? null,
    [targetComponentId, lifecycleState]
  );
  const usesEdges = useMemo(
    () => deriveLocalUsesRemoveCandidates(lifecycleState),
    [lifecycleState]
  );
  const relationshipSection = useMemo(
    () => presentContextLifecycleRelationships(lifecycleState),
    [lifecycleState]
  );
  const relationshipSourceIds = useMemo(() => {
    if (!lifecycleState) {
      return [];
    }

    const sourceIds = operation === "remove_uses_relationship"
      ? usesEdges.map((edge) => edge.sourceComponentId)
      : lifecycleState.components
        .map((component) => component.component_id)
        .filter((sourceId) => deriveLocalUsesAddCandidates(lifecycleState, sourceId).length > 0);
    return [...new Set(sourceIds)].sort(compareIds);
  }, [lifecycleState, operation, usesEdges]);
  const relationshipTargetIds = useMemo(() => {
    if (!lifecycleState || !componentId) {
      return [];
    }

    if (operation === "remove_uses_relationship") {
      return usesEdges
        .filter((edge) => edge.sourceComponentId === componentId)
        .map((edge) => edge.targetComponentId)
        .sort(compareIds);
    }

    return deriveLocalUsesAddCandidates(lifecycleState, componentId);
  }, [componentId, lifecycleState, operation, usesEdges]);
  const commandDraft = buildLocalLifecycleCommitRequest({
    branchName,
    expectedHeadCommitId: selectedCommitId,
    message,
    operation,
    componentKind,
    componentId,
    targetComponentId,
    name,
    metadataText,
    content,
    removeConfirmed
  });
  const canLoad = canSubmitLocalLifecycle({
    bearerToken,
    selectedCommitId,
    isCommandValid: true,
    isLoading,
    isSubmitting
  });
  const canSubmit = canSubmitLocalLifecycle({
    bearerToken,
    selectedCommitId,
    allowsUnbornHead: operation === "initialize",
    isCommandValid: commandDraft.ok,
    isLoading,
    isSubmitting
  });

  async function loadSelectedState() {
    if (!canLoad) {
      return;
    }

    const requestedCommitId = selectedCommitIdRef.current;
    const requestSequence = stateRequestSequenceRef.current + 1;
    stateRequestSequenceRef.current = requestSequence;
    setIsLoading(true);
    setNotice(null);
    try {
      const state = await loadLocalContextLifecycleState(contextId, requestedCommitId, bearerToken);
      if (
        stateRequestSequenceRef.current !== requestSequence
        || selectedCommitIdRef.current !== requestedCommitId
        || state.commit_id !== requestedCommitId
      ) {
        return;
      }
      setLifecycleState(state);
      if (isUsesOperation(operation)) {
        const selection = defaultUsesSelection(state, operation);
        setComponentId(selection.sourceComponentId);
        setTargetComponentId(selection.targetComponentId);
      } else {
        const firstComponentId = state.components[0]?.component_id ?? "";
        setComponentId((current) =>
          state.components.some((component) => component.component_id === current) ? current : firstComponentId
        );
        setTargetComponentId((current) =>
          state.components.some((component) => component.component_id === current) ? current : firstComponentId
        );
      }
    } catch (error) {
      if (
        stateRequestSequenceRef.current !== requestSequence
        || selectedCommitIdRef.current !== requestedCommitId
      ) {
        return;
      }
      setLifecycleState(null);
      setNotice({ message: presentLifecycleError(error), tone: "error" });
    } finally {
      if (stateRequestSequenceRef.current === requestSequence) {
        setIsLoading(false);
      }
    }
  }

  function readSelectedMetadata() {
    const metadata = readLocalLifecycleComponentMetadata(lifecycleState, componentId);
    if (!metadata) {
      setNotice({ message: "Load and select a component first / 请先加载并选择组件。", tone: "error" });
      return;
    }

    const draft = presentLocalLifecycleMetadataDraft(metadata);
    setDraftValue(setName, draft.name);
    setDraftValue(setMetadataText, draft.metadataText);
    setNotice({ message: "Loaded current metadata / 已读取当前元数据。", tone: "success" });
  }

  function readContextMetadata() {
    const metadata = readLocalLifecycleContextMetadata(lifecycleState);
    if (!metadata) {
      setNotice({ message: "Load a commit with Context metadata first / 请先加载包含 Context 元数据的 commit。", tone: "error" });
      return;
    }

    setDraftValue(setMetadataText, JSON.stringify(metadata, null, 2));
    setNotice({ message: "Loaded current Context metadata / 已读取当前 Context 元数据。", tone: "success" });
  }

  async function submitLifecycleChange(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canSubmit) {
      return;
    }

    if (!commandDraft.ok) {
      setNotice({ message: commandDraft.message, tone: "error" });
      return;
    }

    setIsSubmitting(true);
    setNotice(null);
    onCommitSuccess?.(null);
    const submissionCommitId = selectedCommitIdRef.current;
    const submissionSequence = stateRequestSequenceRef.current + 1;
    stateRequestSequenceRef.current = submissionSequence;
    const idempotencyKeyState = resolveLifecycleIdempotencyKey(
      idempotencyKeyStateRef.current,
      draftVersionRef.current
    );
    idempotencyKeyStateRef.current = idempotencyKeyState;
    try {
      const result = await submitLocalComponentLifecycleCommit(
        contextId,
        bearerToken,
        idempotencyKeyState.key,
        commandDraft.value
      );
      idempotencyKeyStateRef.current = null;
      onCommitSuccess?.(createLifecycleCommitReviewPair(selectedCommitId, result.commit_id));
      const refreshed = await refreshAfterLifecycleCommit(
        () => loadLocalContextLifecycleState(contextId, result.commit_id, bearerToken),
        refreshWorkspace
      );
      if (
        stateRequestSequenceRef.current !== submissionSequence
        || selectedCommitIdRef.current !== submissionCommitId
      ) {
        return;
      }
      selectedCommitIdRef.current = result.commit_id;
      setSelectedCommitId(result.commit_id);
      if (refreshed.ok) {
        setLifecycleState(refreshed.value);
        if (isUsesOperation(operation)) {
          const selection = defaultUsesSelection(refreshed.value, operation);
          setComponentId(selection.sourceComponentId);
          setTargetComponentId(selection.targetComponentId);
        }
        setNotice({
          message: `Committed ${result.commit_id}; graph review refreshed / 已提交 ${result.commit_id}；图谱审阅已刷新。`,
          tone: "success"
        });
      } else {
        setLifecycleState(null);
        setNotice({
          message: `Committed ${result.commit_id}, but the new state could not be reloaded / 已提交 ${result.commit_id}，但无法重读新状态。`,
          tone: "error"
        });
      }
    } catch (error) {
      setNotice({ message: presentLifecycleError(error), tone: "error" });
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <section className="context-lifecycle-editor" aria-labelledby="context-lifecycle-editor-heading">
      <div className="context-lifecycle-editor__heading">
        <div>
          <span className="eyebrow">Local Context Lifecycle / 本地 Context 生命周期</span>
          <h3 id="context-lifecycle-editor-heading">Local Context Lifecycle</h3>
          <p>
            Protected local workflow / 仅本地受保护工作流。No credential is persisted / 不会持久化凭据。
          </p>
        </div>
        <StatusPill tone="warning">local only / 仅本地</StatusPill>
      </div>

      <form className="context-lifecycle-editor__form" onSubmit={submitLifecycleChange}>
        <div className="context-lifecycle-editor__controls">
          <Input
            autoComplete="off"
            description="Memory only / 仅内存"
            label="Bearer token / 访问令牌"
            name="lifecycle-bearer-token"
            onChange={(event) => setDraftValue(setBearerToken, event.target.value)}
            type="password"
            value={bearerToken}
          />
          <Select
            label="Materialized head / 已物化 head"
            name="lifecycle-head"
            onChange={(event) => {
              const nextCommitId = event.target.value;
              selectedCommitIdRef.current = nextCommitId;
              stateRequestSequenceRef.current += 1;
              setIsLoading(false);
              setLifecycleState(null);
              setDraftValue(setSelectedCommitId, nextCommitId);
            }}
            value={selectedCommitId}
          >
            {selectedBranchHeadTarget === null ? (
              <option value="">No selected server-owned head / 没有选定的 server-owned head</option>
            ) : null}
            {selectedBranchHeadTarget && !candidates.some((candidate) => candidate.id === selectedBranchHeadTarget.head_commit_id) ? (
              <option value={selectedBranchHeadTarget.head_commit_id}>
                {selectedBranchHeadTarget.branch_name} server-owned head / server-owned 分支 head
              </option>
            ) : null}
            {candidates.length === 0 && !selectedBranchHeadTarget ? <option value="">No materialized commits / 暂无已物化提交</option> : null}
            {candidates.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>
                {candidate.label}
              </option>
            ))}
          </Select>
          <Button disabled={!canLoad} tone="muted" type="button" onClick={loadSelectedState}>
            {isLoading ? "Loading state... / 正在加载状态..." : "Load state / 加载状态"}
          </Button>
        </div>

        {lifecycleState ? (
          <DefinitionGrid
            aria-label="Loaded local Context lifecycle state"
            columns={3}
            compact
            items={[
              { id: "loaded-commit", label: "Commit / 提交", value: lifecycleState.commit_id },
              { id: "loaded-components", label: "Components / 组件", value: String(lifecycleState.components.length) },
              { id: "loaded-schema", label: "Graph schema / 图谱模式", value: String(lifecycleState.graph_snapshot.schema_version) }
            ]}
            surface="raised"
            valueTone="info"
          />
        ) : null}

        {relationshipSection ? <ContextLifecycleRelationshipSection model={relationshipSection} /> : null}

        <div className="context-lifecycle-editor__controls">
          <Select
            label="Operation / 操作"
            name="lifecycle-operation"
            onChange={(event) => {
              const nextOperation = event.target.value as LifecycleEditorOperation;
              setDraftValue(setOperation, nextOperation);
              setDraftValue<boolean>(setRemoveConfirmed, false);
              if (isUsesOperation(nextOperation) && lifecycleState) {
                const selection = defaultUsesSelection(lifecycleState, nextOperation);
                setDraftValue(setComponentId, selection.sourceComponentId);
                setDraftValue(setTargetComponentId, selection.targetComponentId);
              }
            }}
            value={operation}
          >
            <option value="create">Create component / 创建组件</option>
            <option value="initialize">Initialize Context / 初始化 Context</option>
            <option value="update_metadata">Update Context metadata / 更新 Context 元数据</option>
            <option value="update">Update content / 更新正文</option>
            <option value="update_descriptor">Update descriptor / 更新描述符</option>
            <option value="remove">Remove component / 移除组件</option>
            <option value="add_uses_relationship">Add Uses relationship / 添加 Uses 关系</option>
            <option value="remove_uses_relationship">Remove Uses relationship / 移除 Uses 关系</option>
          </Select>
          <Input
            description={selectedBranchHeadTarget ? "Server-owned branch-head target / server-owned 分支 head target" : undefined}
            label="Branch / 分支"
            name="lifecycle-branch"
            onChange={(event) => setDraftValue(setBranchName, event.target.value)}
            readOnly={selectedBranchHeadTarget !== undefined && selectedBranchHeadTarget !== null}
            value={branchName}
          />
          <Input label="Commit message / 提交说明" name="lifecycle-message" onChange={(event) => setDraftValue(setMessage, event.target.value)} value={message} />
        </div>

        {operation === "create" ? (
          <div className="context-lifecycle-editor__controls">
            <Select
              label="Component kind / 组件类型"
              name="lifecycle-component-kind"
              onChange={(event) => setDraftValue(setComponentKind, event.target.value as ContextComponentKind)}
              value={componentKind}
            >
              {componentKinds.map((kind) => (
                <option key={kind.value} value={kind.value}>
                  {kind.label}
                </option>
              ))}
            </Select>
            <Input label="Component name / 组件名称" name="lifecycle-component-name" onChange={(event) => setDraftValue(setName, event.target.value)} value={name} />
            <Textarea
              label="Metadata JSON / 元数据 JSON"
              name="lifecycle-metadata"
              onChange={(event) => setDraftValue(setMetadataText, event.target.value)}
              rows={3}
              value={metadataText}
            />
          </div>
        ) : operation === "initialize" || operation === "update_metadata" ? null : isUsesOperation(operation) ? (
          <div className="context-lifecycle-editor__controls">
            <Select
              description={selectedComponent ? `Source hash / 源 hash: ${selectedComponent.content_hash}` : "Graph-backed pair / 基于图谱的关系对"}
              label="Source component / 源组件"
              name="lifecycle-uses-source-component-id"
              onChange={(event) => {
                const nextSourceId = event.target.value;
                setDraftValue(setComponentId, nextSourceId);
                const nextTargetId = operation === "remove_uses_relationship"
                  ? usesEdges
                    .filter((edge) => edge.sourceComponentId === nextSourceId)
                    .map((edge) => edge.targetComponentId)[0] ?? ""
                  : deriveLocalUsesAddCandidates(lifecycleState, nextSourceId)[0] ?? "";
                setDraftValue(setTargetComponentId, nextTargetId);
              }}
              value={componentId}
            >
              <option value="">Select source component / 选择源组件</option>
              {relationshipSourceIds.map((sourceId) => {
                const component = lifecycleState?.components.find((item) => item.component_id === sourceId);
                return component ? (
                  <option key={component.component_id} value={component.component_id}>
                    {component.name} ({component.component_kind})
                  </option>
                ) : null;
              })}
            </Select>
            <Select
              description={selectedTargetComponent ? `Target hash / 目标 hash: ${selectedTargetComponent.content_hash}` : "Graph-backed target / 基于图谱的目标"}
              label="Target component / 目标组件"
              name="lifecycle-uses-target-component-id"
              onChange={(event) => setDraftValue(setTargetComponentId, event.target.value)}
              value={targetComponentId}
            >
              <option value="">Select target component / 选择目标组件</option>
              {relationshipTargetIds.map((targetId) => {
                const component = lifecycleState?.components.find((item) => item.component_id === targetId);
                return component ? (
                  <option key={component.component_id} value={component.component_id}>
                    {component.name} ({component.component_kind})
                  </option>
                ) : null;
              })}
            </Select>
          </div>
        ) : (
          <div className="context-lifecycle-editor__target">
            <Select
              description={selectedComponent ? `Current hash / 当前 hash: ${selectedComponent.content_hash}` : undefined}
              label="Component / 组件"
              name="lifecycle-component-id"
              onChange={(event) => setDraftValue(setComponentId, event.target.value)}
              value={componentId}
            >
              <option value="">Select loaded component / 选择已加载组件</option>
              {lifecycleState?.components.map((component) => (
                <option key={component.component_id} value={component.component_id}>
                  {component.name} ({component.component_kind})
                </option>
              ))}
            </Select>
            {selectedComponent ? <CodeChip>{selectedComponent.content_hash}</CodeChip> : null}
          </div>
        )}

        {operation === "update_descriptor" ? (
          <div className="context-lifecycle-editor__controls">
            <Input
              label="Component name / 组件名称"
              name="lifecycle-descriptor-name"
              onChange={(event) => setDraftValue(setName, event.target.value)}
              value={name}
            />
            <Textarea
              label="Metadata JSON / 元数据 JSON"
              name="lifecycle-descriptor-metadata"
              onChange={(event) => setDraftValue(setMetadataText, event.target.value)}
              rows={3}
              value={metadataText}
            />
            <Button
              disabled={!selectedComponent || isLoading || isSubmitting}
              tone="muted"
              type="button"
              onClick={readSelectedMetadata}
            >
              Read current metadata / 读取当前元数据
            </Button>
          </div>
        ) : operation === "update_metadata" ? (
          <div className="context-lifecycle-editor__controls">
            <Textarea
              label="Context metadata JSON / Context 元数据 JSON"
              name="lifecycle-context-metadata"
              onChange={(event) => setDraftValue(setMetadataText, event.target.value)}
              rows={5}
              value={metadataText}
            />
            <Button
              disabled={!lifecycleState?.metadata || isLoading || isSubmitting}
              tone="muted"
              type="button"
              onClick={readContextMetadata}
            >
              Read current Context metadata / 读取当前 Context 元数据
            </Button>
          </div>
        ) : null}

        {operation === "create" || operation === "update" ? (
          <Textarea
            label="Component content / 组件正文"
            name="lifecycle-content"
            onChange={(event) => setDraftValue(setContent, event.target.value)}
            rows={7}
            value={content}
          />
        ) : null}

        {operation === "remove" ? (
          <label className="context-lifecycle-editor__confirmation">
            <input
              checked={removeConfirmed}
              disabled={!selectedComponent}
              name="lifecycle-remove-confirmation"
              onChange={(event) => setDraftValue(setRemoveConfirmed, event.target.checked)}
              type="checkbox"
            />
            <span>
              Confirm removal of {selectedComponent?.name ?? "the selected component"} / 确认移除
              {selectedComponent ? ` ${selectedComponent.name}` : "选中的组件"}
            </span>
          </label>
        ) : null}

        {notice ? (
          <p
            className="context-lifecycle-editor__notice"
            data-tone={notice.tone}
            role={lifecycleNoticeRole(notice.tone)}
          >
            {notice.message}
          </p>
        ) : null}

        <div className="context-lifecycle-editor__submit">
          <Button disabled={!canSubmit} tone={operation === "remove" ? "danger" : "solid"} type="submit">
            {isSubmitting ? "Committing... / 正在提交..." : operationLabels[operation]}
          </Button>
          <span>Every change uses the guarded writer / 每次变更均使用 guarded writer。</span>
        </div>
      </form>
    </section>
  );
}

export function presentLifecycleError(error: unknown): string {
  if (error instanceof LocalLifecycleProxyError) {
    return lifecycleErrorMessage(error.status, error.retryAfterMs, error.body.error);
  }

  return "Unable to reach the local lifecycle workflow / 无法连接本地生命周期工作流。";
}

function createIdempotencyKey(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }

  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export function resolveLifecycleIdempotencyKey(
  previous: LifecycleIdempotencyKeyState | null,
  draftVersion: number,
  createKey: () => string = createIdempotencyKey
): LifecycleIdempotencyKeyState {
  if (previous?.draftVersion === draftVersion) {
    return previous;
  }

  return Object.freeze({ draftVersion, key: createKey() });
}

export async function refreshAfterLifecycleCommit<T>(
  loadCommittedState: () => Promise<T>,
  refresh: () => void
): Promise<{ ok: true; value: T } | { ok: false; error: unknown }> {
  try {
    return { ok: true, value: await loadCommittedState() };
  } catch (error) {
    return { ok: false, error };
  } finally {
    refresh();
  }
}

export function createLifecycleCommitReviewPair(
  originalCommitId: string,
  revisedCommitId: string
): ContextLifecycleCommitReviewPair | null {
  const original = originalCommitId.trim();
  const revised = revisedCommitId.trim();
  if (!original || !revised || original === revised) {
    return null;
  }

  return Object.freeze({
    originalCommitId: original,
    revisedCommitId: revised
  });
}

function isUsesOperation(
  operation: LifecycleEditorOperation
): operation is "add_uses_relationship" | "remove_uses_relationship" {
  return operation === "add_uses_relationship" || operation === "remove_uses_relationship";
}

function defaultUsesSelection(
  state: LocalContextLifecycleState,
  operation: "add_uses_relationship" | "remove_uses_relationship"
): LifecycleUsesEdge {
  const edges = operation === "remove_uses_relationship"
    ? deriveLocalUsesRemoveCandidates(state)
    : state.components
      .map((component) => component.component_id)
      .sort(compareIds)
      .flatMap((sourceComponentId) => deriveLocalUsesAddCandidates(state, sourceComponentId)
        .map((targetComponentId) => ({ sourceComponentId, targetComponentId })))
      .sort(compareUsesEdges);
  return edges[0] ?? { sourceComponentId: "", targetComponentId: "" };
}

function compareUsesEdges(left: LifecycleUsesEdge, right: LifecycleUsesEdge): number {
  return compareIds(left.sourceComponentId, right.sourceComponentId)
    || compareIds(left.targetComponentId, right.targetComponentId);
}

function compareIds(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
