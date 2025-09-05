import {
  ActionButton,
  ActionWithDialog,
  ConfirmButton,
} from "@components/util";
import { useExecute, useInvalidate, useRead, useWrite } from "@lib/hooks";
import { file_contents_empty, sync_no_changes } from "@lib/utils";
import { usePermissions } from "@lib/hooks";
import { NotebookPen, RefreshCcw, SquarePlay } from "lucide-react";
import {
  useFullResourceSync,
  usePendingView,
  useResourceSyncTabsView,
} from ".";

export const RefreshSync = ({ id }: { id: string }) => {
  const inv = useInvalidate();
  const { mutate, isPending } = useWrite("RefreshResourceSyncPending", {
    onSuccess: () => inv(["GetResourceSync"], ["ListResourceSyncs"]),
  });
  const pending = isPending;
  return (
    <ActionButton
      title="Refresh"
      icon={<RefreshCcw className="w-4 h-4" />}
      onClick={() => mutate({ sync: id })}
      disabled={pending}
      loading={pending}
    />
  );
};

export const ExecuteSync = ({ id }: { id: string }) => {
  const { mutate, isPending } = useExecute("RunSync");
  const syncing = useRead(
    "GetResourceSyncActionState",
    { sync: id },
    { refetchInterval: 5000 }
  ).data?.syncing;
  const sync = useFullResourceSync(id);
  const { view } = useResourceSyncTabsView(sync);
  const [_pendingView] = usePendingView();
  const pendingView = sync?.config?.managed ? _pendingView : "Execute";

  if (
    view !== "Pending" ||
    pendingView === "Commit" ||
    !sync ||
    sync_no_changes(sync) ||
    !sync.info?.remote_contents
  ) {
    return null;
  }

  let all_empty = true;
  for (const contents of sync.info.remote_contents) {
    if (contents.contents.length > 0) {
      all_empty = false;
      break;
    }
  }

  if (all_empty) return null;

  const pending = isPending || syncing;

  return (
    <ActionWithDialog
      name={sync.name}
      title="Execute Sync"
      icon={<SquarePlay className="w-4 h-4" />}
      onClick={() => mutate({ sync: id })}
      disabled={pending}
      loading={pending}
    />
  );
};

export const CommitSync = ({ id }: { id: string }) => {
  const { mutate, isPending } = useWrite("CommitSync");
  const sync = useFullResourceSync(id);
  const { view } = useResourceSyncTabsView(sync);
  const { canWrite } = usePermissions({ type: "ResourceSync", id });
  const [_pendingView] = usePendingView();
  const pendingView = sync?.config?.managed ? _pendingView : "Execute";

  if (view !== "Pending" || pendingView === "Execute" || !canWrite || !sync) {
    return null;
  }

  const freshSync =
    !sync.config?.files_on_host &&
    file_contents_empty(sync.config?.file_contents) &&
    !sync.config?.repo &&
    !sync.config?.linked_repo;

  if (!freshSync && (!sync.config?.managed || sync_no_changes(sync))) {
    return null;
  }

  if (freshSync) {
    return (
      <ConfirmButton
        title="Commit Changes"
        icon={<NotebookPen className="w-4 h-4" />}
        onClick={() => mutate({ sync: id })}
        disabled={isPending}
        loading={isPending}
      />
    );
  } else {
    return (
      <ActionWithDialog
        name={sync.name}
        title="Commit Changes"
        icon={<NotebookPen className="w-4 h-4" />}
        onClick={() => mutate({ sync: id })}
        disabled={isPending}
        loading={isPending}
      />
    );
  }
};
