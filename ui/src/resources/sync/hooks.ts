import { atomWithStorage } from "@/lib/hooks";
import { resourceSyncNoChanges } from "@/lib/utils";
import { useAtom } from "jotai";
import { Types } from "komodo_client";

type ResourceSyncTabsView = "Config" | "Info" | "Execute" | "Commit";
const syncTabsViewAtom = atomWithStorage<ResourceSyncTabsView>(
  "sync-tabs-v5",
  "Config",
);

export function useResourceSyncTabsView(sync: Types.ResourceSync | undefined) {
  const [_view, setView] = useAtom<ResourceSyncTabsView>(syncTabsViewAtom);

  const hideInfo = sync?.config?.files_on_host
    ? false
    : sync?.config?.file_contents
      ? true
      : false;

  const showCommit = sync && !resourceSyncNoChanges(sync);

  const showExecute =
    showCommit ||
    sync?.info?.pending_deploys?.length ||
    sync?.info?.pending_error ||
    sync?.info?.pending_deploy_error;

  const view =
    _view === "Info" && hideInfo
      ? "Config"
      : (_view === "Execute" && !showExecute) ||
          (_view === "Commit" && !showCommit)
        ? sync?.config?.files_on_host ||
          sync?.config?.repo ||
          sync?.config?.linked_repo
          ? "Info"
          : "Config"
        : _view === "Commit" && !sync?.config?.managed
          ? "Execute"
          : _view;

  return {
    view,
    setView,
    hideInfo,
    showExecute,
    showCommit,
  };
}
