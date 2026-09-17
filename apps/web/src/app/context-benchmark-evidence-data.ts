import {
  parseLocalBenchmarkDecision,
  parseLocalBenchmarkDecisionRunDetails,
  type LocalApiErrorBody,
  type LocalBenchmarkDecision,
  type LocalBenchmarkDecisionRunDetails
} from "@contextlab/local-sdk";

export class LocalBenchmarkEvidenceProxyError extends Error {
  readonly status: number;
  readonly body: LocalApiErrorBody;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    const redactedBody = {
      error: body.error,
      message: redactedBenchmarkEvidenceErrorMessage(status)
    };
    super(redactedBody.message);
    this.name = "LocalBenchmarkEvidenceProxyError";
    this.status = status;
    this.body = redactedBody;
    this.retryAfterMs = retryAfterMs;
  }
}

export async function loadLocalBenchmarkDecision(
  projectId: string,
  contextId: string,
  commitId: string,
  decisionId: string,
  bearerToken: string
): Promise<LocalBenchmarkDecision> {
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-decisions/${encodeURIComponent(decisionId)}`,
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
    throw new LocalBenchmarkEvidenceProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  return parseLocalBenchmarkDecision(await response.json());
}

export async function loadLocalBenchmarkDecisionRunDetails(
  projectId: string,
  contextId: string,
  commitId: string,
  decisionId: string,
  bearerToken: string
): Promise<LocalBenchmarkDecisionRunDetails> {
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-decisions/${encodeURIComponent(decisionId)}/run-details`,
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
    throw new LocalBenchmarkEvidenceProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  return parseLocalBenchmarkDecisionRunDetails(await response.json());
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;
    if (typeof body.error === "string") {
      return {
        error: body.error,
        message: redactedBenchmarkEvidenceErrorMessage(response.status)
      };
    }
  } catch {
  }

  return {
    error: "contextlab_benchmark_evidence_proxy_error",
    message: redactedBenchmarkEvidenceErrorMessage(response.status)
  };
}

function redactedBenchmarkEvidenceErrorMessage(status: number): string {
  return `Benchmark evidence request failed with status ${status} / Benchmark 证据请求失败，状态 ${status}`;
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }

  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}
