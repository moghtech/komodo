import { resourceSyncNoChanges } from "@/lib/utils";
import { useLocalStorage } from "@mantine/hooks";
import { Types } from "komodo_client";

type ResourceSyncTabsView = "Config" | "Info" | "Execute" | "Commit";

export function useResourceSyncTabsView(sync: Types.ResourceSync | undefined) {
  const [_view, setView] = useLocalStorage<ResourceSyncTabsView>({
    key: `sync-${sync?._id?.$oid}-tab-v1`,
    defaultValue: "Config",
  });

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
