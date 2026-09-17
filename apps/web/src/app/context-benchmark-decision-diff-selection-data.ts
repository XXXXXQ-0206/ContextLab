import {
  loadLocalBenchmarkDecisionDiscovery,
  type LocalBenchmarkDecisionDiscovery
} from "./context-benchmark-decision-discovery-data";

export type LocalBenchmarkDecisionDiffSelections = Readonly<{
  baseline: LocalBenchmarkDecisionDiscovery;
  revised: LocalBenchmarkDecisionDiscovery;
}>;

export async function loadLocalBenchmarkDecisionDiffSelections(
  projectId: string,
  contextId: string,
  baselineCommitId: string,
  revisedCommitId: string,
  bearerToken: string
): Promise<LocalBenchmarkDecisionDiffSelections> {
  const [baseline, revised] = await Promise.all([
    loadLocalBenchmarkDecisionDiscovery(projectId, contextId, baselineCommitId, bearerToken),
    loadLocalBenchmarkDecisionDiscovery(projectId, contextId, revisedCommitId, bearerToken)
  ]);

  return Object.freeze({ baseline, revised });
}
