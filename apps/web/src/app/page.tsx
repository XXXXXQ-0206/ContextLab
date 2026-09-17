import { ContextWorkspaceScreen } from "./context-workspace-screen";
import { loadContextWorkspace } from "./context-workspace-data";
import {
  isLocalContextMergeReviewDevelopmentEnabled,
  isLocalLifecycleDevelopmentEnabled
} from "./context-lifecycle-proxy";
import { presentContextWorkspaceScreen } from "./context-workspace-presenter";

export const dynamic = "force-dynamic";

export default async function Home() {
  const workspaceData = await loadContextWorkspace();

  return (
    <ContextWorkspaceScreen
      {...presentContextWorkspaceScreen(workspaceData)}
      localLifecycleEnabled={isLocalLifecycleDevelopmentEnabled()}
      mergeReviewEnabled={isLocalContextMergeReviewDevelopmentEnabled()}
    />
  );
}
