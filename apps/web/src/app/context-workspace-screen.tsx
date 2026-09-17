import {
  Button,
  CodeChip,
  DefinitionGrid,
  Panel,
  PanelHeader,
  StackTable,
  StatusPill
} from "@contextlab/ui";
import {
  Activity,
  ArrowUpRight,
  Boxes,
  Braces,
  Clock3,
  Database,
  Fingerprint,
  GitBranch,
  GitCommitHorizontal,
  GitCompareArrows,
  LineChart,
  Network,
  Play,
  Search,
  Settings,
  Sparkles
} from "lucide-react";
import { ContextGraphInspector } from "./context-graph-inspector";
import { ContextLifecycleGraphReviewBridge } from "./context-lifecycle-graph-review-bridge";
import { ContextLifecycleReadInspector } from "./context-lifecycle-read-inspector";
import { LocalBenchmarkWorkspaceInspector } from "./local-benchmark-workspace-inspector";
import { LocalBenchmarkDefinitionAuthoringEditor } from "./local-benchmark-definition-authoring-screen";
import { LocalBenchmarkDefinitionBindingInspectionEditor } from "./local-benchmark-definition-binding-inspection-screen";
import { LocalBenchmarkExecutionInspector } from "./local-benchmark-execution-inspector";
import { LocalWorkflowCapabilityInspector } from "./local-workflow-capability-inspector";
import { LocalWorkflowContextBindingsInspector } from "./local-workflow-context-bindings-inspector";
import { LocalKnowledgeMemoryProjectionInspector } from "./local-knowledge-memory-projection-inspector";
import { LocalContextMergeReviewInspector } from "./local-context-merge-review-inspector";
import { LocalPersistedContextDiffReviewInspector } from "./local-persisted-context-diff-review-inspector";
import type { ContextWorkspaceScreenModel } from "./context-workspace-presenter";

