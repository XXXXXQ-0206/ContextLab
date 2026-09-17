import type {
  LocalBenchmarkDecisionList,
  LocalBenchmarkDecisionListItem,
  LocalLifecycleReadCredentials
} from "./types";
import { parseLocalBenchmarkDecisionList } from "./types";

export const LOCAL_BENCHMARK_WORKSPACE_SCHEMA_V1 = "contextlab.local-benchmark-workspace.v1";

export type LocalBenchmarkWorkspaceDecisionSelectionScope = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
}>;

export type LocalBenchmarkWorkspaceDecisionSelection = Readonly<
  LocalBenchmarkWorkspaceDecisionSelectionScope & {
    decisionId: string;
    decision: LocalBenchmarkDecisionListItem;
  }
>;

export type LocalBenchmarkWorkspaceDecisionSelectionResource =
  | Readonly<{
      state: "loading" | "error" | "unavailable";
      scope: LocalBenchmarkWorkspaceDecisionSelectionScope;
      message?: string;
    }>
  | Readonly<{
      state: "empty";
      scope: LocalBenchmarkWorkspaceDecisionSelectionScope;
      decisions: readonly [];
      selection: null;
    }>
  | Readonly<{
      state: "available";
      scope: LocalBenchmarkWorkspaceDecisionSelectionScope;
      decisions: readonly LocalBenchmarkDecisionListItem[];
      selection: LocalBenchmarkWorkspaceDecisionSelection | null;
    }>;

export type LocalBenchmarkWorkspaceDecisionSelectionReader = {
  getBenchmarkWorkspaceForDecision(
    projectId: string,
    contextId: string,
    commitId: string,
    decisionId: string,
    credentials: LocalLifecycleReadCredentials,
    baseline?: LocalBenchmarkWorkspaceDecisionBaselineRequest
  ): Promise<LocalBenchmarkWorkspace>;
};

export type LocalBenchmarkWorkspaceDecisionSelectionResourceInput =
  | Readonly<{
      state: "loading" | "error" | "unavailable";
      scope: LocalBenchmarkWorkspaceDecisionSelectionScope;
      message?: string;
    }>
  | Readonly<{
      state: "available";
      scope: LocalBenchmarkWorkspaceDecisionSelectionScope;
      list: LocalBenchmarkDecisionList;
      selectedDecisionId: string | null;
    }>;

export function createLocalBenchmarkWorkspaceDecisionSelectionResource(
  input: LocalBenchmarkWorkspaceDecisionSelectionResourceInput
): LocalBenchmarkWorkspaceDecisionSelectionResource {
  const scope = freezeSelectionScope(input.scope);
  if (input.state !== "available") {
    return Object.freeze({
      state: input.state,
      scope,
      ...(input.message === undefined ? {} : { message: input.message })
    });
  }

  const list = parseLocalBenchmarkDecisionList(input.list);
  assertDecisionListScope(list, scope);
  if (list.decisions.length === 0) {
    return Object.freeze({
      state: "empty" as const,
      scope,
      decisions: Object.freeze([]) as readonly [],
      selection: null
    });
  }

  const selection = selectLocalBenchmarkWorkspaceDecision(
    list,
    scope,
    input.selectedDecisionId
  );
  return Object.freeze({
    state: "available" as const,
    scope,
    decisions: Object.freeze([...list.decisions]),
    selection
  });
}

export function selectLocalBenchmarkWorkspaceDecision(
  list: LocalBenchmarkDecisionList,
  scope: LocalBenchmarkWorkspaceDecisionSelectionScope,
  decisionId: string | null
): LocalBenchmarkWorkspaceDecisionSelection | null {
  const parsedList = parseLocalBenchmarkDecisionList(list);
  const frozenScope = freezeSelectionScope(scope);
  assertDecisionListScope(parsedList, frozenScope);
  if (decisionId === null) {
    return null;
  }

  const requestedDecisionId = asNonBlankString(decisionId, "decisionId");
  const decision = parsedList.decisions.find(
    (candidate) => candidate.decision_id === requestedDecisionId
  );
  if (decision === undefined) {
    throw new TypeError("selected benchmark decision is not present in the exact decision list");
  }

  return Object.freeze({
    ...frozenScope,
    decisionId: decision.decision_id,
    decision
  });
}

