import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import {
  adaptLocalBranchHeadsV1,
  type FrozenLocalBranchHeadsDto,
  type LocalBranchHeadsResource
} from "./local-branch-heads-data";

export type LocalBranchHeadsRowModel = Readonly<{
  id: string;
  branch: string;
  headCommitId: string;
  revision: number;
}>;

export type LocalBranchHeadsState = "loading" | "error" | "empty" | "ready" | "unavailable";

export type LocalBranchHeadsScreenModel = Readonly<{
  title: string;
  description: string;
  state: LocalBranchHeadsState;
  status: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  rows: ReadonlyArray<LocalBranchHeadsRowModel>;
  selectedBranch: string;
}>;

const stateCopy: Record<LocalBranchHeadsState, Readonly<{
  label: string;
  description: BilingualCapabilityText;
}>> = {
  loading: {
    label: "Loading branch heads / 正在加载分支 head",
    description: bilingual("The protected branch-head read is loading.", "受保护的分支 head 读取正在加载。")
  },
  error: {
    label: "Branch-head read failed / 分支 head 读取失败",
    description: bilingual("The protected branch-head read failed closed.", "受保护的分支 head 读取已失败关闭。")
  },
  empty: {
    label: "No branch heads / 没有分支 head",
    description: bilingual("No durable branches are recorded for this Context.", "此 Context 没有已记录的持久分支。")
  },
  ready: {
    label: "Ready / 就绪",
    description: bilingual("Durable branch heads are available for read-only inspection.", "持久分支 head 可供只读检查。")
  },
  unavailable: {
    label: "Inspection unavailable / 检查不可用",
    description: bilingual("This read requires a request-memory credential and capability.", "此读取需要请求内存凭据和能力。")
  }
};

export function presentLocalBranchHeads(
  resource: LocalBranchHeadsResource,
  selectedBranch = ""
): LocalBranchHeadsScreenModel {
  const dto = adaptLocalBranchHeadsV1(resource);
  const state: LocalBranchHeadsState = dto.state === "available" ? "ready" : dto.state;
  const rows = dto.branches.map(presentBranchHead);
  const nextSelectedBranch = rows.some((row) => row.branch === selectedBranch)
    ? selectedBranch
    : (rows[0]?.branch ?? "");
  const statusState = dto.state === "available" ? "available" : dto.state;
  const description = stateCopy[state].description;
  const status = presentCapabilityState({
    id: dto.id,
    capability: dto.capability,
    state: statusState,
    summary: description,
    detail: resource.kind === "error" || resource.kind === "unavailable"
      ? resource.message
        ? bilingual(resource.message, resource.message)
        : undefined
      : bilingual(`${rows.length} branch head${rows.length === 1 ? "" : "s"}`, `${rows.length} 个分支 head`)
  } satisfies FrozenCapabilityStateDto);

  return Object.freeze({
    title: "Local branch heads / 本地分支 head",
    description: "Read-only durable Context branch inspection. / 只读持久 Context 分支检查。",
    state,
    status: Object.freeze({
      ...status,
      stateLabel: stateCopy[state].label,
      description: `${description.en} / ${description.zh}`
    }),
    scope: Object.freeze([
      Object.freeze({ id: "context-id", label: "Context / 上下文", value: dto.contextId })
    ]),
    rows: Object.freeze(rows),
    selectedBranch: nextSelectedBranch
  });
}

function presentBranchHead(
  head: FrozenLocalBranchHeadsDto["branches"][number]
): LocalBranchHeadsRowModel {
  return Object.freeze({
    id: head.branch_name,
    branch: head.branch_name,
    headCommitId: head.head_commit_id ?? "Unborn / 未初始化",
    revision: head.revision
  });
}

function bilingual(en: string, zh: string): BilingualCapabilityText {
  return Object.freeze({ en, zh });
}
