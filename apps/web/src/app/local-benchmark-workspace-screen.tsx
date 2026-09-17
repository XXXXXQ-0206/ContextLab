import {
  CapabilityState,
  CodeChip,
  DefinitionGrid,
  StackTable,
  StatusPill
} from "@contextlab/ui";
import React from "react";
import type {
  LocalBenchmarkWorkspaceTableRow,
  LocalBenchmarkWorkspaceViewModel
} from "./local-benchmark-workspace-presenter";

export type LocalBenchmarkWorkspaceScreenProps = Readonly<{
  view: LocalBenchmarkWorkspaceViewModel;
  controls?: React.ReactNode;
}>;

export function LocalBenchmarkWorkspaceScreen({ controls, view }: LocalBenchmarkWorkspaceScreenProps) {
  const projection = view.projection;

  return (
    <section
      aria-labelledby="local-benchmark-workspace-heading"
      className="operation-block local-benchmark-workspace"
      id="local-benchmark-workspace"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Benchmark operations / Benchmark 操作</span>
          <h3 id="local-benchmark-workspace-heading">{view.title}</h3>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">protected local read / 受保护本地读取</StatusPill>
      </div>

      {controls}

      <CapabilityState
        ariaLabel={view.status.ariaLabel}
        description={view.status.description}
        detail={view.status.detail}
        label={view.status.label}
        state={view.status.state}
        stateLabel={view.status.stateLabel}
      />

      {projection ? (
        <div className="local-benchmark-workspace__projection">
          <DefinitionGrid
            aria-label="Exact benchmark workspace scope / 精确 Benchmark 工作台范围"
            columns={3}
            compact
            items={view.scope.map((fact) => ({
              ...fact,
              value: <CodeChip>{fact.value}</CodeChip>
            }))}
            surface="raised"
            valueTone="info"
          />

          <section
            aria-labelledby="local-benchmark-workspace-suite-heading"
            className="local-benchmark-workspace__section"
          >
            <SectionHeading
              id="local-benchmark-workspace-suite-heading"
              title="Evaluation suite / 评测套件"
              trailing={<CodeChip>{projection.suite.id}</CodeChip>}
            />
            <p className="local-benchmark-workspace__suite-name">{projection.suite.name}</p>
          </section>

          <section
            aria-labelledby="local-benchmark-workspace-datasets-heading"
            className="local-benchmark-workspace__section"
          >
            <SectionHeading
              id="local-benchmark-workspace-datasets-heading"
              title="Datasets / 数据集"
            />
            <ProjectionTable
              ariaLabel="Benchmark workspace datasets / Benchmark 工作台数据集"
              emptyMessage={projection.datasets.emptyMessage}
              headers={["Dataset ID / 数据集 ID", "Name / 名称", "Cases / 用例数"]}
              rows={projection.datasets.rows}
            />
          </section>

          <section
            aria-labelledby="local-benchmark-workspace-runs-heading"
            className="local-benchmark-workspace__section"
          >
            <SectionHeading
              id="local-benchmark-workspace-runs-heading"
              title="Run provenance / 运行来源"
            />
            <ProjectionTable
              ariaLabel="Benchmark workspace run provenance / Benchmark 工作台运行来源"
              emptyMessage={projection.runs.emptyMessage}
              headers={["Dataset ID / 数据集 ID", "Case ID / 用例 ID", "Metrics / 指标数"]}
              rows={projection.runs.rows}
            />
          </section>

          <section
            aria-labelledby="local-benchmark-workspace-scorecard-heading"
            className="local-benchmark-workspace__section"
          >
            <SectionHeading
              id="local-benchmark-workspace-scorecard-heading"
              title="Server scorecard / 服务端记分卡"
              trailing={<StatusPill tone="info">{projection.scorecard.runCount}</StatusPill>}
            />
            <ProjectionTable
              ariaLabel="Server-owned benchmark scorecard / 服务端 Benchmark 记分卡"
              emptyMessage={projection.scorecard.emptyMessage}
              headers={[
                "Metric / 指标",
                "Threshold direction / 阈值方向",
                "Threshold / 阈值",
                "Observed / 观测值",
                "Coverage / 覆盖",
                "Completeness / 完整性",
                "Outcome / 结果"
              ]}
              rows={projection.scorecard.rows}
            />
          </section>

          <section
            aria-labelledby="local-benchmark-workspace-regression-heading"
            className="local-benchmark-workspace__section local-benchmark-workspace__regression"
          >
            <SectionHeading
              id="local-benchmark-workspace-regression-heading"
              title="Regression status / 回归状态"
              trailing={
                <StatusPill tone={projection.regressionStatus.tone}>
                  {projection.regressionStatus.label}
                </StatusPill>
              }
            />
          </section>

          <section
            aria-labelledby="local-benchmark-workspace-diff-heading"
            className="local-benchmark-workspace__section"
          >
            <SectionHeading
              id="local-benchmark-workspace-diff-heading"
              title="Evaluation diff / 评测差异"
              trailing={
                projection.evaluationDiff
                  ? <StatusPill tone="info">{projection.evaluationDiff.statusChange}</StatusPill>
                  : <StatusPill tone="neutral">Single scope / 单一范围</StatusPill>
              }
            />
            {projection.evaluationDiff ? (
              <>
                <DefinitionGrid
                  aria-label="Evaluation diff cohorts / 评测差异队列"
                  columns={2}
                  compact
                  items={projection.evaluationDiff.scopes.map((fact) => ({
                    ...fact,
                    value: <CodeChip>{fact.value}</CodeChip>
                  }))}
                  surface="raised"
                  valueTone="info"
                />
                <ProjectionTable
                  ariaLabel="Server-owned evaluation metric diff / 服务端评测指标差异"
                  emptyMessage={projection.evaluationDiff.emptyMessage}
                  headers={[
                    "Metric / 指标",
                    "Change / 变更",
                    "Baseline / 基线",
                    "Revised / 修订"
                  ]}
                  rows={projection.evaluationDiff.rows}
                />
              </>
            ) : (
              <p
                aria-live="polite"
                className="local-benchmark-workspace__empty"
                role="status"
              >
                No baseline was requested; the server returned no evaluation diff. / 未请求基线；服务端未返回评测差异。
              </p>
            )}
          </section>
        </div>
      ) : null}
    </section>
  );
}

function SectionHeading({
  id,
  title,
  trailing
}: Readonly<{ id: string; title: string; trailing?: React.ReactNode }>) {
  return (
    <div className="local-benchmark-workspace__section-heading">
      <h4 id={id}>{title}</h4>
      {trailing}
    </div>
  );
}

function ProjectionTable({
  ariaLabel,
  emptyMessage,
  headers,
  rows
}: Readonly<{
  ariaLabel: string;
  emptyMessage: string;
  headers: ReadonlyArray<string>;
  rows: ReadonlyArray<LocalBenchmarkWorkspaceTableRow>;
}>) {
  return rows.length > 0 ? (
    <StackTable
      aria-label={ariaLabel}
      headers={[...headers]}
      rows={rows.map((row) => ({ id: row.id, cells: [...row.cells] }))}
    />
  ) : (
    <p aria-live="polite" className="local-benchmark-workspace__empty" role="status">
      {emptyMessage}
    </p>
  );
}