export async function loadLocalBenchmarkWorkspaceForDecisionSelection(
  reader: LocalBenchmarkWorkspaceDecisionSelectionReader,
  selection: LocalBenchmarkWorkspaceDecisionSelection,
  credentials: LocalLifecycleReadCredentials,
  baseline?: LocalBenchmarkWorkspaceDecisionBaselineRequest
): Promise<LocalBenchmarkWorkspace> {
  const scope = freezeSelectionScope(selection);
  if (selection.decisionId !== selection.decision.decision_id) {
    throw new TypeError("selected benchmark decision identity is inconsistent");
  }

  return reader.getBenchmarkWorkspaceForDecision(
    scope.projectId,
    scope.contextId,
    scope.commitId,
    selection.decisionId,
    credentials,
    baseline
  );
}

export type LocalBenchmarkWorkspaceScope = {
  commit_id: string;
  cohort_id: string;
};

export type LocalBenchmarkWorkspaceBaselineRequest = {
  commitId: string;
  cohortId: string;
};

export type LocalBenchmarkWorkspaceDecisionBaselineRequest = {
  commitId: string;
  decisionId: string;
};

export type LocalBenchmarkWorkspaceDecisionScope = {
  commit_id: string;
  decision_id: string;
};

export type LocalBenchmarkWorkspaceDecisionPairWitness = {
  schema_version: 1;
  project_id: string;
  context_id: string;
  baseline: LocalBenchmarkWorkspaceDecisionScope;
  revised: LocalBenchmarkWorkspaceDecisionScope;
};

export type LocalBenchmarkWorkspaceMetricKind =
  | "latency_ms"
  | "cost_usd"
  | "accuracy"
  | "hallucination_rate"
  | "tool_usage_count"
  | "token_count"
  | "execution_time_ms"
  | "output_quality"
  | "success_rate";

export type LocalBenchmarkWorkspaceDecisionStatus =
  | "passed"
  | "regressed"
  | "insufficient_data";

export type LocalBenchmarkWorkspaceMetric = {
  metric: LocalBenchmarkWorkspaceMetricKind;
  threshold_direction: "minimum" | "maximum";
  threshold_value: number;
  observed: number | null;
  sample_count: number;
  required_sample_count: number;
  has_complete_coverage: boolean;
  outcome: LocalBenchmarkWorkspaceDecisionStatus;
};

export type LocalBenchmarkWorkspaceProjectionV1 = {
  schema_version: 1;
  receipt: { cohort_id: string };
  suite: { id: string; name: string };
  datasets: Array<{ id: string; name: string; case_count: number }>;
  runs: Array<{ dataset_id: string; case_id: string; metric_count: number }>;
  scorecard: {
    run_count: number;
    metrics: LocalBenchmarkWorkspaceMetric[];
  };
  regression_status: LocalBenchmarkWorkspaceDecisionStatus;
  evaluation_diff: null | {
    baseline_cohort_id: string;
    revised_cohort_id: string;
    status_change: null | [
      LocalBenchmarkWorkspaceDecisionStatus,
      LocalBenchmarkWorkspaceDecisionStatus
    ];
    metric_changes: Array<{
      metric: LocalBenchmarkWorkspaceMetricKind;
      change_kind: "added" | "removed" | "modified";
      baseline: LocalBenchmarkWorkspaceMetric | null;
      revised: LocalBenchmarkWorkspaceMetric | null;
    }>;
  };
};

export type LocalBenchmarkWorkspace = {
  schema_version: typeof LOCAL_BENCHMARK_WORKSPACE_SCHEMA_V1;
  project_id: string;
  context_id: string;
  revised: LocalBenchmarkWorkspaceScope;
  baseline: LocalBenchmarkWorkspaceScope | null;
  decision_pair_witness?: LocalBenchmarkWorkspaceDecisionPairWitness;
  projection: LocalBenchmarkWorkspaceProjectionV1;
};

