import {
  useExecute,
  useInvalidate,
  usePermissions,
  useRead,
  useWrite,
} from "@/lib/hooks";
import { Button, Loader } from "@mantine/core";
import { ICONS } from "@/theme/icons";
import ConfirmButton from "@/ui/confirm-button";
import { useFullResourceSync } from ".";
import { useResourceSyncTabsView } from "./hooks";
import { fileContentsEmpty, resourceSyncNoChanges } from "@/lib/utils";
import ConfirmModalWithDisable from "@/components/confirm-modal-with-disable";

export function RefreshSync({ id }: { id: string }) {
  const inv = useInvalidate();
  const { mutate, isPending } = useWrite("RefreshResourceSyncPending", {
    onSuccess: () => inv(["GetResourceSync"], ["ListResourceSyncs"]),
  });
  const pending = isPending;
  return (
    <Button
      justify="space-between"
      w={190}
      rightSection={
        pending ? (
          <Loader color="white" size="1rem" />
        ) : (
          <ICONS.Refresh size="1rem" />
        )
      }
      onClick={() => mutate({ sync: id })}
      disabled={pending}
    >
      Refresh
    </Button>
  );
}

export function ExecuteSync({ id }: { id: string }) {
  const { mutateAsync: execute, isPending } = useExecute("RunSync");
  const syncing = useRead(
    "GetResourceSyncActionState",
    { sync: id },
    { refetchInterval: 5000 },
  ).data?.syncing;
  const sync = useFullResourceSync(id);
  const { view } = useResourceSyncTabsView(sync);

  if (
    view !== "Execute" ||
    !sync ||
    resourceSyncNoChanges(sync) ||
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
    <ConfirmModalWithDisable
      confirmText={sync.name}
      icon={<ICONS.Run size="1rem" />}
      onConfirm={() => execute({ sync: id })}
      disabled={pending}
      loading={pending}
    >
      Execute Sync
    </ConfirmModalWithDisable>
  );
}

export function CommitSync({ id }: { id: string }) {
  const { mutateAsync: commit, isPending } = useWrite("CommitSync");
  const sync = useFullResourceSync(id);
  const { view } = useResourceSyncTabsView(sync);
  const { canWrite } = usePermissions({ type: "ResourceSync", id });

  if (view !== "Commit" || !canWrite || !sync) {
    return null;
  }

  const freshSync =
    !sync.config?.files_on_host &&
    fileContentsEmpty(sync.config?.file_contents) &&
    !sync.config?.repo &&
    !sync.config?.linked_repo;

  if (!freshSync && (!sync.config?.managed || resourceSyncNoChanges(sync))) {
    return null;
  }

  if (freshSync) {
    return (
      <ConfirmButton
        icon={<ICONS.Commit size="1rem" />}
        onClick={() => commit({ sync: id })}
        disabled={isPending}
        loading={isPending}
      >
        Commit Changes
      </ConfirmButton>
    );
  } else {
    return (
      <ConfirmModalWithDisable
        confirmText={sync.name}
        icon={<ICONS.Commit size="1rem" />}
        onConfirm={() => commit({ sync: id })}
        disabled={isPending}
        loading={isPending}
      >
        Commit Changes
      </ConfirmModalWithDisable>
    );
  }
}
