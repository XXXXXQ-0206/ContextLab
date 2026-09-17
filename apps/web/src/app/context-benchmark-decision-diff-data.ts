import {
  parseLocalBenchmarkDecisionDiff,
  type LocalApiErrorBody,
  type LocalBenchmarkDecisionDiff,
  type LocalBenchmarkDecisionScope
} from "@contextlab/local-sdk";

export class LocalBenchmarkDecisionDiffProxyError extends Error {
  readonly status: number;
  readonly body: LocalApiErrorBody;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalBenchmarkDecisionDiffProxyError";
    this.status = status;
    this.body = body;
    this.retryAfterMs = retryAfterMs;
  }
}

export async function loadLocalBenchmarkDecisionDiff(
  projectId: string,
  contextId: string,
  baseline: LocalBenchmarkDecisionScope,
  revised: LocalBenchmarkDecisionScope,
  bearerToken: string
): Promise<LocalBenchmarkDecisionDiff> {
  const query = new URLSearchParams({
    baseline_commit_id: baseline.commit_id,
    baseline_decision_id: baseline.decision_id,
    revised_commit_id: revised.commit_id,
    revised_decision_id: revised.decision_id
  });
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/benchmark-decision-diffs?${query.toString()}`,
    {
      headers: {
        accept: "application/json",
        authorization: `Bearer ${bearerToken.trim()}`
      },
      credentials: "omit",
      cache: "no-store"
    }
  );

  if (!response.ok) {
    throw new LocalBenchmarkDecisionDiffProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  return parseLocalBenchmarkDecisionDiff(await response.json());
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;
    if (typeof body.error === "string" && typeof body.message === "string") {
      return { error: body.error, message: body.message };
    }
  } catch {
  }

  return {
    error: "contextlab_benchmark_decision_diff_proxy_error",
    message: `ContextLab benchmark decision diff request failed with status ${response.status}`
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }

  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}