const metricOrder: readonly LocalBenchmarkWorkspaceMetricKind[] = [
  "latency_ms",
  "cost_usd",
  "accuracy",
  "hallucination_rate",
  "tool_usage_count",
  "token_count",
  "execution_time_ms",
  "output_quality",
  "success_rate"
];

const metricOrderIndex = new Map(metricOrder.map((metric, index) => [metric, index]));

export function parseLocalBenchmarkWorkspace(value: unknown): LocalBenchmarkWorkspace {
  const record = asRecord(value, "local benchmark workspace");
  const hasDecisionPairWitness = Object.prototype.hasOwnProperty.call(
    record,
    "decision_pair_witness"
  );
  assertExactKeys(record, [
    "schema_version",
    "project_id",
    "context_id",
    "revised",
    "baseline",
    ...(hasDecisionPairWitness ? ["decision_pair_witness"] : []),
    "projection"
  ]);
  if (record.schema_version !== LOCAL_BENCHMARK_WORKSPACE_SCHEMA_V1) {
    throw new TypeError(`schema_version must be ${LOCAL_BENCHMARK_WORKSPACE_SCHEMA_V1}`);
  }

  const projectId = asNonBlankString(record.project_id, "project_id");
  const contextId = asNonBlankString(record.context_id, "context_id");
  const revised = parseScope(record.revised, "revised");
  const baseline = record.baseline === null ? null : parseScope(record.baseline, "baseline");
  const decisionPairWitness = hasDecisionPairWitness
    ? parseDecisionPairWitness(record.decision_pair_witness)
    : undefined;
  if (decisionPairWitness !== undefined) {
    if (baseline === null) {
      throw new TypeError("decision_pair_witness requires a benchmark workspace comparison");
    }
    if (
      decisionPairWitness.project_id !== projectId
      || decisionPairWitness.context_id !== contextId
      || decisionPairWitness.baseline.commit_id !== baseline.commit_id
      || decisionPairWitness.revised.commit_id !== revised.commit_id
      || decisionPairWitness.baseline.commit_id === decisionPairWitness.revised.commit_id
      || decisionPairWitness.baseline.decision_id === decisionPairWitness.revised.decision_id
    ) {
      throw new TypeError("decision_pair_witness does not match a valid benchmark workspace pair");
    }
  }
  const projection = parseProjection(record.projection);
  if (projection.receipt.cohort_id !== revised.cohort_id) {
    throw new TypeError("benchmark workspace projection does not match the revised cohort");
  }
  if (baseline === null) {
    if (projection.evaluation_diff !== null) {
      throw new TypeError("single benchmark workspace projection cannot include an evaluation diff");
    }
  } else if (
    projection.evaluation_diff === null
    || projection.evaluation_diff.baseline_cohort_id !== baseline.cohort_id
    || projection.evaluation_diff.revised_cohort_id !== revised.cohort_id
  ) {
    throw new TypeError("benchmark workspace evaluation diff does not match the requested scopes");
  }

  return {
    schema_version: LOCAL_BENCHMARK_WORKSPACE_SCHEMA_V1,
    project_id: projectId,
    context_id: contextId,
    revised,
    baseline,
    ...(decisionPairWitness === undefined
      ? {}
      : { decision_pair_witness: decisionPairWitness }),
    projection
  };
}

function parseDecisionPairWitness(
  value: unknown
): LocalBenchmarkWorkspaceDecisionPairWitness {
  const record = asRecord(value, "decision_pair_witness");
  assertExactKeys(record, [
    "schema_version",
    "project_id",
    "context_id",
    "baseline",
    "revised"
  ]);
  if (record.schema_version !== 1) {
    throw new TypeError("decision_pair_witness.schema_version must be 1");
  }

  return {
    schema_version: 1,
    project_id: asNonBlankString(record.project_id, "decision_pair_witness.project_id"),
    context_id: asNonBlankString(record.context_id, "decision_pair_witness.context_id"),
    baseline: parseDecisionScope(record.baseline, "decision_pair_witness.baseline"),
    revised: parseDecisionScope(record.revised, "decision_pair_witness.revised")
  };
}

