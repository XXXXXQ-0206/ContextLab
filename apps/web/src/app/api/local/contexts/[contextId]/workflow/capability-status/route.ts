import { proxyLocalWorkflowCapabilityStatus } from "../../../../../../context-lifecycle-proxy";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string }> }
) {
  const { contextId } = await context.params;
  return proxyLocalWorkflowCapabilityStatus(request, contextId);
}
