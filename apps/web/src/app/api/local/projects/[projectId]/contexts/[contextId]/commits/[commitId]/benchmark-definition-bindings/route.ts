import {
  getBenchmarkDefinitionBindings,
  postBenchmarkDefinitionAuthoring,
  type BenchmarkDefinitionAuthoringScope
} from "../../../../../../../contexts/[contextId]/commits/[commitId]/benchmark-definitions/route";

export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ projectId: string; contextId: string; commitId: string }>;
};

export async function POST(request: Request, context: RouteContext): Promise<Response> {
  const params = await context.params;
  const scope: BenchmarkDefinitionAuthoringScope = Object.freeze({
    projectId: params.projectId,
    contextId: params.contextId,
    commitId: params.commitId
  });
  return postBenchmarkDefinitionAuthoring(request, scope);
}

export async function GET(request: Request, context: RouteContext): Promise<Response> {
  const params = await context.params;
  const scope = {
    projectId: params.projectId,
    contextId: params.contextId,
    commitId: params.commitId
  } as const;
  return getBenchmarkDefinitionBindings(request, scope);
}