function parseDecisionScope(
  value: unknown,
  field: string
): LocalBenchmarkWorkspaceDecisionScope {
  const record = asRecord(value, field);
  assertExactKeys(record, ["commit_id", "decision_id"]);
  return {
    commit_id: asNonBlankString(record.commit_id, `${field}.commit_id`),
    decision_id: asNonBlankString(record.decision_id, `${field}.decision_id`)
  };
}

function parseProjection(value: unknown): LocalBenchmarkWorkspaceProjectionV1 {
  const record = asRecord(value, "projection");
  assertExactKeys(record, [
    "schema_version",
    "receipt",
    "suite",
    "datasets",
    "runs",
    "scorecard",
    "regression_status",
    "evaluation_diff"
  ]);
  if (record.schema_version !== 1) {
    throw new TypeError("projection.schema_version must be 1");
  }

  const receiptRecord = asRecord(record.receipt, "projection.receipt");
  assertExactKeys(receiptRecord, ["cohort_id"]);
  const suiteRecord = asRecord(record.suite, "projection.suite");
  assertExactKeys(suiteRecord, ["id", "name"]);
  const datasets = asArray(record.datasets, "projection.datasets").map(parseDataset);
  assertStableOrder(datasets, (dataset) => dataset.id, "projection.datasets");
  const runs = asArray(record.runs, "projection.runs").map(parseRun);
  assertStableOrder(
    runs,
    (run) => `${run.dataset_id}\0${run.case_id}`,
    "projection.runs"
  );
  const knownDatasets = new Map(datasets.map((dataset) => [dataset.id, dataset.case_count]));
  const observedDatasetCases = new Map<string, number>();
  for (const run of runs) {
    if (!knownDatasets.has(run.dataset_id)) {
      throw new TypeError("projection.runs references an unknown dataset");
    }
    observedDatasetCases.set(
      run.dataset_id,
      (observedDatasetCases.get(run.dataset_id) ?? 0) + 1
    );
  }
  for (const dataset of datasets) {
    if ((observedDatasetCases.get(dataset.id) ?? 0) !== dataset.case_count) {
      throw new TypeError("projection dataset case counts do not match run provenance");
    }
  }

  const scorecardRecord = asRecord(record.scorecard, "projection.scorecard");
  assertExactKeys(scorecardRecord, ["run_count", "metrics"]);
  const runCount = asNonNegativeInteger(scorecardRecord.run_count, "scorecard.run_count");
  if (runCount !== runs.length) {
    throw new TypeError("scorecard.run_count must match projection.runs");
  }
  const metrics = asArray(scorecardRecord.metrics, "scorecard.metrics").map((metric) =>
    parseMetric(metric, runCount)
  );
  assertMetricOrder(metrics, "scorecard.metrics");
  const evaluationDiff =
    record.evaluation_diff === null
      ? null
      : parseEvaluationDiff(record.evaluation_diff, runCount);

  return {
    schema_version: 1,
    receipt: {
      cohort_id: asNonBlankString(receiptRecord.cohort_id, "projection.receipt.cohort_id")
    },
    suite: {
      id: asNonBlankString(suiteRecord.id, "projection.suite.id"),
      name: asNonBlankString(suiteRecord.name, "projection.suite.name")
    },
    datasets,
    runs,
    scorecard: { run_count: runCount, metrics },
    regression_status: asDecisionStatus(record.regression_status, "regression_status"),
    evaluation_diff: evaluationDiff
  };
}

function parseScope(value: unknown, field: string): LocalBenchmarkWorkspaceScope {
  const record = asRecord(value, field);
  assertExactKeys(record, ["commit_id", "cohort_id"]);
  return {
    commit_id: asNonBlankString(record.commit_id, `${field}.commit_id`),
    cohort_id: asNonBlankString(record.cohort_id, `${field}.cohort_id`)
  };
}

