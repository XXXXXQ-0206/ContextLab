import { proxyLocalContextLifecycleState } from "../../../../../../../context-lifecycle-proxy";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string; commitId: string }> }
) {
  const { contextId, commitId } = await context.params;
  return proxyLocalContextLifecycleState(request, contextId, commitId);
}
