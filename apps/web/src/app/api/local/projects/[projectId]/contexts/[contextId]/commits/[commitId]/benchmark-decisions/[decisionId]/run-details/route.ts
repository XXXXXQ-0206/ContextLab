import { proxyLocalBenchmarkDecisionRunDetails } from "../../../../../../../../../../../context-lifecycle-proxy";

export async function GET(
  request: Request,
  context: {
    params: Promise<{
      projectId: string;
      contextId: string;
      commitId: string;
      decisionId: string;
    }>;
  }
) {
  const { projectId, contextId, commitId, decisionId } = await context.params;
  return proxyLocalBenchmarkDecisionRunDetails(request, projectId, contextId, commitId, decisionId);
}