function parseDataset(value: unknown) {
  const record = asRecord(value, "projection.datasets item");
  assertExactKeys(record, ["id", "name", "case_count"]);
  return {
    id: asNonBlankString(record.id, "datasets.id"),
    name: asNonBlankString(record.name, "datasets.name"),
    case_count: asPositiveInteger(record.case_count, "datasets.case_count")
  };
}

function parseRun(value: unknown) {
  const record = asRecord(value, "projection.runs item");
  assertExactKeys(record, ["dataset_id", "case_id", "metric_count"]);
  return {
    dataset_id: asNonBlankString(record.dataset_id, "runs.dataset_id"),
    case_id: asNonBlankString(record.case_id, "runs.case_id"),
    metric_count: asNonNegativeInteger(record.metric_count, "runs.metric_count")
  };
}

function parseMetric(value: unknown, runCount: number): LocalBenchmarkWorkspaceMetric {
  const record = asRecord(value, "benchmark workspace metric");
  assertExactKeys(record, [
    "metric",
    "threshold_direction",
    "threshold_value",
    "observed",
    "sample_count",
    "required_sample_count",
    "has_complete_coverage",
    "outcome"
  ]);
  const sampleCount = asNonNegativeInteger(record.sample_count, "metric.sample_count");
  const requiredSampleCount = asNonNegativeInteger(
    record.required_sample_count,
    "metric.required_sample_count"
  );
  if (sampleCount > requiredSampleCount || requiredSampleCount !== runCount) {
    throw new TypeError("benchmark workspace metric coverage counts are inconsistent");
  }
  if (typeof record.has_complete_coverage !== "boolean") {
    throw new TypeError("metric.has_complete_coverage must be a boolean");
  }

  return {
    metric: asMetricKind(record.metric, "metric.metric"),
    threshold_direction: asThresholdDirection(record.threshold_direction),
    threshold_value: asFiniteNumber(record.threshold_value, "metric.threshold_value"),
    observed:
      record.observed === null
        ? null
        : asFiniteNumber(record.observed, "metric.observed"),
    sample_count: sampleCount,
    required_sample_count: requiredSampleCount,
    has_complete_coverage: record.has_complete_coverage,
    outcome: asDecisionStatus(record.outcome, "metric.outcome")
  };
}

function parseEvaluationDiff(value: unknown, runCount: number) {
  const record = asRecord(value, "projection.evaluation_diff");
  assertExactKeys(record, [
    "baseline_cohort_id",
    "revised_cohort_id",
    "status_change",
    "metric_changes"
  ]);
  const statusChange = parseStatusChange(record.status_change);
  const metricChanges = asArray(record.metric_changes, "evaluation_diff.metric_changes").map(
    (change) => parseMetricChange(change, runCount)
  );
  assertMetricOrder(metricChanges, "evaluation_diff.metric_changes");
  return {
    baseline_cohort_id: asNonBlankString(
      record.baseline_cohort_id,
      "evaluation_diff.baseline_cohort_id"
    ),
    revised_cohort_id: asNonBlankString(
      record.revised_cohort_id,
      "evaluation_diff.revised_cohort_id"
    ),
    status_change: statusChange,
    metric_changes: metricChanges
  };
}

function parseStatusChange(value: unknown) {
  if (value === null) {
    return null;
  }
  const statuses = asArray(value, "evaluation_diff.status_change");
  if (statuses.length !== 2) {
    throw new TypeError("evaluation_diff.status_change must contain two statuses");
  }
  return [
    asDecisionStatus(statuses[0], "status_change baseline"),
    asDecisionStatus(statuses[1], "status_change revised")
  ] as [LocalBenchmarkWorkspaceDecisionStatus, LocalBenchmarkWorkspaceDecisionStatus];
}

