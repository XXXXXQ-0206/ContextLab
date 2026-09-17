import { proxyLocalComponentLifecycleCommit } from "../../../../../context-lifecycle-proxy";

export async function POST(
  request: Request,
  context: { params: Promise<{ contextId: string }> }
) {
  const { contextId } = await context.params;
  return proxyLocalComponentLifecycleCommit(request, contextId);
}
