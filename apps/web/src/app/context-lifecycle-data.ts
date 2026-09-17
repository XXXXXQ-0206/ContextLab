import type {
  LocalApiErrorBody,
  LocalComponentLifecycleCommitRequest,
  LocalComponentLifecycleCommitResponse,
  LocalContextMetadata,
  LocalJsonValue,
  LocalContextLifecycleState
} from "@contextlab/local-sdk";
import {
  parseLocalComponentLifecycleCommitResponse,
  parseLocalContextLifecycleState
} from "@contextlab/local-sdk";

export class LocalLifecycleProxyError extends Error {
  readonly status: number;
  readonly body: LocalApiErrorBody;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    const redactedBody = {
      error: body.error,
      message: redactedProxyErrorMessage(status)
    };
    super(redactedBody.message);
    this.name = "LocalLifecycleProxyError";
    this.status = status;
    this.body = redactedBody;
    this.retryAfterMs = retryAfterMs;
  }
}

export type LocalLifecycleComponentMetadata = Readonly<{
  component_id: string;
  name: string;
  metadata: LocalJsonValue;
}>;

export function readLocalLifecycleContextMetadata(
  state: Pick<LocalContextLifecycleState, "metadata"> | null
): LocalContextMetadata | null {
  return state?.metadata ?? null;
}

export function readLocalLifecycleComponentMetadata(
  state: Pick<LocalContextLifecycleState, "components"> | null,
  componentId: string
): LocalLifecycleComponentMetadata | null {
  const normalizedComponentId = componentId.trim();
  if (!state || !normalizedComponentId) {
    return null;
  }

  const component = state.components.find((candidate) => candidate.component_id === normalizedComponentId);
  return component
    ? {
      component_id: component.component_id,
      name: component.name,
      metadata: component.metadata
    }
    : null;
}

export function loadLocalContextLifecycleState(
  contextId: string,
  commitId: string,
  bearerToken: string
): Promise<LocalContextLifecycleState> {
  return requestLocalLifecycle(
    `/api/local/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/lifecycle-state`,
    bearerToken,
    parseLocalContextLifecycleState
  ).then((state) => {
    if (state.context_id !== contextId || state.commit_id !== commitId) {
      throw new TypeError("local Context lifecycle response does not match the requested scope");
    }
    return state;
  });
}

export function submitLocalComponentLifecycleCommit(
  contextId: string,
  bearerToken: string,
  idempotencyKey: string,
  command: LocalComponentLifecycleCommitRequest
): Promise<LocalComponentLifecycleCommitResponse> {
  return requestLocalLifecycle(
    `/api/local/contexts/${encodeURIComponent(contextId)}/component-lifecycle-commits`,
    bearerToken,
    parseLocalComponentLifecycleCommitResponse,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "idempotency-key": idempotencyKey
      },
      body: serializeLifecycleCommand(command)
    }
  ).then((result) => {
    if (result.snapshot.context_id !== contextId || result.snapshot.commit_id !== result.commit_id) {
      throw new TypeError("local Context lifecycle commit response does not match the requested scope");
    }
    return result;
  });
}

function serializeLifecycleCommand(command: LocalComponentLifecycleCommitRequest): string {
  if (command.operation.kind !== "update_descriptor") {
    return JSON.stringify(command);
  }

  return JSON.stringify({
    ...command,
    operation: {
      kind: command.operation.kind,
      component_id: command.operation.component_id,
      name: command.operation.name,
      metadata: command.operation.metadata
    }
  });
}

async function requestLocalLifecycle<TResponse>(
  path: string,
  bearerToken: string,
  parse: (value: unknown) => TResponse,
  init: RequestInit = {}
): Promise<TResponse> {
  const headers = new Headers(init.headers);
  headers.set("accept", "application/json");
  headers.set("authorization", `Bearer ${bearerToken.trim()}`);
  const response = await fetch(path, { ...init, credentials: "omit", cache: "no-store", headers });

  if (!response.ok) {
    throw new LocalLifecycleProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  return parse(await response.json());
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;
    if (typeof body.error === "string") {
      return { error: body.error, message: redactedProxyErrorMessage(response.status) };
    }
  } catch {
  }

  return redactedProxyErrorBody(response.status);
}

function redactedProxyErrorBody(status: number): LocalApiErrorBody {
  return {
    error: "contextlab_local_proxy_error",
    message: redactedProxyErrorMessage(status)
  };
}

function redactedProxyErrorMessage(status: number): string {
  return `Context lifecycle request failed with status ${status}`;
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }

  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}