function WorkspaceSidebar({ source }: Pick<ContextWorkspaceScreenModel, "source">) {
  return (
    <aside className="sidebar" aria-label="Workspace navigation">
      <div className="brand-mark">
        <span className="brand-glyph">CL</span>
        <div>
          <div className="brand-title">ContextLab</div>
          <div className="brand-subtitle">Context Engineering / 上下文工程</div>
        </div>
      </div>

      <nav className="nav-stack">
        <a className="nav-item" data-active="true" href="#workspace">
          <Boxes />
          Workspace
        </a>
        <a className="nav-item" href="#graph">
          <Network />
          Context Graph
        </a>
        <a className="nav-item" href="#history">
          <GitCommitHorizontal />
          Version History
        </a>
        <a className="nav-item" href="#evaluation">
          <LineChart />
          Evaluation
        </a>
        <a className="nav-item" href="#settings">
          <Settings />
          Settings
        </a>
      </nav>

      <div className="sidebar-status" aria-label="Data source mode">
        <div className="source-current">
          <StatusPill tone={source.tone}>{source.label}</StatusPill>
          <p>{source.description}</p>
        </div>
        <ul className="source-mode-list" aria-label="Data source mode states">
          {source.modes.map((mode) => (
            <li className="source-mode" data-active={mode.active} key={mode.id}>
              <span className="source-mode__indicator" data-tone={mode.tone} aria-hidden="true" />
              <span className="source-mode__body">
                <span className="source-mode__label">{mode.label}</span>
                <span className="source-mode__description">{mode.description}</span>
              </span>
              <span className="source-mode__status">{mode.statusLabel}</span>
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
}

function WorkspaceHeader({ title }: { title: string }) {
  return (
    <header className="topbar">
      <div className="workspace-heading">
        <span className="eyebrow">Default Workspace / 默认工作区</span>
        <h1 className="workspace-title">{title}</h1>
        <p className="workspace-kicker">
          Context graph, commit history, component fingerprints, and evaluation runs now share one
          operational surface. 上下文图谱、版本历史、组件指纹与评测运行在同一个工作台里协同检查。
        </p>
      </div>
      <div className="topbar-actions">
        <Button tone="muted" icon={<Search aria-hidden="true" />}>
          Search
        </Button>
        <Button tone="muted" icon={<GitBranch aria-hidden="true" />}>
          Branch
        </Button>
        <Button icon={<Play aria-hidden="true" />}>Run Eval</Button>
      </div>
    </header>
  );
}

function ContextDetailPanel({ workspace }: Pick<ContextWorkspaceScreenModel, "workspace">) {
  return (
    <Panel as="section" className="context-panel" aria-labelledby="context-heading">
      <PanelHeader actions={<StatusPill tone="success">Evaluated</StatusPill>}>
        <h2 className="panel-title" id="context-heading">
          Context Detail
        </h2>
        <p className="panel-caption">Production unit / 生产上下文单元</p>
      </PanelHeader>

      <DefinitionGrid items={workspace.contextFacts} />

      <p className="context-description">{workspace.description}</p>

      <DefinitionGrid
        aria-label="Discovery routes"
        columns={1}
        compact
        items={workspace.discoveryRoutes}
        valueTone="info"
      />
    </Panel>
  );
}

function OperationsPanel({
  operations,
  localLifecycleEnabled
}: Pick<ContextWorkspaceScreenModel, "operations"> & { localLifecycleEnabled: boolean }) {
  const componentRows =
    operations.componentInventory.length > 0
      ? operations.componentInventory.map((component) => ({
          id: component.id,
          cells: [
            <div className="component-summary" key={`${component.id}-summary`}>
              <div className="component-summary__line">
                <StatusPill tone={component.tone}>{component.kindLabel}</StatusPill>
                <span className="component-name">{component.name}</span>
              </div>
              <span className="component-created">{component.createdAt}</span>
            </div>,
            <CodeChip key={`${component.id}-hash`}>{component.hash}</CodeChip>
          ]
        }))
      : [
          {
            id: "no-components",
            cells: ["No components", "暂无组件"]
          }
        ];
  const evaluationRows =
    operations.evaluationRuns.length > 0
      ? operations.evaluationRuns.map((run) => ({
          id: run.id,
          cells: [run.suiteName, run.modelVersion, run.metricsSummary]
        }))
      : [
          {
            id: "no-evaluation-runs",
            cells: ["No evaluation runs", "暂无评测运行", "0 metrics"]
          }
        ];
  const commitChangeRows = operations.commitDetail.changeItems.map((change) => ({
    id: change.id,
    cells: [
      <div className="commit-change__summary" key={`${change.id}-summary`}>
        <div className="commit-change__line">
          <StatusPill tone="info">{change.operation}</StatusPill>
          <span className="commit-change__target">{change.target}</span>
        </div>
        <span className="commit-change__summary-text">{change.summary}</span>
      </div>,
      <DefinitionGrid
        aria-label={`Commit change payload ${change.id}`}
        columns={1}
        compact
        items={change.payloadItems}
        key={`${change.id}-payload`}
        surface="raised"
        valueTone="info"
      />
    ]
  }));

  return (
    <Panel as="section" className="operations-panel" aria-labelledby="operations-heading">
      <PanelHeader actions={<Activity aria-hidden="true" size={18} />}>
        <h2 className="panel-title" id="operations-heading">
          Operations
        </h2>
        <p className="panel-caption">History and benchmark readiness</p>
      </PanelHeader>

      <div className="operation-block" id="history">
        <div className="block-heading">
          <GitCommitHorizontal aria-hidden="true" />
          <span>Commit History / 版本历史</span>
        </div>
        <div className="timeline">
          {operations.commits.map((commit) => (
            <article className="timeline-row" key={commit.id}>
              <div className="timeline-marker" aria-hidden="true" />
              <div className="timeline-body">
                <div className="row-title">
                  <span>{commit.message}</span>
                  <StatusPill tone="info">{commit.branchName}</StatusPill>
                </div>
                <div className="row-meta">
                  <Clock3 aria-hidden="true" />
                  {commit.authoredAt}
                  <span>{commit.changeCount} change</span>
                  <span>{commit.parentCount} parent</span>
                </div>
              </div>
            </article>
          ))}
        </div>
      </div>

      <div className="operation-block commit-detail" id="commit-detail">
        <div className="block-heading">
          <GitCompareArrows aria-hidden="true" />
          <span>Commit Detail / 提交详情</span>
          <StatusPill tone="info">changes JSON / 变更 JSON</StatusPill>
        </div>
        <div className="commit-detail__summary">
          <div className="commit-detail__title">
            <StatusPill tone="info">{operations.commitDetail.branchName}</StatusPill>
            <strong>{operations.commitDetail.message}</strong>
          </div>
          <CodeChip>{operations.commitDetail.commitId}</CodeChip>
        </div>
        <DefinitionGrid items={operations.commitDetail.facts} />
        <StackTable
          aria-label="Selected commit changes"
          columnTemplate="minmax(0, 0.8fr) minmax(0, 1.2fr)"
          headers={["Change", "Payload"]}
          rows={commitChangeRows}
        />
        <p className="commit-detail__note">
          Detail reads expose ordered change payloads for replay and review; semantic diff remains a
          future workflow layered on this version history. 详情读取用于复现与审查 ordered change payload；semantic
          diff 仍是后续叠加在版本历史上的 workflow。
        </p>
      </div>

      <div className="operation-block" id="components">
        <div className="block-heading">
          <Fingerprint aria-hidden="true" />
          <span>Component Inventory / 组件清单</span>
          <StatusPill tone="info">{operations.componentCount} items</StatusPill>
        </div>
        <StackTable
          aria-label="Context components"
          headers={["Component", "Fingerprint"]}
          rows={componentRows}
        />
      </div>

      <div className="operation-block component-detail" id="component-detail">
        <div className="block-heading">
          <Braces aria-hidden="true" />
          <span>Component Detail / 组件详情</span>
          <StatusPill tone="warning">metadata only / 仅元数据</StatusPill>
        </div>
        <div className="component-detail__summary">
          <div className="component-detail__title">
            <StatusPill tone={operations.componentDetail.tone}>
              {operations.componentDetail.kindLabel}
            </StatusPill>
            <strong>{operations.componentDetail.name}</strong>
          </div>
          <CodeChip>{operations.componentDetail.hash}</CodeChip>
        </div>
        <DefinitionGrid items={operations.componentDetail.facts} />
        <DefinitionGrid
          aria-label="Selected component metadata"
          compact
          items={operations.componentDetail.metadataItems}
          surface="raised"
          valueTone="info"
        />
        <p className="component-detail__note">
          Public detail reads expose metadata and fingerprints for review. Effective body content is
          available only through the protected local lifecycle workflow at a selected commit. Public
          detail read 用于审查 metadata 与 fingerprint；有效正文仅可经选定 commit 的 protected local lifecycle
          workflow 获取。
        </p>
      </div>

      <div className="operation-block" id="evaluation">
        <div className="block-heading">
          <LineChart aria-hidden="true" />
          <span>Evaluation Runs / 评测运行</span>
        </div>
        <StackTable aria-label="Evaluation runs" headers={["Suite", "Model", "Metrics"]} rows={evaluationRows} />
      </div>

      <div className="operation-block evaluation-detail" id="evaluation-detail">
        <div className="block-heading">
          <LineChart aria-hidden="true" />
          <span>Evaluation Run Detail / 评测详情</span>
          <StatusPill tone="info">metrics JSON / 指标 JSON</StatusPill>
        </div>
        {operations.evaluationDetail ? (
          <>
            <div className="evaluation-detail__summary">
              <div className="evaluation-detail__title">
                <StatusPill tone="neutral">Benchmark</StatusPill>
                <strong>{operations.evaluationDetail.suiteName}</strong>
              </div>
              <CodeChip>{operations.evaluationDetail.modelVersion}</CodeChip>
            </div>
            <DefinitionGrid items={operations.evaluationDetail.facts} />
            <DefinitionGrid
              aria-label="Selected evaluation run metrics"
              compact
              items={operations.evaluationDetail.metricItems}
              surface="raised"
              valueTone="info"
            />
            <p className="evaluation-detail__note">
              Detail reads expose persisted metrics for inspection; scorecards now aggregate
              numeric metrics across matching runs, while regression conclusions still need policy
              thresholds. 详情读取用于检查持久化 metrics；scorecard 现在聚合匹配 runs 的 numeric metrics，
              regression conclusion 仍需要 policy thresholds。
            </p>
          </>
        ) : (
          <div className="evaluation-detail__empty">
            <StatusPill tone="neutral">No detail / 暂无详情</StatusPill>
            <p>
              Live data has no selected evaluation metrics detail to inspect, and preview metrics
              are not substituted. 当前 live data 没有可检查的 selected evaluation metrics detail；不会替换为
              preview metrics。
            </p>
          </div>
        )}
      </div>

      {localLifecycleEnabled ? (
        <ContextLifecycleGraphReviewBridge
          candidates={operations.graphReview.candidates}
          contextId={operations.graphReview.contextId}
          localLifecycleEnabled
          review={operations.graphReview}
        />
      ) : (
        <ContextLifecycleGraphReviewBridge
          candidates={operations.graphReview.candidates}
          contextId={operations.graphReview.contextId}
          localLifecycleEnabled={false}
          review={operations.graphReview}
        />
      )}

      <ContextLifecycleReadInspector
        commitId={operations.commitDetail.commitId}
        contextId={operations.graphReview.contextId}
        key={`${operations.graphReview.contextId}:${operations.commitDetail.commitId}`}
      />

      <LocalWorkflowCapabilityInspector contextId={operations.graphReview.contextId} />

      <LocalWorkflowContextBindingsInspector
        commitId={operations.commitDetail.commitId}
        contextId={operations.graphReview.contextId}
        key={`${operations.graphReview.contextId}:${operations.commitDetail.commitId}`}
      />

      <LocalKnowledgeMemoryProjectionInspector
        commitId={operations.commitDetail.commitId}
        contextId={operations.graphReview.contextId}
        projectId={operations.benchmarkEvidence.projectId}
      />

    </Panel>
  );
}

function ContractStrip() {
  return (
    <section className="contract-strip" aria-label="Discovery contract status">
      <div>
        <Database aria-hidden="true" />
        <span>Storage contracts</span>
        <strong>workspace, project, experiment, context, component, commit, evaluation run</strong>
      </div>
      <div>
        <Braces aria-hidden="true" />
        <span>REST first</span>
        <strong>pagination / filtering / sorting</strong>
      </div>
      <div>
        <ArrowUpRight aria-hidden="true" />
        <span>Next</span>
        <strong>diff workflows and regression thresholds</strong>
      </div>
    </section>
  );
}

type ContextWorkspaceScreenProps = ContextWorkspaceScreenModel & {
  localLifecycleEnabled?: boolean;
  mergeReviewEnabled?: boolean;
  persistedDiffReviewEnabled?: boolean;
};

export function ContextWorkspaceScreen({ localLifecycleEnabled = false, mergeReviewEnabled = false, persistedDiffReviewEnabled = false, ...props }: ContextWorkspaceScreenProps) {
  const benchmarkWorkspaceScopeKey = JSON.stringify([
    props.operations.benchmarkEvidence.projectId,
    props.operations.benchmarkEvidence.contextId,
    ...props.operations.benchmarkEvidence.candidates.map((candidate) => candidate.id)
  ]);

  return (
    <main className="app-shell">
      <WorkspaceSidebar source={props.source} />

      <section className="main-surface" id="workspace">
        <WorkspaceHeader title={props.workspace.title} />

        <div className="workspace-grid">
          <ContextDetailPanel workspace={props.workspace} />
          <ContextGraphInspector graph={props.graph} />
          <OperationsPanel
            localLifecycleEnabled={localLifecycleEnabled}
            operations={props.operations}
          />
        </div>

        <section
          className="benchmark-workspace-region"
          aria-label="Local benchmark workspace / 本地 Benchmark 工作台"
        >
          <LocalBenchmarkWorkspaceInspector
            candidates={props.operations.benchmarkEvidence.candidates}
            contextId={props.operations.benchmarkEvidence.contextId}
            key={benchmarkWorkspaceScopeKey}
            projectId={props.operations.benchmarkEvidence.projectId}
          />
          <LocalBenchmarkDefinitionAuthoringEditor
            projectId={props.operations.benchmarkEvidence.projectId}
            contextId={props.operations.benchmarkEvidence.contextId}
            commitId={props.operations.commitDetail.commitId}
            enabled={localLifecycleEnabled}
          />
          <LocalBenchmarkDefinitionBindingInspectionEditor
            projectId={props.operations.benchmarkEvidence.projectId}
            contextId={props.operations.benchmarkEvidence.contextId}
            commitId={props.operations.commitDetail.commitId}
            enabled={localLifecycleEnabled}
          />
          <LocalBenchmarkExecutionInspector
            projectId={props.operations.benchmarkEvidence.projectId}
            contextId={props.operations.benchmarkEvidence.contextId}
            commitId={props.operations.commitDetail.commitId}
            enabled={localLifecycleEnabled}
          />
          <LocalPersistedContextDiffReviewInspector
            candidates={props.operations.graphReview.candidates}
            contextId={props.operations.graphReview.contextId}
            defaultSourceCommitId={props.operations.graphReview.defaultOriginalCommitId}
            defaultTargetCommitId={props.operations.graphReview.defaultRevisedCommitId}
            enabled={persistedDiffReviewEnabled}
            projectId={props.operations.benchmarkEvidence.projectId}
          />
          <LocalContextMergeReviewInspector
            candidates={props.operations.graphReview.candidates}
            contextId={props.operations.graphReview.contextId}
            defaultLeftCommitId={props.operations.graphReview.defaultOriginalCommitId}
            defaultRightCommitId={props.operations.graphReview.defaultRevisedCommitId}
            enabled={mergeReviewEnabled}
            projectId={props.operations.benchmarkEvidence.projectId}
          />
        </section>

        <ContractStrip />
      </section>
    </main>
  );
}
