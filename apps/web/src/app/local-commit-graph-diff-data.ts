import {
  parseLocalCommitGraphDiffResponseV1,
  type LocalCommitGraphDiffResponseV1
} from "@contextlab/local-sdk";

export type LocalCommitGraphDiffErrorBody = Readonly<{
  error: string;
  message: string;
}>;

const REDACTED_GRAPH_DIFF_ERROR_MESSAGE = "Unable to load local graph review / 无法加载本地图谱审阅。";

export class LocalCommitGraphDiffProxyError extends Error {
  readonly body: LocalCommitGraphDiffErrorBody;
  readonly status: number;

  constructor(status: number, body: LocalCommitGraphDiffErrorBody) {
    super(body.message);
    this.name = "LocalCommitGraphDiffProxyError";
    this.status = status;
    this.body = body;
  }
}

export async function loadLocalCommitGraphDiff(
  contextId: string,
  originalCommitId: string,
  revisedCommitId: string,
  bearerToken: string
): Promise<LocalCommitGraphDiffResponseV1> {
  const response = await fetch(
    `/api/local/contexts/${encodeURIComponent(contextId)}/graph-diff?${new URLSearchParams({
      original_commit_id: originalCommitId,
      revised_commit_id: revisedCommitId
    }).toString()}`,
    {
      headers: {
        accept: "application/json",
        authorization: `Bearer ${bearerToken.trim()}`
      },
      cache: "no-store",
      credentials: "omit"
    }
  );

  if (!response.ok) {
    throw new LocalCommitGraphDiffProxyError(response.status, await parseErrorBody(response));
  }

  const result = parseLocalCommitGraphDiffResponseV1(await response.json());
  if (
    result.context_id !== contextId
    || result.original.commit_id !== originalCommitId
    || result.revised.commit_id !== revisedCommitId
  ) {
    throw new TypeError("local commit graph diff response does not match the requested scope");
  }

  return result;
}

async function parseErrorBody(response: Response): Promise<LocalCommitGraphDiffErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalCommitGraphDiffErrorBody>;
    if (typeof body.error === "string" && typeof body.message === "string") {
      return Object.freeze({ error: body.error, message: REDACTED_GRAPH_DIFF_ERROR_MESSAGE });
    }
  } catch {
    // The proxy deliberately substitutes a stable error for malformed upstream responses.
  }

  return Object.freeze({
    error: "contextlab_local_commit_graph_diff_error",
    message: REDACTED_GRAPH_DIFF_ERROR_MESSAGE
  });
}
