import {
  parseLocalWorkflowCapabilityStatus,
  type LocalApiErrorBody,
  type LocalWorkflowCapabilityStatus
} from "@contextlab/local-sdk";

export class LocalWorkflowCapabilityProxyError extends Error {
  readonly status: number;
  readonly body: LocalApiErrorBody;

  constructor(status: number, body: LocalApiErrorBody) {
    super(body.message);
    this.name = "LocalWorkflowCapabilityProxyError";
    this.status = status;
    this.body = body;
  }
}

export async function loadLocalWorkflowCapabilityStatus(
  contextId: string,
  bearerToken: string
): Promise<LocalWorkflowCapabilityStatus> {
  const response = await fetch(
    `/api/local/contexts/${encodeURIComponent(contextId)}/workflow/capability-status`,
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
    throw new LocalWorkflowCapabilityProxyError(response.status, await parseProxyErrorBody(response));
  }

  return parseLocalWorkflowCapabilityStatus(await response.json());
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
    error: "contextlab_local_workflow_capability_error",
    message: `ContextLab local workflow capability request failed with status ${response.status}`
  };
}
