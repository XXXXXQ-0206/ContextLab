import {
  parsePersistedContextDiffReviewV1,
  type LocalApiErrorBody,
  type LocalPersistedContextDiffReviewV1
} from "@contextlab/local-sdk";

export type LocalPersistedContextDiffReviewTarget = Readonly<{
  project_id: string;
  context_id: string;
  source_commit_id: string;
  target_commit_id: string;
}>;

export type LocalPersistedContextDiffReviewResource =
  | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalPersistedContextDiffReviewTarget; message?: string }>
  | Readonly<{ kind: "ready"; target: LocalPersistedContextDiffReviewTarget; review: LocalPersistedContextDiffReviewV1 }>;

const REDACTED_PERSISTED_CONTEXT_DIFF_ERROR_MESSAGE =
  "The local persisted Context diff review is unavailable. / 本地持久化 Context diff review 不可用。";

export class LocalPersistedContextDiffReviewProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalPersistedContextDiffReviewProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export function createLocalPersistedContextDiffReviewResource(
  input: LocalPersistedContextDiffReviewResource
): LocalPersistedContextDiffReviewResource {
  const target = freezeTarget(input.target);
  if (input.kind !== "ready") return Object.freeze({ ...input, target });
  const review = parsePersistedContextDiffReviewV1(input.review);
  assertScope(review, target);
  return Object.freeze({ kind: "ready", target, review });
}

export async function loadLocalPersistedContextDiffReview(
  target: LocalPersistedContextDiffReviewTarget,
  bearerToken: string
): Promise<LocalPersistedContextDiffReviewV1> {
  const exactTarget = freezeTarget(target);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(exactTarget.project_id)}/contexts/${encodeURIComponent(exactTarget.context_id)}/diff-review?${new URLSearchParams({
      source_commit_id: exactTarget.source_commit_id,
      target_commit_id: exactTarget.target_commit_id
    }).toString()}`,
    {
      credentials: "omit",
      cache: "no-store",
      headers: { accept: "application/json", authorization: `Bearer ${token}` }
    }
  );
  if (!response.ok) {
    throw new LocalPersistedContextDiffReviewProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }
  const review = parsePersistedContextDiffReviewV1(await response.json());
  assertScope(review, exactTarget);
  return review;
}

function assertScope(review: LocalPersistedContextDiffReviewV1, target: LocalPersistedContextDiffReviewTarget): void {
  if (
    review.source_scope.project_id !== target.project_id
    || review.source_scope.context_id !== target.context_id
    || review.source_scope.commit_id !== target.source_commit_id
    || review.target_scope.project_id !== target.project_id
    || review.target_scope.context_id !== target.context_id
    || review.target_scope.commit_id !== target.target_commit_id
  ) {
    throw new TypeError("persisted Context diff review response is outside the requested scope");
  }
}

function freezeTarget(target: LocalPersistedContextDiffReviewTarget): LocalPersistedContextDiffReviewTarget {
  const exact = {
    project_id: requireNonBlank(target.project_id, "target.project_id"),
    context_id: requireNonBlank(target.context_id, "target.context_id"),
    source_commit_id: requireNonBlank(target.source_commit_id, "target.source_commit_id"),
    target_commit_id: requireNonBlank(target.target_commit_id, "target.target_commit_id")
  };
  if (exact.source_commit_id === exact.target_commit_id) throw new RangeError("source and target commits must differ");
  return Object.freeze(exact);
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const value = await response.json();
    if (typeof value === "object" && value !== null && !Array.isArray(value)
      && typeof value.error === "string" && typeof value.message === "string") {
      return { error: value.error, message: REDACTED_PERSISTED_CONTEXT_DIFF_ERROR_MESSAGE };
    }
  } catch {
  }
  return {
    error: "contextlab_local_persisted_context_diff_review_error",
    message: REDACTED_PERSISTED_CONTEXT_DIFF_ERROR_MESSAGE
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}

function requireNonBlank(value: string, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`);
  return value.trim();
}