function parseMetricChange(value: unknown, runCount: number) {
  const record = asRecord(value, "evaluation_diff metric change");
  assertExactKeys(record, ["metric", "change_kind", "baseline", "revised"]);
  const metric = asMetricKind(record.metric, "metric_change.metric");
  if (
    record.change_kind !== "added"
    && record.change_kind !== "removed"
    && record.change_kind !== "modified"
  ) {
    throw new TypeError("metric_change.change_kind is invalid");
  }
  const changeKind: "added" | "removed" | "modified" = record.change_kind;
  const baseline = record.baseline === null ? null : parseMetric(record.baseline, runCount);
  const revised = record.revised === null ? null : parseMetric(record.revised, runCount);
  if (
    (changeKind === "added" && (baseline !== null || revised === null))
    || (changeKind === "removed" && (baseline === null || revised !== null))
    || (changeKind === "modified" && (baseline === null || revised === null))
    || (baseline !== null && baseline.metric !== metric)
    || (revised !== null && revised.metric !== metric)
  ) {
    throw new TypeError("evaluation diff metric change evidence is inconsistent");
  }
  return {
    metric,
    change_kind: changeKind,
    baseline,
    revised
  };
}

function freezeSelectionScope(
  scope: LocalBenchmarkWorkspaceDecisionSelectionScope
): LocalBenchmarkWorkspaceDecisionSelectionScope {
  return Object.freeze({
    projectId: asNonBlankString(scope.projectId, "projectId"),
    contextId: asNonBlankString(scope.contextId, "contextId"),
    commitId: asNonBlankString(scope.commitId, "commitId")
  });
}

function assertDecisionListScope(
  list: LocalBenchmarkDecisionList,
  scope: LocalBenchmarkWorkspaceDecisionSelectionScope
): void {
  if (
    list.project_id !== scope.projectId
    || list.context_id !== scope.contextId
    || list.commit_id !== scope.commitId
  ) {
    throw new TypeError("benchmark decision list does not match the exact workspace scope");
  }
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as Record<string, unknown>;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new TypeError(`${field} must be an array`);
  }
  return value;
}

function asNonBlankString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be a non-blank string`);
  }
  return value;
}

function asFiniteNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`${field} must be finite`);
  }
  return value;
}

function asNonNegativeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${field} must be a non-negative integer`);
  }
  return value;
}

function asPositiveInteger(value: unknown, field: string): number {
  const parsed = asNonNegativeInteger(value, field);
  if (parsed === 0) {
    throw new TypeError(`${field} must be positive`);
  }
  return parsed;
}

function asMetricKind(value: unknown, field: string): LocalBenchmarkWorkspaceMetricKind {
  if (typeof value !== "string" || !metricOrderIndex.has(value as LocalBenchmarkWorkspaceMetricKind)) {
    throw new TypeError(`${field} is invalid`);
  }
  return value as LocalBenchmarkWorkspaceMetricKind;
}

function asThresholdDirection(value: unknown): "minimum" | "maximum" {
  if (value !== "minimum" && value !== "maximum") {
    throw new TypeError("metric.threshold_direction is invalid");
  }
  return value;
}

function asDecisionStatus(
  value: unknown,
  field: string
): LocalBenchmarkWorkspaceDecisionStatus {
  if (value !== "passed" && value !== "regressed" && value !== "insufficient_data") {
    throw new TypeError(`${field} is invalid`);
  }
  return value;
}

function assertExactKeys(record: Record<string, unknown>, expected: readonly string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const sortedExpected = [...expected].sort(compareStrings);
  if (
    actual.length !== sortedExpected.length
    || actual.some((key, index) => key !== sortedExpected[index])
  ) {
    throw new TypeError("benchmark workspace value contains an unexpected shape");
  }
}

function assertStableOrder<T>(
  values: readonly T[],
  identity: (value: T) => string,
  field: string
): void {
  for (let index = 1; index < values.length; index += 1) {
    if (compareStrings(identity(values[index - 1]!), identity(values[index]!)) >= 0) {
      throw new TypeError(`${field} must be unique and deterministically ordered`);
    }
  }
}

function assertMetricOrder(
  values: readonly { metric: LocalBenchmarkWorkspaceMetricKind }[],
  field: string
): void {
  for (let index = 1; index < values.length; index += 1) {
    const previous = metricOrderIndex.get(values[index - 1]!.metric)!;
    const current = metricOrderIndex.get(values[index]!.metric)!;
    if (previous >= current) {
      throw new TypeError(`${field} must be unique and deterministically ordered`);
    }
  }
}

function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
