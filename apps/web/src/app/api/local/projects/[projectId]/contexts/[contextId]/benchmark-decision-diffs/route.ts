import { proxyLocalBenchmarkDecisionDiff } from "../../../../../../../context-lifecycle-proxy";

export async function GET(
  request: Request,
  context: {
    params: Promise<{ projectId: string; contextId: string }>;
  }
) {
  const { projectId, contextId } = await context.params;
  const searchParams = new URL(request.url).searchParams;
  const baselineCommitId = searchParams.get("baseline_commit_id");
  const baselineDecisionId = searchParams.get("baseline_decision_id");
  const revisedCommitId = searchParams.get("revised_commit_id");
  const revisedDecisionId = searchParams.get("revised_decision_id");

  if (!baselineCommitId || !baselineDecisionId || !revisedCommitId || !revisedDecisionId) {
    return new Response(
      JSON.stringify({
        error: "invalid_benchmark_decision_diff_request",
        message: "All baseline and revised decision scope fields are required"
      }),
      {
        status: 400,
        headers: {
          "cache-control": "private, no-store",
          "content-type": "application/json"
        }
      }
    );
  }

  return proxyLocalBenchmarkDecisionDiff(
    request,
    projectId,
    contextId,
    { commit_id: baselineCommitId, decision_id: baselineDecisionId },
    { commit_id: revisedCommitId, decision_id: revisedDecisionId }
  );
}
